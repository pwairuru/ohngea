from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.fifo.models import Fifo
from app.fifo.schemas import FifoCreate
async def create_fifo(session: AsyncSession, data: FifoCreate, domain_id: UUID):
    name=data.fifo_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="fifo_name required")
    existing=await session.execute(select(Fifo).where(Fifo.domain_id==domain_id, Fifo.fifo_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="FIFO already exists")
    obj=Fifo(domain_id=domain_id, fifo_name=name, fifo_extension=data.fifo_extension or data.extension, fifo_enabled=data.fifo_enabled if data.fifo_enabled is not None else (data.enabled if data.enabled is not None else True), fifo_description=data.fifo_description or data.description)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_fifo(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(Fifo).where(Fifo.domain_id==domain_id)
    count_q=select(func.count()).select_from(Fifo).where(Fifo.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(Fifo.fifo_name.ilike(like), Fifo.fifo_description.ilike(like)))
        count_q=count_q.where(or_(Fifo.fifo_name.ilike(like), Fifo.fifo_description.ilike(like)))
    query=query.order_by(Fifo.fifo_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
