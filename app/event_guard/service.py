from typing import Optional
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.event_guard.models import EventGuardLog
from app.event_guard.schemas import EventGuardLogCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_eg(session: AsyncSession, data: EventGuardLogCreate):
    obj=EventGuardLog(hostname=data.hostname, ip_address=data.ip_address, extension=data.extension, filter=data.filter, log_status=data.log_status)
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache()
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_eg(session: AsyncSession, page:int, per_page:int, q:Optional[str]):
    query=select(EventGuardLog)
    count_q=select(func.count()).select_from(EventGuardLog)
    if q:
        like=f"%{q}%"
        query=query.where(or_(EventGuardLog.ip_address.ilike(like), EventGuardLog.extension.ilike(like)))
        count_q=count_q.where(or_(EventGuardLog.ip_address.ilike(like), EventGuardLog.extension.ilike(like)))
    query=query.order_by(EventGuardLog.created_at.desc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
