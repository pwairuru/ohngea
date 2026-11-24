from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.ring_groups.models import RingGroup, RingGroupMember
from app.ring_groups.schemas import RingGroupCreate, RingGroupRead, RingGroupUpdate, RingGroupDestinationRead
from app.ring_groups.service import create_ring_group, list_ring_groups, get_ring_group_with_members

router = APIRouter(prefix="/api/v1/ring_groups", tags=["ring_groups"])


def _links(rg: RingGroup):
    base = f"/api/v1/ring_groups/{rg.ring_group_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/ring_groups"}}


def _serialize(rg: RingGroup, members: list[RingGroupMember] | None = None):
    # base dict from model_validate
    d = RingGroupRead.model_validate(rg, from_attributes=True).model_dump(mode="json")
    # members already included via model? But we need to override with actual members list
    if members is not None:
        d["members"] = [RingGroupDestinationRead.model_validate(m).model_dump(mode="json") for m in members]
        d["destinations"] = d["members"]
    else:
        # ensure fields exist
        d.setdefault("members", [])
        d.setdefault("destinations", d["members"])
    d["_links"] = _links(rg)
    # also provide legacy keys for drop-in compatibility
    d["ring_group_uuid"] = str(rg.ring_group_uuid)
    d["ring_group_name"] = rg.name
    d["ring_group_extension"] = rg.extension
    d["ring_group_strategy"] = rg.strategy
    d["ring_group_description"] = rg.description
    return d


@router.get("", response_model=dict, dependencies=[Depends(require_permission("ring_group_view"))])
async def list_rg(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("name"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    filter_name: Optional[str] = Query(None, alias="filter[name]"),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_strategy: Optional[str] = Query(None, alias="filter[strategy]"),
    filter_enabled: Optional[bool] = Query(None, alias="filter[enabled]"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)

    # support alias filter[extension] already handled; also support legacy filter[ring_group_extension]
    # also support generic q param via filter[name] etc already mapping

    items, total = await list_ring_groups(
        session, domain_id, page, per_page, q, sort, order,
        filter_extension=filter_extension,
        filter_name=filter_name,
        filter_strategy=filter_strategy,
        filter_enabled=filter_enabled,
    )

    response.headers["X-Total-Count"] = str(total)
    base_url = str(request.url).split("?")[0]
    if page * per_page < total:
        next_page = page + 1
        from urllib.parse import urlencode

        qs = dict(request.query_params)
        qs["page"] = str(next_page)
        qs["per_page"] = str(per_page)
        response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="next"'
        if page > 1:
            qs["page"] = str(page - 1)
            response.headers["Link"] += f', <{base_url}?{urlencode(qs)}>; rel="prev"'
    else:
        if page > 1:
            from urllib.parse import urlencode

            qs = dict(request.query_params)
            qs["page"] = str(page - 1)
            response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="prev"'

    data = []
    for rg in items:
        # fetch members for each (N+1 but per_page limited to 100)
        _, members = await get_ring_group_with_members(session, rg.ring_group_uuid, domain_id)
        data.append(_serialize(rg, members))
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/health")
async def health():
    return {"plugin": "ring_groups", "status": "ok", "mode": "core"}


@router.post("", response_model=dict, status_code=201)
async def create_rg(
    payload: RingGroupCreate,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("ring_group_add")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if not payload.domain_id:
        payload.domain_id = domain_id
    elif str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    rg = await create_ring_group(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/ring_groups/{rg.ring_group_uuid}"
    _, members = await get_ring_group_with_members(session, rg.ring_group_uuid, domain_id)
    result = _serialize(rg, members)
    return result


@router.get("/{ring_group_uuid}", response_model=dict, dependencies=[Depends(require_permission("ring_group_view"))])
async def get_rg(ring_group_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    rg, members = await get_ring_group_with_members(session, ring_group_uuid, domain_id)
    if not rg:
        raise HTTPException(status_code=404, detail="Ring group not found")
    return _serialize(rg, members)


@router.patch("/{ring_group_uuid}", response_model=dict)
async def patch_rg(
    ring_group_uuid: UUID,
    payload: RingGroupUpdate,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("ring_group_edit")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(RingGroup).where(RingGroup.ring_group_uuid == ring_group_uuid, RingGroup.domain_id == domain_id))
    rg = r.scalar_one_or_none()
    if not rg:
        raise HTTPException(status_code=404, detail="Ring group not found")
    update = payload.model_dump(exclude_unset=True, exclude={"members", "destinations"})
    for k, v in update.items():
        setattr(rg, k, v)
    # handle members update if provided
    members_payload = payload.members if payload.members is not None else payload.destinations
    if members_payload is not None:
        # delete existing members and recreate
        existing = await session.execute(select(RingGroupMember).where(RingGroupMember.ring_group_uuid == ring_group_uuid))
        for m in existing.scalars().all():
            await session.delete(m)
        await session.flush()
        for item in members_payload:
            if isinstance(item, str):
                dest = item.strip()
                if dest:
                    m = RingGroupMember(domain_id=domain_id, ring_group_uuid=rg.ring_group_uuid, destination_number=dest)
                    session.add(m)
            elif isinstance(item, dict):
                num = item.get("destination_number") or item.get("number") or item.get("extension")
                if num:
                    m = RingGroupMember(
                        domain_id=domain_id,
                        ring_group_uuid=rg.ring_group_uuid,
                        destination_number=str(num).strip(),
                        destination_delay=int(item.get("destination_delay", item.get("delay", 0)) or 0),
                        destination_timeout=int(item.get("destination_timeout", item.get("timeout", 30)) or 30),
                        destination_enabled=bool(item.get("destination_enabled", item.get("enabled", True))),
                        destination_description=item.get("destination_description") or item.get("description"),
                    )
                    session.add(m)
            else:
                # pydantic model
                try:
                    d = item.model_dump() if hasattr(item, "model_dump") else dict(item)
                    num = d.get("destination_number")
                    if num:
                        m = RingGroupMember(
                            domain_id=domain_id,
                            ring_group_uuid=rg.ring_group_uuid,
                            destination_number=str(num).strip(),
                            destination_delay=int(d.get("destination_delay", 0) or 0),
                            destination_timeout=int(d.get("destination_timeout", 30) or 30),
                            destination_enabled=bool(d.get("destination_enabled", True)),
                            destination_description=d.get("destination_description"),
                        )
                        session.add(m)
                except Exception:
                    continue
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(rg)
    _, members = await get_ring_group_with_members(session, rg.ring_group_uuid, domain_id)
    return _serialize(rg, members)


@router.delete("/{ring_group_uuid}", status_code=204)
async def delete_rg(ring_group_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("ring_group_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(RingGroup).where(RingGroup.ring_group_uuid == ring_group_uuid, RingGroup.domain_id == domain_id))
    rg = r.scalar_one_or_none()
    if not rg:
        raise HTTPException(status_code=404, detail="Ring group not found")
    await session.delete(rg)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
