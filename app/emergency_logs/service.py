from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.emergency_logs.models import EmergencyLog
from app.emergency_logs.schemas import EmergencyLogCreate
async def create_log(session: AsyncSession, data: EmergencyLogCreate, domain_id: UUID):
    obj=EmergencyLog(domain_id=domain_id, extension=data.extension, event=data.event)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_logs(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(EmergencyLog).where(EmergencyLog.domain_id==domain_id)
    count_q=select(func.count()).select_from(EmergencyLog).where(EmergencyLog.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(EmergencyLog.extension.ilike(like), EmergencyLog.event.ilike(like)))
        count_q=count_q.where(or_(EmergencyLog.extension.ilike(like), EmergencyLog.event.ilike(like)))
    query=query.order_by(EmergencyLog.created_at.desc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
