from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.pin_numbers.models import PinNumber
from app.pin_numbers.schemas import PinNumberCreate

async def create_pin(session: AsyncSession, data: PinNumberCreate, domain_id: UUID) -> PinNumber:
    pin = data.pin_number or data.number
    if not pin:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="pin_number required")
    existing = await session.execute(select(PinNumber).where(PinNumber.domain_id == domain_id, PinNumber.pin_number == str(pin)))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="PIN already exists")
    obj = PinNumber(domain_id=domain_id, pin_number=str(pin), accountcode=data.accountcode, enabled=data.enabled if data.enabled is not None else True, description=data.description)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj

async def list_pins(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str]):
    query = select(PinNumber).where(PinNumber.domain_id == domain_id)
    count_q = select(func.count()).select_from(PinNumber).where(PinNumber.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(PinNumber.pin_number.ilike(like), PinNumber.description.ilike(like)))
        count_q = count_q.where(or_(PinNumber.pin_number.ilike(like), PinNumber.description.ilike(like)))
    query = query.order_by(PinNumber.pin_number.asc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total
