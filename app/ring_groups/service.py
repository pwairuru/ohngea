from typing import Optional, List, Any
from uuid import UUID

from fastapi import HTTPException
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.ring_groups.models import RingGroup, RingGroupMember
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session


async def create_ring_group(
    session: AsyncSession,
    data,
    domain_id: UUID,
) -> RingGroup:
    # data is RingGroupCreate instance
    # check duplicate extension per domain
    existing = await session.execute(
        select(RingGroup).where(RingGroup.domain_id == domain_id, RingGroup.extension == data.extension)
    )
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Ring group extension already exists")

    # extract members/destinations before model creation
    members_raw = getattr(data, "members", None)
    destinations_raw = getattr(data, "destinations", None)

    # combine members and destinations payloads
    raw_list: List[Any] = []
    if members_raw:
        raw_list.extend(members_raw)
    if destinations_raw:
        raw_list.extend([d.model_dump() if hasattr(d, "model_dump") else d for d in destinations_raw])

    # build RingGroup fields excluding members/destinations
    payload = data.model_dump(exclude={"members", "destinations"}, exclude_unset=True)
    # ensure domain_id matches tenant (override)
    payload["domain_id"] = domain_id
    rg = RingGroup(**payload)
    session.add(rg)
    await session.flush()  # to get PK

    # create members
    for item in raw_list:
        if isinstance(item, str):
            dest_number = item.strip()
            if not dest_number:
                continue
            m = RingGroupMember(
                domain_id=domain_id,
                ring_group_uuid=rg.ring_group_uuid,
                destination_number=dest_number,
            )
            session.add(m)
        elif isinstance(item, dict):
            # handle dict with various key styles
            num = item.get("destination_number") or item.get("destinationNumber") or item.get("number") or item.get("extension")
            if not num:
                continue
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
                m = RingGroupMember(
                    domain_id=domain_id,
                    ring_group_uuid=rg.ring_group_uuid,
                    destination_number=str(d.get("destination_number", "")).strip(),
                    destination_delay=int(d.get("destination_delay", 0) or 0),
                    destination_timeout=int(d.get("destination_timeout", 30) or 30),
                    destination_enabled=bool(d.get("destination_enabled", True)),
                    destination_description=d.get("destination_description"),
                )
                if m.destination_number:
                    session.add(m)
            except Exception:
                continue

    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(rg)
    return rg


async def list_ring_groups(
    session: AsyncSession,
    domain_id: UUID,
    page: int = 1,
    per_page: int = 20,
    q: Optional[str] = None,
    sort: str = "name",
    order: str = "asc",
    filter_extension: Optional[str] = None,
    filter_name: Optional[str] = None,
    filter_strategy: Optional[str] = None,
    filter_enabled: Optional[bool] = None,
) -> tuple[List[RingGroup], int]:
    per_page = min(max(per_page, 1), 100)
    offset = (page - 1) * per_page
    base = select(RingGroup).where(RingGroup.domain_id == domain_id)

    if q:
        # search name, extension, strategy, description
        ilike = f"%{q}%"
        base = base.where(
            or_(
                RingGroup.name.ilike(ilike),
                RingGroup.extension.ilike(ilike),
                RingGroup.strategy.ilike(ilike),
                RingGroup.description.ilike(ilike),
            )
        )
    if filter_extension:
        base = base.where(RingGroup.extension.ilike(f"%{filter_extension}%"))
    if filter_name:
        base = base.where(RingGroup.name.ilike(f"%{filter_name}%"))
    if filter_strategy:
        base = base.where(RingGroup.strategy == filter_strategy)
    if filter_enabled is not None:
        base = base.where(RingGroup.enabled == filter_enabled)

    count_q = select(func.count()).select_from(base.subquery())
    total = (await session.execute(count_q)).scalar() or 0

    # sorting — allowlist columns
    allowed_sort = {"name": RingGroup.name, "extension": RingGroup.extension, "strategy": RingGroup.strategy, "created_at": RingGroup.created_at, "timeout": RingGroup.timeout}
    col = allowed_sort.get(sort, RingGroup.name)
    if order == "desc":
        base = base.order_by(col.desc())
    else:
        base = base.order_by(col.asc())

    base = base.offset(offset).limit(per_page)
    result = await session.execute(base)
    return result.scalars().all(), total


async def get_ring_group_with_members(session: AsyncSession, ring_group_uuid: UUID, domain_id: UUID):
    r = await session.execute(select(RingGroup).where(RingGroup.ring_group_uuid == ring_group_uuid, RingGroup.domain_id == domain_id))
    rg = r.scalar_one_or_none()
    if not rg:
        return None, []
    m = await session.execute(select(RingGroupMember).where(RingGroupMember.ring_group_uuid == ring_group_uuid, RingGroupMember.domain_id == domain_id).order_by(RingGroupMember.destination_delay, RingGroupMember.destination_number))
    members = m.scalars().all()
    return rg, members
