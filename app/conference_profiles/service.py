from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.conference_profiles.models import ConferenceProfile, ConferenceProfileParam
from app.conference_profiles.schemas import ConferenceProfileCreate
async def create_profile(session: AsyncSession, data: ConferenceProfileCreate, domain_id: UUID):
    name=data.profile_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="profile_name required")
    existing=await session.execute(select(ConferenceProfile).where(ConferenceProfile.domain_id==domain_id, ConferenceProfile.profile_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Conference profile already exists")
    obj=ConferenceProfile(domain_id=domain_id, profile_name=name, profile_enabled=data.profile_enabled if data.profile_enabled is not None else (data.enabled if data.enabled is not None else True), profile_description=data.profile_description or data.description)
    session.add(obj)
    await session.flush()
    for p in (data.params or []):
        if isinstance(p, dict):
            par=ConferenceProfileParam(conference_profile_uuid=obj.conference_profile_uuid, domain_id=domain_id, profile_param_name=p.get("profile_param_name") or p.get("name") or "param", profile_param_value=str(p.get("profile_param_value") or p.get("value") or ""), profile_param_enabled=p.get("profile_param_enabled", True))
            session.add(par)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_profiles(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(ConferenceProfile).where(ConferenceProfile.domain_id==domain_id)
    count_q=select(func.count()).select_from(ConferenceProfile).where(ConferenceProfile.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(ConferenceProfile.profile_name.ilike(like), ConferenceProfile.profile_description.ilike(like)))
        count_q=count_q.where(or_(ConferenceProfile.profile_name.ilike(like), ConferenceProfile.profile_description.ilike(like)))
    query=query.order_by(ConferenceProfile.profile_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
async def get_with_params(session: AsyncSession, uid: UUID, domain_id: UUID):
    r=await session.execute(select(ConferenceProfile).where(ConferenceProfile.conference_profile_uuid==uid, ConferenceProfile.domain_id==domain_id))
    prof=r.scalar_one_or_none()
    if not prof: return None, []
    params=(await session.execute(select(ConferenceProfileParam).where(ConferenceProfileParam.conference_profile_uuid==uid))).scalars().all()
    return prof, params
