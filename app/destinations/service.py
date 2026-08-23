from typing import Optional
from uuid import UUID

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.destinations.models import Destination
from app.destinations.schemas import DestinationCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session


async def create_destination(session: AsyncSession, data: DestinationCreate, domain_id: UUID) -> Destination:
    number = data.destination_number or data.number
    if not number:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="destination_number required")
    existing = await session.execute(select(Destination).where(Destination.domain_id == domain_id, Destination.destination_number == number))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Destination already exists")
    dest = Destination(
        domain_id=domain_id,
        destination_type=data.destination_type or "inbound",
        destination_number=str(number),
        destination_enabled=data.destination_enabled if data.destination_enabled is not None else (data.enabled if data.enabled is not None else True),
        destination_description=data.destination_description or data.description,
        destination_context=data.destination_context or "public",
        destination_app=data.destination_app,
        destination_data=data.destination_data,
        destination_caller_id_name=data.destination_caller_id_name,
        destination_caller_id_number=data.destination_caller_id_number,
    )
    session.add(dest)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(dest)
    return dest


async def list_destinations(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str]):
    query = select(Destination).where(Destination.domain_id == domain_id)
    count_q = select(func.count()).select_from(Destination).where(Destination.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(Destination.destination_number.ilike(like), Destination.destination_description.ilike(like)))
        count_q = count_q.where(or_(Destination.destination_number.ilike(like), Destination.destination_description.ilike(like)))
    query = query.order_by(Destination.destination_number.asc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total
