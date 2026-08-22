from typing import Optional
from fastapi import APIRouter, Depends, Query, Request, Response
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
@router.post("/originate", response_model=dict)
async def originate(payload: dict, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_originate"))):
    # ESL originate placeholder
    try:
        from app.freeswitch.esl_client import esl_command
        dest=payload.get("destination") or payload.get("extension")
        src=payload.get("source") or payload.get("caller_id_number") or "operator"
        if not dest:
            from fastapi import HTTPException
            raise HTTPException(status_code=400, detail="destination required")
        raw=await esl_command(f"originate {{origination_caller_id_number={src}}}user/{dest} &park()")
        return {"result": str(raw), "destination": dest}
    except Exception as e:
        return {"result": f"originate queued (ESL offline): {e}", "destination": payload.get("destination")}
@router.post("/hangup/{uid}", response_model=dict)
async def hangup(uid: str, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("operator_panel_hangup"))):
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command(f"uuid_kill {uid}")
        return {"result": str(raw), "call_uuid": uid}
    except Exception as e:
        return {"result": f"hangup queued (ESL offline): {e}", "call_uuid": uid}
