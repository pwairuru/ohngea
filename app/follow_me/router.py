from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.follow_me.models import FollowMe, FollowMeDestination
from app.follow_me.schemas import FollowMeCreate, FollowMeRead, FollowMeUpdate
from app.follow_me.service import create_follow_me, list_follow_me, get_follow_me_with_dests

router = APIRouter(prefix="/api/v1/follow_me", tags=["follow_me"])


def _links(fm: FollowMe):
    base = f"/api/v1/follow_me/{fm.follow_me_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/follow_me"}}


def _serialize(fm: FollowMe, dests=None):
    d = FollowMeRead.model_validate(fm, from_attributes=True).model_dump(mode="json")
    if dests is not None:
        d["destinations"] = [{"follow_me_destination_uuid": str(x.follow_me_destination_uuid), "follow_me_destination": x.follow_me_destination, "follow_me_delay": x.follow_me_delay, "follow_me_timeout": x.follow_me_timeout} for x in dests]
    d["_links"] = _links(fm)
    d["follow_me_uuid"] = str(fm.follow_me_uuid)
    return d


@router.get("", response_model=dict)
async def list_fm(request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), q: Optional[str] = Query(None), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_follow_me(session, domain_id, page, per_page, q)
    response.headers["X-Total-Count"] = str(total)
    data = []
    for fm in items:
        _, dests = await get_follow_me_with_dests(session, fm.follow_me_uuid, domain_id)
        data.append(_serialize(fm, dests))
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", response_model=dict, status_code=201)
async def create_fm(payload: FollowMeCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("follow_me_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    fm = await create_follow_me(session, payload, domain_id)
    _, dests = await get_follow_me_with_dests(session, fm.follow_me_uuid, domain_id)
    response.headers["Location"] = f"/api/v1/follow_me/{fm.follow_me_uuid}"
    return _serialize(fm, dests)


@router.get("/{fm_uuid}", response_model=dict)
async def get_fm(fm_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    fm, dests = await get_follow_me_with_dests(session, fm_uuid, domain_id)
    if not fm:
        raise HTTPException(status_code=404, detail="Follow me not found")
    return _serialize(fm, dests)


@router.patch("/{fm_uuid}", response_model=dict)
async def patch_fm(fm_uuid: UUID, payload: FollowMeUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("follow_me_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    fm, _ = await get_follow_me_with_dests(session, fm_uuid, domain_id)
    if not fm:
        raise HTTPException(status_code=404, detail="Follow me not found")
    update = payload.model_dump(exclude_unset=True, exclude={"destinations"})
    for k, v in update.items():
        setattr(fm, k, v)
    if payload.destinations is not None:
        # replace dests
        existing = await session.execute(select(FollowMeDestination).where(FollowMeDestination.follow_me_uuid == fm_uuid))
        for e in existing.scalars().all():
            await session.delete(e)
        await session.flush()
        for dest in payload.destinations:
            if isinstance(dest, str):
                d = FollowMeDestination(domain_id=domain_id, follow_me_uuid=fm.follow_me_uuid, follow_me_destination=dest)
                session.add(d)
            elif isinstance(dest, dict):
                num = dest.get("follow_me_destination") or dest.get("destination")
                if num:
                    d = FollowMeDestination(domain_id=domain_id, follow_me_uuid=fm.follow_me_uuid, follow_me_destination=str(num), follow_me_delay=int(dest.get("follow_me_delay", 0) or 0), follow_me_timeout=int(dest.get("follow_me_timeout", 30) or 30))
                    session.add(d)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(fm)
    _, dests = await get_follow_me_with_dests(session, fm.follow_me_uuid, domain_id)
    return _serialize(fm, dests)


@router.delete("/{fm_uuid}", status_code=204)
async def delete_fm(fm_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("follow_me_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    fm, _ = await get_follow_me_with_dests(session, fm_uuid, domain_id)
    if not fm:
        raise HTTPException(status_code=404, detail="Follow me not found")
    await session.delete(fm)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)
