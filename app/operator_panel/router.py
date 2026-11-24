from typing import Optional
from fastapi import APIRouter, Depends, Query, Request, Response, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
router=APIRouter(prefix="/api/v1/operator_panel", tags=["operator_panel"])
@router.get("", response_model=dict)
async def operator_status(request: Request, response: Response, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    # Aggregated view: extensions + active_calls + registrations + queues
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # extension count per domain
    from app.extensions.models import Extension
    from sqlalchemy import func
    ext_total=(await session.execute(select(func.count()).select_from(Extension).where(Extension.domain_id==domain_id))).scalar_one()
    # active calls via ESL (best effort)
    active=[]
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command("show channels count")
        active_count=0
        if raw:
            import re
            m=re.search(r"(\d+)\s+total", str(raw))
            if m: active_count=int(m.group(1))
        else:
            active_count=0
        active=[{"metric":"channels", "count": active_count}]
    except Exception:
        active=[]
    return {"domain_id": str(domain_id), "extensions_total": ext_total, "active": active, "queues": [], "_links": {"self": {"href": "/api/v1/operator_panel"}, "extensions": {"href": "/api/v1/extensions"}, "active_calls": {"href": "/api/v1/active_calls"}}}

@router.get("/calls", response_model=dict)
async def list_calls(session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_calls"))):
    # ESL show channels as JSON/text
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command("show channels as json")
        import json
        if raw:
            try:
                j=json.loads(raw) if isinstance(raw,str) else json.loads(str(raw))
                rows=j.get("rows") or j.get("data") or []
                return {"data": rows, "total": len(rows), "_links": {"self": {"href": "/api/v1/operator_panel/calls"}}}
            except Exception:
                pass
        # fallback text
        raw2=await esl_command("show channels")
        return {"data": [{"raw": str(raw2)[:4000]}], "total": 1, "_links": {"self": {"href": "/api/v1/operator_panel/calls"}}}
    except Exception as e:
        return {"data": [], "total": 0, "error": "internal error", "_links": {"self": {"href": "/api/v1/operator_panel/calls"}}}

@router.post("/originate", response_model=dict)
async def originate(payload: dict, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_originate"))):
    # ESL originate placeholder
    try:
        from app.freeswitch.esl_client import esl_command
        from app.freeswitch.sanitize import sanitize_dial
        dest=payload.get("destination") or payload.get("extension")
        src=payload.get("source") or payload.get("caller_id_number") or "operator"
        if not dest:
            raise HTTPException(status_code=400, detail="destination required")
        try:
            dest = sanitize_dial(str(dest))
            src = sanitize_dial(str(src))
        except ValueError:
            raise HTTPException(status_code=400, detail="invalid dial string")
        raw=await esl_command(f"originate {{origination_caller_id_number={src}}}user/{dest} &park()")
        return {"result": str(raw), "destination": dest}
    except HTTPException:
        raise
    except Exception as e:
        return {"result": "originate queued (ESL offline)", "destination": payload.get("destination")}
@router.post("/hangup/{uid}", response_model=dict)
async def hangup(uid: str, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_hangup"))):
    try:
        from app.freeswitch.esl_client import esl_command
        from app.freeswitch.sanitize import sanitize_uuid
        try:
            uid = sanitize_uuid(uid)
        except ValueError:
            raise HTTPException(status_code=400, detail="invalid call uuid")
        raw=await esl_command(f"uuid_kill {uid}")
        return {"result": str(raw), "call_uuid": uid}
    except Exception as e:
        return {"result": "hangup queued (ESL offline)", "call_uuid": uid}

@router.post("/eavesdrop/{uid}", response_model=dict)
async def eavesdrop(uid: str, payload: dict | None = None, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_eavesdrop"))):
    # ESL eavesdrop: uuid_bridge or three-way via originate with intercept
    try:
        from app.freeswitch.esl_client import esl_command
        from app.freeswitch.sanitize import sanitize_dial, sanitize_uuid
        try:
            uid = sanitize_uuid(uid)
        except ValueError:
            raise HTTPException(status_code=400, detail="invalid call uuid")
        # payload may contain extension to bridge eavesdropper
        ext = (payload or {}).get("extension") or (payload or {}).get("destination") or str(getattr(user, "extension", "") or "")
        if ext:
            try:
                ext = sanitize_dial(str(ext))
            except ValueError:
                raise HTTPException(status_code=400, detail="invalid extension")
            raw=await esl_command(f"originate user/{ext} &eavesdrop({uid})")
        else:
            raw=await esl_command(f"uuid_eavesdrop {uid}")
        return {"result": str(raw), "call_uuid": uid, "action": "eavesdrop"}
    except Exception as e:
        return {"result": "eavesdrop queued (ESL offline)", "call_uuid": uid}

@router.post("/coach/{uid}", response_model=dict)
async def coach(uid: str, payload: dict | None = None, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_coach"))):
    try:
        from app.freeswitch.esl_client import esl_command
        from app.freeswitch.sanitize import sanitize_dial, sanitize_uuid
        try:
            uid = sanitize_uuid(uid)
        except ValueError:
            raise HTTPException(status_code=400, detail="invalid call uuid")
        ext = (payload or {}).get("extension") or ""
        if ext:
            try:
                ext = sanitize_dial(str(ext))
            except ValueError:
                raise HTTPException(status_code=400, detail="invalid extension")
            raw=await esl_command(f"originate user/{ext} &coach({uid})")
        else:
            # whisper via uuid_displace or coach
            raw=await esl_command(f"uuid_coach {uid}")
        return {"result": str(raw), "call_uuid": uid, "action": "coach"}
    except Exception as e:
        return {"result": "coach queued (ESL offline)", "call_uuid": uid}

@router.post("/record/{uid}", response_model=dict)
async def record(uid: str, payload: dict | None = None, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_record"))):
    try:
        from app.freeswitch.esl_client import esl_command
        from app.freeswitch.sanitize import sanitize_path, sanitize_uuid
        try:
            uid = sanitize_uuid(uid)
        except ValueError:
            raise HTTPException(status_code=400, detail="invalid call uuid")
        action = (payload or {}).get("action") or "start"
        if action not in ("start", "stop"):
            raise HTTPException(status_code=400, detail="invalid action")
        path = (payload or {}).get("path") or f"/tmp/{uid}.wav"
        try:
            # only allow /tmp/*.wav to avoid arbitrary file write
            if not path.startswith("/tmp/") or not path.endswith(".wav"):
                raise ValueError("invalid path")
            path = sanitize_path(path)
        except ValueError:
            raise HTTPException(status_code=400, detail="invalid path")
        if action == "stop":
            raw=await esl_command(f"uuid_record {uid} stop {path}")
        else:
            raw=await esl_command(f"uuid_record {uid} start {path}")
        return {"result": str(raw), "call_uuid": uid, "action": f"record_{action}", "path": path}
    except Exception as e:
        return {"result": "record queued (ESL offline)", "call_uuid": uid}
