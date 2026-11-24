from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.call_forward.models import CallForward
from app.call_forward.schemas import CallForwardCreate, CallForwardRead, CallForwardUpdate
from app.call_forward.service import create_call_forward, list_call_forward, get_call_forward
from app.freeswitch.cache import invalidate_fs_cache_for_session

router = APIRouter(prefix="/api/v1/call_forward", tags=["call_forward"])


def _links(cf: CallForward):
    base = f"/api/v1/call_forward/{cf.call_forward_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/call_forward"}}


def _serialize(cf: CallForward):
    d = CallForwardRead.model_validate(cf, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(cf)
    d["call_forward_uuid"] = str(cf.call_forward_uuid)
    d["extension_uuid"] = str(cf.extension_uuid) if cf.extension_uuid else None
    d["domain_id"] = str(cf.domain_id)
    return d


@router.get("", response_model=dict)
async def list_cf(request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), q: Optional[str] = Query(None), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_call_forward(session, domain_id, page, per_page, q)
    response.headers["X-Total-Count"] = str(total)
    return {"data": [_serialize(x) for x in items], "total": total, "page": page, "per_page": per_page}


@router.post("", response_model=dict, status_code=201)
async def create_cf(payload: CallForwardCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_forward_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    if payload.domain_uuid and str(payload.domain_uuid) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    cf = await create_call_forward(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/call_forward/{cf.call_forward_uuid}"
    return _serialize(cf)


@router.get("/{cf_uuid}", response_model=dict)
async def get_cf(cf_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    cf = await get_call_forward(session, cf_uuid, domain_id)
    if not cf:
        raise HTTPException(status_code=404, detail="Call forward not found")
    return _serialize(cf)


@router.patch("/{cf_uuid}", response_model=dict)
async def patch_cf(cf_uuid: UUID, payload: CallForwardUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_forward_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    cf = await get_call_forward(session, cf_uuid, domain_id)
    if not cf:
        raise HTTPException(status_code=404, detail="Call forward not found")
    update = payload.model_dump(exclude_unset=True)
    # handle aliases
    if "forward_destination" in update:
        update["forward_all_destination"] = update.pop("forward_destination")
    if "forward_enabled" in update:
        update["forward_all_enabled"] = update.pop("forward_enabled")
    for k, v in update.items():
        setattr(cf, k, v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(cf)
    return _serialize(cf)


@router.delete("/{cf_uuid}", status_code=204)
async def delete_cf(cf_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_forward_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    cf = await get_call_forward(session, cf_uuid, domain_id)
    if not cf:
        raise HTTPException(status_code=404, detail="Call forward not found")
    await session.delete(cf)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
