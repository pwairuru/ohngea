from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.bridges.models import Bridge
from app.bridges.schemas import BridgeCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_bridge(session: AsyncSession, data: BridgeCreate, domain_id: UUID) -> Bridge:
    name=data.bridge_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="bridge_name required")
    existing=await session.execute(select(Bridge).where(Bridge.domain_id==domain_id, Bridge.bridge_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Bridge already exists")
    obj=Bridge(domain_id=domain_id, bridge_name=name, bridge_destination=data.bridge_destination or data.destination, bridge_enabled=data.bridge_enabled if data.bridge_enabled is not None else (data.enabled if data.enabled is not None else True), bridge_description=data.bridge_description or data.description)
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_bridges(session: AsyncSession, domain_id:UUID, page:int, per_page:int, q:Optional[str]):
    query=select(Bridge).where(Bridge.domain_id==domain_id)
    count_q=select(func.count()).select_from(Bridge).where(Bridge.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(Bridge.bridge_name.ilike(like), Bridge.bridge_description.ilike(like)))
        count_q=count_q.where(or_(Bridge.bridge_name.ilike(like), Bridge.bridge_description.ilike(like)))
    query=query.order_by(Bridge.bridge_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
