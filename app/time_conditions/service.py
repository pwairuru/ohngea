from typing import Optional, List
from uuid import UUID

from fastapi import HTTPException
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.time_conditions.models import TimeCondition
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session


async def create_time_condition(
    session: AsyncSession,
    data,
    domain_id: UUID,
) -> TimeCondition:
    # data is TimeConditionCreate instance or dict-like
    # extract fields
    if hasattr(data, "model_dump"):
        payload = data.model_dump(exclude_unset=True)
    elif isinstance(data, dict):
        payload = dict(data)
    else:
        payload = {}

    # require name and extension
    name = payload.get("name")
    extension = payload.get("extension")
    if not name:
        raise HTTPException(status_code=422, detail="name is required")
    if not extension:
        raise HTTPException(status_code=422, detail="extension is required")

    # check duplicate extension per domain
    existing = await session.execute(
        select(TimeCondition).where(TimeCondition.domain_id == domain_id, TimeCondition.extension == str(extension))
    )
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Time condition extension already exists")

    # ensure domain_id matches tenant (override)
    payload["domain_id"] = domain_id
    # remove None dialplan_uuid if empty string?
    if payload.get("dialplan_uuid") == "":
        payload["dialplan_uuid"] = None
    # sanitize details if empty
    if "details" in payload and payload["details"] == "":
        payload["details"] = None

    tc = TimeCondition(**payload)
    session.add(tc)
    await session.flush()
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(tc)
    return tc


async def list_time_conditions(
    session: AsyncSession,
    domain_id: UUID,
    page: int = 1,
    per_page: int = 20,
    q: Optional[str] = None,
    sort: str = "name",
    order: str = "asc",
    filter_name: Optional[str] = None,
    filter_extension: Optional[str] = None,
    filter_enabled: Optional[bool] = None,
    filter_context: Optional[str] = None,
) -> tuple[List[TimeCondition], int]:
    per_page = min(max(per_page, 1), 100)
    offset = (page - 1) * per_page
    base = select(TimeCondition).where(TimeCondition.domain_id == domain_id)

    if q:
        ilike = f"%{q}%"
        base = base.where(
            or_(
                TimeCondition.name.ilike(ilike),
                TimeCondition.extension.ilike(ilike),
                TimeCondition.description.ilike(ilike),
                TimeCondition.context.ilike(ilike),
                TimeCondition.condition.ilike(ilike),
                TimeCondition.true_destination.ilike(ilike),
                TimeCondition.false_destination.ilike(ilike),
            )
        )
    if filter_name:
        base = base.where(TimeCondition.name.ilike(f"%{filter_name}%"))
    if filter_extension:
        base = base.where(TimeCondition.extension.ilike(f"%{filter_extension}%"))
    if filter_enabled is not None:
        base = base.where(TimeCondition.enabled == filter_enabled)
    if filter_context:
        base = base.where(TimeCondition.context.ilike(f"%{filter_context}%"))

    count_q = select(func.count()).select_from(base.subquery())
    total = (await session.execute(count_q)).scalar() or 0

    allowed_sort = {
        "name": TimeCondition.name,
        "extension": TimeCondition.extension,
        "created_at": TimeCondition.created_at,
        "updated_at": TimeCondition.updated_at,
        "enabled": TimeCondition.enabled,
        "context": TimeCondition.context,
    }
    col = allowed_sort.get(sort, TimeCondition.name)
    if order == "desc":
        base = base.order_by(col.desc())
    else:
        base = base.order_by(col.asc())

    base = base.offset(offset).limit(per_page)
    result = await session.execute(base)
    return result.scalars().all(), total


async def get_time_condition(session: AsyncSession, time_condition_uuid: UUID, domain_id: UUID) -> Optional[TimeCondition]:
    r = await session.execute(select(TimeCondition).where(TimeCondition.time_condition_uuid == time_condition_uuid, TimeCondition.domain_id == domain_id))
    return r.scalar_one_or_none()
