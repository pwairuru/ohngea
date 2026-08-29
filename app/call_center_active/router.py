"""Call Center Active — live queue/agent view.

Thin functional router for parity with legacy call_center_active app.
Tries ESL `callcenter_config list queues/agents`, falls back to DB call_centers.
"""
from typing import Optional

from fastapi import APIRouter, Depends, Query, Request, Response
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1/call_center_active", tags=["call_center_active"])


@router.get("", response_model=dict)
async def list_call_center_active(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    q: Optional[str] = Query(None),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    """List active call-center queues — ESL first, DB fallback."""
    # Try ESL live
    try:
        from app.freeswitch.esl_client import esl_command
        import json

        raw = await esl_command("callcenter_config queue list")
        if raw and raw.strip():
            # ESL may return JSON or pipe-delimited text; handle both
            data: list[dict] = []
            # try JSON
            try:
                j = json.loads(raw) if isinstance(raw, str) and raw.strip().startswith("{") else None
                if j:
                    rows = j.get("rows") or j.get("queues") or j.get("data") or []
                    if isinstance(rows, dict):
                        rows = list(rows.values())
                    if q:
                        rows = [r for r in rows if q.lower() in str(r).lower()]
                    total = len(rows)
                    start = (page - 1) * per_page
                    data = rows[start : start + per_page]
                    if data and isinstance(data[0], dict):
                        norm = [{"queue_name": r.get("name") or r.get("queue_name") or str(r), "queue_status": r.get("status"), "_raw": r} for r in data]
                    else:
                        norm = [{"queue_name": str(r)} for r in data]
                    response.headers["X-Total-Count"] = str(total)
                    return {"data": norm, "total": total, "page": page, "per_page": per_page, "source": "esl", "_links": {"self": {"href": "/api/v1/call_center_active"}}}
            except Exception:
                pass
            # fallback text parse — return raw snippet
            lines = [l.strip() for l in raw.splitlines() if l.strip() and not l.lower().startswith("content-")]
            if lines:
                # filter by q
                if q:
                    lines = [l for l in lines if q.lower() in l.lower()]
                total = len(lines)
                start = (page - 1) * per_page
                data = [{"queue_name": l[:120], "raw": l} for l in lines[start : start + per_page]]
                response.headers["X-Total-Count"] = str(total)
                return {"data": data, "total": total, "page": page, "per_page": per_page, "source": "esl_text", "_links": {"self": {"href": "/api/v1/call_center_active"}}}
    except Exception:
        pass

    # DB fallback
    from app.call_centers.service import list_cc

    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_cc(session, domain_id, page, per_page, q)
    response.headers["X-Total-Count"] = str(total)
    return {
        "data": [
            {
                "queue_name": c.queue_name,
                "queue_extension": c.queue_extension,
                "queue_strategy": getattr(c, "queue_strategy", None),
                "queue_description": getattr(c, "queue_description", None),
                "call_center_queue_uuid": str(c.call_center_queue_uuid),
                "status": "DB",
            }
            for c in items
        ],
        "total": total,
        "page": page,
        "per_page": per_page,
        "source": "db",
        "_links": {"self": {"href": "/api/v1/call_center_active"}},
    }


@router.get("/{queue}", response_model=dict)
async def get_queue_active(
    queue: str,
    request: Request,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    """Detail for a single queue — tries ESL agents, falls back to DB."""
    try:
        from app.freeswitch.esl_client import esl_command

        raw = await esl_command(f"callcenter_config queue list agents {queue}")
        if raw and raw.strip() and "error" not in raw.lower()[:100]:
            return {"queue": queue, "agents_raw": raw[:2000], "source": "esl", "_links": {"self": {"href": f"/api/v1/call_center_active/{queue}"}, "collection": {"href": "/api/v1/call_center_active"}}}
    except Exception:
        pass
    # DB lookup
    from sqlalchemy import select
    from app.call_centers.models import CallCenter

    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallCenter).where(CallCenter.domain_id == domain_id, CallCenter.queue_name == queue))
    cc = r.scalar_one_or_none()
    if not cc:
        # also try by extension
        r = await session.execute(select(CallCenter).where(CallCenter.domain_id == domain_id, CallCenter.queue_extension == queue))
        cc = r.scalar_one_or_none()
    if not cc:
        from fastapi import HTTPException

        raise HTTPException(status_code=404, detail="Queue not found")
    # fetch agents
    from app.call_centers.service import get_cc_with_agents

    _, agents = await get_cc_with_agents(session, cc.call_center_queue_uuid, domain_id)
    return {
        "queue_name": cc.queue_name,
        "queue_extension": cc.queue_extension,
        "queue_strategy": getattr(cc, "queue_strategy", None),
        "agents": [{"agent_name": a.agent_name, "agent_status": a.agent_status, "agent_contact": a.agent_contact} for a in (agents or [])],
        "source": "db",
        "_links": {"self": {"href": f"/api/v1/call_center_active/{queue}"}, "collection": {"href": "/api/v1/call_center_active"}},
    }
