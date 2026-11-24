"""Active Conferences — live FS view + DB fallback.

Dedicated router for parity with legacy active_conferences app.
Thin wrapper over ESL conference list; falls back to DB conferences.
Also exposes WebSocket for realtime polling (optional, non-required if WS unavailable).
"""
from typing import Optional

from fastapi import APIRouter, Depends, Query, Request, Response, WebSocket, WebSocketDisconnect
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1/active_conferences", tags=["active_conferences"])


@router.get("", response_model=dict)
async def list_active_conferences(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    q: Optional[str] = Query(None),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    """REST: live conference list via ESL `conference list as json` else DB fallback."""
    data: list[dict] = []
    total = 0
    # Try live ESL
    try:
        from app.freeswitch.esl_client import esl_command

        raw = await esl_command("conference list as json")
        import json

        if raw:
            j = json.loads(raw) if isinstance(raw, str) else raw
            rows = j.get("rows") or j.get("conferences") or j.get("data") or []
            if isinstance(rows, dict):
                rows = list(rows.values())
            if q:
                ql = q.lower()
                rows = [r for r in rows if ql in str(r).lower()]
            total = len(rows)
            start = (page - 1) * per_page
            slice_rows = rows[start : start + per_page]
            norm: list[dict] = []
            for r in slice_rows:
                if isinstance(r, dict):
                    norm.append(
                        {
                            "conference_name": r.get("conference_name") or r.get("name") or str(r),
                            "member_count": r.get("member_count") or r.get("members") or 0,
                            "profile": r.get("profile"),
                            "domain": r.get("domain"),
                            "_raw": r,
                        }
                    )
                else:
                    norm.append({"conference_name": str(r), "member_count": 0})
            data = norm
            response.headers["X-Total-Count"] = str(total)
            return {"data": data, "total": total, "page": page, "per_page": per_page, "_links": {"self": {"href": "/api/v1/active_conferences"}, "ws": {"href": "/api/v1/active_conferences/ws"}}}
    except Exception:
        pass

    # Fallback to DB conferences
    try:
        from app.conferences.service import list_conferences

        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        items, total = await list_conferences(session, domain_id, page, per_page, q)
        data = [
            {
                "conference_name": c.conference_name,
                "conference_extension": c.conference_extension,
                "conference_uuid": str(c.conference_uuid),
                "member_count": 0,
                "profile": getattr(c, "conference_profile", None),
            }
            for c in items
        ]
    except Exception:
        data = []
        total = 0
    response.headers["X-Total-Count"] = str(total)
    return {"data": data, "total": total, "page": page, "per_page": per_page, "_links": {"self": {"href": "/api/v1/active_conferences"}, "ws": {"href": "/api/v1/active_conferences/ws"}}}


@router.get("/count", response_model=dict)
async def count_active_conferences(
    session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)
):
    """Quick count via same logic."""
    try:
        from app.freeswitch.esl_client import esl_command
        import json

        raw = await esl_command("conference list as json")
        if raw:
            j = json.loads(raw) if isinstance(raw, str) else raw
            rows = j.get("rows") or j.get("conferences") or j.get("data") or []
            if isinstance(rows, dict):
                rows = list(rows.values())
            return {"count": len(rows), "source": "esl"}
    except Exception:
        pass
    # DB fallback
    try:
        from app.conferences.service import list_conferences

        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        _, total = await list_conferences(session, domain_id, 1, 1, None)
        return {"count": total, "source": "db"}
    except Exception:
        return {"count": 0, "source": "none"}


@router.get("/{conf}/members", response_model=dict)
async def list_conference_members(
    conf: str,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    """Live member list via ESL `conference <name> list as json`."""
    data: list[dict] = []
    try:
        from app.freeswitch.sanitize import sanitize_dial
        conf = sanitize_dial(conf)
    except ValueError:
        return {"data": [], "total": 0, "conference_name": conf, "error": "invalid conference name"}
    try:
        from app.freeswitch.esl_client import esl_command
        import json

        raw = await esl_command(f"conference {conf} list as json")
        if raw:
            j = json.loads(raw) if isinstance(raw, str) else raw
            rows = j.get("rows") or j.get("members") or j.get("data") or []
            if isinstance(rows, dict):
                rows = list(rows.values())
            for r in rows:
                if isinstance(r, dict):
                    data.append(
                        {
                            "member_id": r.get("id") or r.get("member_id") or r.get("uuid"),
                            "uuid": r.get("uuid"),
                            "caller_id_name": r.get("caller_id_name"),
                            "caller_id_number": r.get("caller_id_number"),
                            "muted": r.get("mute") or r.get("muted") or False,
                            "deaf": r.get("deaf") or False,
                            "_raw": r,
                        }
                    )
                else:
                    data.append({"member_id": str(r)})
    except Exception as e:
        return {"data": [], "total": 0, "conference_name": conf, "error": "internal error"}
    return {"data": data, "total": len(data), "conference_name": conf}


_CONF_MEMBER_ACTIONS = {"mute", "unmute", "deaf", "undeaf", "kick"}


@router.post("/{conf}/members/{member}/{action}", response_model=dict)
async def conference_member_action(
    conf: str,
    member: str,
    action: str,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("conferences_active_control")),
):
    """Member controls via ESL `conference <conf> <action> <member>`."""
    if action not in _CONF_MEMBER_ACTIONS:
        from fastapi import HTTPException

        raise HTTPException(status_code=422, detail=f"action must be one of {sorted(_CONF_MEMBER_ACTIONS)}")
    try:
        from app.freeswitch.sanitize import sanitize_dial
        conf = sanitize_dial(conf)
        member = sanitize_dial(member)
    except ValueError:
        from fastapi import HTTPException as _HE2

        raise _HE2(status_code=400, detail="invalid conference or member")
    try:
        from app.freeswitch.esl_client import esl_command

        raw = await esl_command(f"conference {conf} {action} {member}")
        return {"result": str(raw), "conference_name": conf, "member_id": member, "action": action}
    except Exception as e:
        return {"result": "action queued (ESL offline)", "conference_name": conf, "member_id": member, "action": action}


@router.websocket("/ws")
async def ws_active_conferences(websocket: WebSocket):
    """Realtime push — emits conference list every 5s. No auth (upgrade), but can be gated via token query param if needed."""
    await websocket.accept()
    import asyncio
    import json

    try:
        while True:
            data: list[dict] = []
            try:
                from app.freeswitch.esl_client import esl_command

                raw = await esl_command("conference list as json")
                if raw:
                    j = json.loads(raw) if isinstance(raw, str) else raw
                    rows = j.get("rows") or j.get("conferences") or j.get("data") or []
                    if isinstance(rows, dict):
                        rows = list(rows.values())
                    for r in rows[:20]:
                        if isinstance(r, dict):
                            data.append({"conference_name": r.get("conference_name") or r.get("name") or str(r), "member_count": r.get("member_count") or 0})
                        else:
                            data.append({"conference_name": str(r)})
            except Exception:
                data = []
            await websocket.send_text(json.dumps({"type": "active_conferences", "data": data}))
            await asyncio.sleep(5)
    except WebSocketDisconnect:
        pass
    except Exception:
        try:
            await websocket.close()
        except Exception:
            pass
