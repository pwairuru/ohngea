from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.destinations.models import Destination
from app.destinations.schemas import DestinationCreate, DestinationRead, DestinationUpdate
from app.destinations.service import create_destination, list_destinations

router = APIRouter(prefix="/api/v1/destinations", tags=["destinations"])


def _links(d: Destination):
    base = f"/api/v1/destinations/{d.destination_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/destinations"}}


def _serialize(d: Destination):
    j = DestinationRead.model_validate(d, from_attributes=True).model_dump(mode="json")
    j["_links"] = _links(d)
    j["destination_uuid"] = str(d.destination_uuid)
    return j


@router.get("", response_model=dict)
async def list_dest(request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), q: Optional[str] = Query(None), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_destinations(session, domain_id, page, per_page, q)
    response.headers["X-Total-Count"] = str(total)
    return {"data": [_serialize(x) for x in items], "total": total, "page": page, "per_page": per_page}


@router.post("", response_model=dict, status_code=201)
async def create_dest(payload: DestinationCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("destination_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    dest = await create_destination(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/destinations/{dest.destination_uuid}"
    return _serialize(dest)


@router.get("/{dest_uuid}", response_model=dict)
async def get_dest(dest_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Destination).where(Destination.destination_uuid == dest_uuid, Destination.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Destination not found")
    return _serialize(d)


@router.patch("/{dest_uuid}", response_model=dict)
async def patch_dest(dest_uuid: UUID, payload: DestinationUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("destination_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Destination).where(Destination.destination_uuid == dest_uuid, Destination.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Destination not found")
    update = payload.model_dump(exclude_unset=True)
    for k, v in update.items():
        # alias
        if k == "number":
            d.destination_number = str(v)
        elif k == "enabled":
            d.destination_enabled = v
        elif k == "description":
            d.destination_description = v
        else:
            setattr(d, k, v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(d)
    return _serialize(d)


@router.delete("/{dest_uuid}", status_code=204)
async def delete_dest(dest_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("destination_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Destination).where(Destination.destination_uuid == dest_uuid, Destination.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Destination not found")
    await session.delete(d)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)
