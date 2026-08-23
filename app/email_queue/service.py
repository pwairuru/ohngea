from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.email_queue.models import EmailQueue
from app.email_queue.schemas import EmailQueueCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_eq(session: AsyncSession, data: EmailQueueCreate, domain_id: UUID):
    if not data.email_from or not data.email_to:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="email_from and email_to required")
    obj=EmailQueue(domain_id=domain_id, email_from=data.email_from, email_to=data.email_to, email_subject=data.email_subject, email_body=data.email_body, email_status=data.email_status or "waiting")
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_eq(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(EmailQueue).where(EmailQueue.domain_id==domain_id)
    count_q=select(func.count()).select_from(EmailQueue).where(EmailQueue.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(EmailQueue.email_to.ilike(like), EmailQueue.email_subject.ilike(like)))
        count_q=count_q.where(or_(EmailQueue.email_to.ilike(like), EmailQueue.email_subject.ilike(like)))
    query=query.order_by(EmailQueue.created_at.desc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
