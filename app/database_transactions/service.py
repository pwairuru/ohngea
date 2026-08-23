from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.database_transactions.models import DatabaseTransaction
from app.database_transactions.schemas import DatabaseTransactionCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_dt(session: AsyncSession, data: DatabaseTransactionCreate, domain_id: UUID):
    obj=DatabaseTransaction(domain_id=domain_id, app_name=data.app_name, transaction_code=data.transaction_code, transaction_type=data.transaction_type or "add", transaction_address=data.transaction_address)
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_dt(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(DatabaseTransaction).where(DatabaseTransaction.domain_id==domain_id)
    count_q=select(func.count()).select_from(DatabaseTransaction).where(DatabaseTransaction.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(DatabaseTransaction.app_name.ilike(like), DatabaseTransaction.transaction_code.ilike(like)))
        count_q=count_q.where(or_(DatabaseTransaction.app_name.ilike(like), DatabaseTransaction.transaction_code.ilike(like)))
    query=query.order_by(DatabaseTransaction.transaction_date.desc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
