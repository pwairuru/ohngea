from typing import Optional
from uuid import UUID

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.fax.models import Fax
from app.fax.schemas import FaxCreate


async def create_fax(session: AsyncSession, data: FaxCreate) -> Fax:
    # normalize
    fax_extension = data.fax_extension or data.extension
    fax_name = data.fax_name or data.name or f"Fax {fax_extension}"
    domain_id = data.domain_id
    if not domain_id or not fax_extension:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="domain_id and fax_extension required")
    # duplicate check
    existing = await session.execute(select(Fax).where(Fax.domain_id == domain_id, Fax.fax_extension == fax_extension))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Fax extension already exists")
    fax = Fax(
        domain_id=domain_id,
        fax_extension=fax_extension,
        fax_name=fax_name,
        fax_email=data.fax_email or data.email,
        fax_pin_number=data.fax_pin_number,
        fax_caller_id_name=data.fax_caller_id_name,
        fax_caller_id_number=data.fax_caller_id_number,
        fax_forward_number=data.fax_forward_number,
        fax_toll_allow=data.fax_toll_allow,
        enabled=data.enabled if data.enabled is not None else True,
        description=data.description,
    )
    session.add(fax)
    await session.commit()
    await session.refresh(fax)
    return fax


async def list_faxes(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str], sort: str, order: str, filter_extension=None, filter_enabled=None):
    query = select(Fax).where(Fax.domain_id == domain_id)
    count_q = select(func.count()).select_from(Fax).where(Fax.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(Fax.fax_name.ilike(like), Fax.fax_extension.ilike(like), Fax.description.ilike(like)))
        count_q = count_q.where(or_(Fax.fax_name.ilike(like), Fax.fax_extension.ilike(like), Fax.description.ilike(like)))
    if filter_extension:
        query = query.where(Fax.fax_extension == filter_extension)
        count_q = count_q.where(Fax.fax_extension == filter_extension)
    if filter_enabled is not None:
        query = query.where(Fax.enabled == filter_enabled)
        count_q = count_q.where(Fax.enabled == filter_enabled)
    # sort
    allowed = {"fax_extension": Fax.fax_extension, "fax_name": Fax.fax_name, "created_at": Fax.created_at, "name": Fax.fax_name, "extension": Fax.fax_extension}
    col = allowed.get(sort, Fax.fax_extension)
    if order == "desc":
        query = query.order_by(col.desc())
    else:
        query = query.order_by(col.asc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total
