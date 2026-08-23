from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.call_broadcast.models import CallBroadcast
from app.call_broadcast.schemas import CallBroadcastCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_cb(session: AsyncSession, data: CallBroadcastCreate, domain_id: UUID):
    name=data.broadcast_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="broadcast_name required")
    existing=await session.execute(select(CallBroadcast).where(CallBroadcast.domain_id==domain_id, CallBroadcast.broadcast_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Call broadcast already exists")
    obj=CallBroadcast(domain_id=domain_id, broadcast_name=name, broadcast_description=data.broadcast_description or data.description, broadcast_timeout=data.broadcast_timeout or 30, broadcast_concurrent_limit=data.broadcast_concurrent_limit or 1, broadcast_caller_id_name=data.broadcast_caller_id_name, broadcast_caller_id_number=data.broadcast_caller_id_number, broadcast_destination_type=data.broadcast_destination_type, broadcast_phone_numbers=data.broadcast_phone_numbers, broadcast_enabled=data.broadcast_enabled if data.broadcast_enabled is not None else (data.enabled if data.enabled is not None else True))
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_cb(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(CallBroadcast).where(CallBroadcast.domain_id==domain_id)
    count_q=select(func.count()).select_from(CallBroadcast).where(CallBroadcast.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(CallBroadcast.broadcast_name.ilike(like), CallBroadcast.broadcast_description.ilike(like)))
        count_q=count_q.where(or_(CallBroadcast.broadcast_name.ilike(like), CallBroadcast.broadcast_description.ilike(like)))
    query=query.order_by(CallBroadcast.broadcast_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
