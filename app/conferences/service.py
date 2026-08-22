from typing import Optional, List
from uuid import UUID

from fastapi import HTTPException
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.conferences.models import Conference


async def create_conference(
    session: AsyncSession,
    data,
    domain_id: UUID,
) -> Conference:
    if hasattr(data, "model_dump"):
        payload = data.model_dump(exclude_unset=True)
    elif isinstance(data, dict):
        payload = dict(data)
    else:
        payload = {}

    name = payload.get("conference_name")
    extension = payload.get("conference_extension")
    if not name:
        raise HTTPException(status_code=422, detail="conference_name is required")
    if not extension:
        raise HTTPException(status_code=422, detail="conference_extension is required")

    existing = await session.execute(
        select(Conference).where(Conference.domain_id == domain_id, Conference.conference_extension == str(extension))
    )
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Conference extension already exists")

    payload["domain_id"] = domain_id
    # normalize empty strings to None for optional fields
    for k in ("conference_pin_number", "conference_flags", "conference_description", "conference_context", "conference_email_address", "conference_account_code"):
        if payload.get(k) == "":
            payload[k] = None
    if payload.get("dialplan_uuid") == "":
        payload["dialplan_uuid"] = None

    # filter only valid model fields
    valid = {k: v for k, v in payload.items() if k in Conference.model_fields}
    conf = Conference(**valid)
    session.add(conf)
    await session.flush()
    await session.commit()
    await session.refresh(conf)
    return conf


async def list_conferences(
    session: AsyncSession,
    domain_id: UUID,
    page: int = 1,
    per_page: int = 20,
    q: Optional[str] = None,
    sort: str = "conference_name",
    order: str = "asc",
    filter_name: Optional[str] = None,
    filter_extension: Optional[str] = None,
    filter_enabled: Optional[bool] = None,
    filter_profile: Optional[str] = None,
) -> tuple[List[Conference], int]:
    per_page = min(max(per_page, 1), 100)
    offset = (page - 1) * per_page
    base = select(Conference).where(Conference.domain_id == domain_id)

    if q:
        ilike = f"%{q}%"
        base = base.where(
            or_(
                Conference.conference_name.ilike(ilike),
                Conference.conference_extension.ilike(ilike),
                Conference.conference_description.ilike(ilike),
                Conference.conference_profile.ilike(ilike),
                Conference.conference_pin_number.ilike(ilike),
                Conference.conference_context.ilike(ilike),
            )
        )
    if filter_name:
        base = base.where(Conference.conference_name.ilike(f"%{filter_name}%"))
    if filter_extension:
        base = base.where(Conference.conference_extension.ilike(f"%{filter_extension}%"))
    if filter_enabled is not None:
        base = base.where(Conference.conference_enabled == filter_enabled)
    if filter_profile:
        base = base.where(Conference.conference_profile == filter_profile)

    count_q = select(func.count()).select_from(base.subquery())
    total = (await session.execute(count_q)).scalar() or 0

    allowed_sort = {
        "conference_name": Conference.conference_name,
        "name": Conference.conference_name,
        "conference_extension": Conference.conference_extension,
        "extension": Conference.conference_extension,
        "conference_profile": Conference.conference_profile,
        "profile": Conference.conference_profile,
        "conference_enabled": Conference.conference_enabled,
        "enabled": Conference.conference_enabled,
        "conference_order": Conference.conference_order,
        "order": Conference.conference_order,
        "created_at": Conference.created_at,
        "updated_at": Conference.updated_at,
    }
    col = allowed_sort.get(sort, Conference.conference_name)
    if order == "desc":
        base = base.order_by(col.desc())
    else:
        base = base.order_by(col.asc())

    base = base.offset(offset).limit(per_page)
    result = await session.execute(base)
    return result.scalars().all(), total


async def get_conference(session: AsyncSession, conference_uuid: UUID, domain_id: UUID) -> Optional[Conference]:
    r = await session.execute(select(Conference).where(Conference.conference_uuid == conference_uuid, Conference.domain_id == domain_id))
    return r.scalar_one_or_none()
