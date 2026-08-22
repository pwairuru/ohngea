from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.fax_queue.models import FaxQueue
from app.fax_queue.schemas import FaxQueueCreate
async def create_fq(session: AsyncSession, data: FaxQueueCreate, domain_id: UUID):
    obj=FaxQueue(domain_id=domain_id, fax_uuid=data.fax_uuid, fax_number=data.fax_number, fax_file=data.fax_file, fax_status=data.fax_status or "waiting", fax_queue_name=data.fax_queue_name)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_fq(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(FaxQueue).where(FaxQueue.domain_id==domain_id)
    count_q=select(func.count()).select_from(FaxQueue).where(FaxQueue.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(FaxQueue.fax_number.ilike(like), FaxQueue.fax_status.ilike(like)))
        count_q=count_q.where(or_(FaxQueue.fax_number.ilike(like), FaxQueue.fax_status.ilike(like)))
    query=query.order_by(FaxQueue.created_at.desc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
