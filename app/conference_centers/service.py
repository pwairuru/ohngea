from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.conference_centers.models import ConferenceCenter
from app.conference_centers.schemas import ConferenceCenterCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_center(session: AsyncSession, data: ConferenceCenterCreate, domain_id: UUID):
    name=data.conference_center_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="conference_center_name required")
    existing=await session.execute(select(ConferenceCenter).where(ConferenceCenter.domain_id==domain_id, ConferenceCenter.conference_center_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Conference center already exists")
    obj=ConferenceCenter(domain_id=domain_id, conference_center_name=name, conference_center_extension=data.conference_center_extension or (str(data.extension) if data.extension else None), conference_center_pin_length=data.conference_center_pin_length or 6, conference_center_greeting=data.conference_center_greeting, conference_center_description=data.conference_center_description or data.description, conference_center_enabled=data.conference_center_enabled if data.conference_center_enabled is not None else (data.enabled if data.enabled is not None else True))
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_centers(session: AsyncSession, domain_id:UUID, page:int, per_page:int, q:Optional[str]):
    query=select(ConferenceCenter).where(ConferenceCenter.domain_id==domain_id)
    count_q=select(func.count()).select_from(ConferenceCenter).where(ConferenceCenter.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(ConferenceCenter.conference_center_name.ilike(like), ConferenceCenter.conference_center_extension.ilike(like)))
        count_q=count_q.where(or_(ConferenceCenter.conference_center_name.ilike(like), ConferenceCenter.conference_center_extension.ilike(like)))
    query=query.order_by(ConferenceCenter.conference_center_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
