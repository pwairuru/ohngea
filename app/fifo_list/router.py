"""FIFO List — live FIFO queue status.

Thin functional router for parity with legacy fifo_list app.
Tries ESL `fifo list` JSON, falls back to DB fifo.
"""
from typing import Optional

from fastapi import APIRouter, Depends, Query, Request, Response
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1/fifo_list", tags=["fifo_list"])


@router.get("", response_model=dict)
async def list_fifo_status(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    q: Optional[str] = Query(None),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    """Show FIFO status — ESL `fifo list` or DB fallback with waiting count placeholder."""
    # Try live FS fifo list
    try:
        from app.freeswitch.esl_client import esl_command
        import json

        raw = await esl_command("fifo list")
        if raw and raw.strip():
            # Try JSON parse
            j = None
            try:
                if raw.strip().startswith("{") or raw.strip().startswith("["):
                    j = json.loads(raw) if isinstance(raw, str) else raw
            except Exception:
                j = None
            if j is not None:
                rows = j.get("rows") or j.get("fifo") or j.get("data") or []
                if isinstance(rows, dict):
                    rows = list(rows.values())
                if q:
                    rows = [r for r in rows if q.lower() in str(r).lower()]
                total = len(rows)
                start = (page - 1) * per_page
                slice_rows = rows[start : start + per_page]
                data = []
                for r in slice_rows:
                    if isinstance(r, dict):
                        data.append(
                            {
                                "fifo_name": r.get("fifo_name") or r.get("name") or str(r),
                                "consumer_count": r.get("consumer_count") or r.get("consumers") or 0,
                                "caller_count": r.get("caller_count") or r.get("callers") or 0,
                                "status": r.get("status") or "active",
                                "_raw": r,
                            }
                        )
                    else:
                        data.append({"fifo_name": str(r)})
                response.headers["X-Total-Count"] = str(total)
                return {"data": data, "total": total, "page": page, "per_page": per_page, "source": "esl", "_links": {"self": {"href": "/api/v1/fifo_list"}}}
            # Text fallback: parse lines like "fifo_name <calls> <consumers>"
            lines = [l.strip() for l in raw.splitlines() if l.strip() and not l.lower().startswith("content-")]
            # filter empty header lines
            if lines and any("fifo" in l.lower() for l in lines[:2]):
                # assume first line is header, skip
                header = lines[0]
                body = lines[1:]
            else:
                body = lines
            if q:
                body = [l for l in body if q.lower() in l.lower()]
            total = len(body)
            start = (page - 1) * per_page
            data = [{"fifo_name": l.split()[0] if l.split() else l, "raw": l} for l in body[start : start + per_page]]
            if data or raw.strip():
                # if we got non-empty text, return esl_text
                if data:
                    response.headers["X-Total-Count"] = str(total)
                    return {"data": data, "total": total, "page": page, "per_page": per_page, "source": "esl_text", "_links": {"self": {"href": "/api/v1/fifo_list"}}}
    except Exception:
        pass

    # DB fallback
    from app.fifo.service import list_fifo

    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_fifo(session, domain_id, page, per_page, q)
    response.headers["X-Total-Count"] = str(total)
    return {
        "data": [
            {
                "fifo_name": c.fifo_name,
                "fifo_extension": getattr(c, "fifo_extension", None),
                "fifo_enabled": getattr(c, "fifo_enabled", True),
                "fifo_description": getattr(c, "fifo_description", None),
                "consumer_count": 0,
                "caller_count": 0,
                "status": "DB",
                "fifo_uuid": str(c.fifo_uuid),
            }
            for c in items
        ],
        "total": total,
        "page": page,
        "per_page": per_page,
        "source": "db",
        "_links": {"self": {"href": "/api/v1/fifo_list"}},
    }


@router.get("/status", response_model=dict)
async def fifo_status_summary(
    session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)
):
    """Summary counts — delegates to list but single page."""
    # Reuse list logic for count
    from app.fifo.service import list_fifo

    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    _, total = await list_fifo(session, domain_id, 1, 1, None)
    # Try live count
    try:
        from app.freeswitch.esl_client import esl_command

        raw = await esl_command("fifo list")
        live = None
        if raw and raw.strip() and "connection" not in raw.lower():
            live = len([l for l in raw.splitlines() if l.strip() and not l.lower().startswith("content-")])
    except Exception:
        live = None
    return {"total_fifo": total, "live_lines": live, "_links": {"self": {"href": "/api/v1/fifo_list/status"}, "collection": {"href": "/api/v1/fifo_list"}}}
