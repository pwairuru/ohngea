from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.music_on_hold.models import MusicOnHold
from app.music_on_hold.schemas import MusicOnHoldCreate, MusicOnHoldRead, MusicOnHoldUpdate
from app.music_on_hold.service import create_moh, list_moh as service_list_moh

router = APIRouter(prefix="/api/v1/music_on_hold", tags=["music_on_hold"])


def _links(m: MusicOnHold):
    base = f"/api/v1/music_on_hold/{m.music_on_hold_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/music_on_hold"}}


def _serialize(m: MusicOnHold):
    d = MusicOnHoldRead.model_validate(m, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(m)
    d["music_on_hold_uuid"] = str(m.music_on_hold_uuid)
    return d


@router.get("", response_model=dict)
async def list_moh(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("music_name"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await service_list_moh(session, domain_id, page, per_page, q, sort, order)
    response.headers["X-Total-Count"] = str(total)
    base_url = str(request.url).split("?")[0]
    if page * per_page < total:
        from urllib.parse import urlencode
        qs = dict(request.query_params)
        qs["page"] = str(page + 1)
        response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="next"'
    return {"data": [_serialize(m) for m in items], "total": total, "page": page, "per_page": per_page}


@router.get("/health", response_model=dict)
async def health():
    return {"status": "ok", "mode": "core"}


@router.post("", response_model=dict, status_code=201)
async def create_moh_ep(payload: MusicOnHoldCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("music_on_hold_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    if not payload.domain_id:
        payload.domain_id = domain_id
    await set_tenant(session, domain_id)
    m = await create_moh(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/music_on_hold/{m.music_on_hold_uuid}"
    return _serialize(m)


@router.get("/{moh_uuid}", response_model=dict)
async def get_moh(moh_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(MusicOnHold).where(MusicOnHold.music_on_hold_uuid == moh_uuid, MusicOnHold.domain_id == domain_id))
    m = r.scalar_one_or_none()
    if not m:
        raise HTTPException(status_code=404, detail="Music on hold not found")
    return _serialize(m)


@router.patch("/{moh_uuid}", response_model=dict)
async def patch_moh(moh_uuid: UUID, payload: MusicOnHoldUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("music_on_hold_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(MusicOnHold).where(MusicOnHold.music_on_hold_uuid == moh_uuid, MusicOnHold.domain_id == domain_id))
    m = r.scalar_one_or_none()
    if not m:
        raise HTTPException(status_code=404, detail="Music on hold not found")
    update = payload.model_dump(exclude_unset=True)
    for k, v in update.items():
        # map aliases
        if k == "name":
            m.music_name = v
        elif k == "path":
            m.music_path = v
        elif k == "rate":
            m.music_rate = v
        else:
            setattr(m, k if hasattr(m, k) else k, v)
    await session.commit()
    await session.refresh(m)
    return _serialize(m)


@router.delete("/{moh_uuid}", status_code=204)
async def delete_moh(moh_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("music_on_hold_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(MusicOnHold).where(MusicOnHold.music_on_hold_uuid == moh_uuid, MusicOnHold.domain_id == domain_id))
    m = r.scalar_one_or_none()
    if not m:
        raise HTTPException(status_code=404, detail="Music on hold not found")
    await session.delete(m)
    await session.commit()
    return Response(status_code=204)
