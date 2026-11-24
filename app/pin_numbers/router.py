from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.pin_numbers.models import PinNumber
from app.pin_numbers.schemas import PinNumberCreate, PinNumberRead, PinNumberUpdate
from app.pin_numbers.service import create_pin, list_pins
router = APIRouter(prefix="/api/v1/pin_numbers", tags=["pin_numbers"])
def _links(o: PinNumber):
    return {"self": {"href": f"/api/v1/pin_numbers/{o.pin_number_uuid}"}, "collection": {"href": "/api/v1/pin_numbers"}}
def _ser(o: PinNumber):
    d = PinNumberRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(o)
    d["pin_number_uuid"] = str(o.pin_number_uuid)
    return d
@router.get("", response_model=dict)
async def list_pins_ep(request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), q: Optional[str] = Query(None), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_pins(session, domain_id, page, per_page, q)
    response.headers["X-Total-Count"] = str(total)
    return {"data": [_ser(x) for x in items], "total": total, "page": page, "per_page": per_page}
@router.post("", response_model=dict, status_code=201)
async def create_pin_ep(payload: PinNumberCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("pin_number_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    obj = await create_pin(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/pin_numbers/{obj.pin_number_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_pin(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(PinNumber).where(PinNumber.pin_number_uuid == uid, PinNumber.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="PIN not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_pin(uid: UUID, payload: PinNumberUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("pin_number_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(PinNumber).where(PinNumber.pin_number_uuid == uid, PinNumber.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="PIN not found")
    for k, v in payload.model_dump(exclude_unset=True).items():
        setattr(o, k if hasattr(o, k) else k, v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(o)
    return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_pin(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("pin_number_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(PinNumber).where(PinNumber.pin_number_uuid == uid, PinNumber.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="PIN not found")
    await session.delete(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
