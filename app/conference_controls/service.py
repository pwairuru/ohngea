from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.conference_controls.models import ConferenceControl, ConferenceControlDetail
from app.conference_controls.schemas import ConferenceControlCreate
async def create_control(session: AsyncSession, data: ConferenceControlCreate, domain_id: UUID):
    name=data.control_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="control_name required")
    existing=await session.execute(select(ConferenceControl).where(ConferenceControl.domain_id==domain_id, ConferenceControl.control_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Conference control already exists")
    obj=ConferenceControl(domain_id=domain_id, control_name=name, control_enabled=data.control_enabled if data.control_enabled is not None else (data.enabled if data.enabled is not None else True), control_description=data.control_description or data.description)
    session.add(obj)
    await session.flush()
    for d in (data.details or []):
        if isinstance(d, dict):
            det=ConferenceControlDetail(conference_control_uuid=obj.conference_control_uuid, domain_id=domain_id, control_digits=str(d.get("control_digits") or d.get("digits") or ""), control_action=d.get("control_action") or d.get("action") or "mute", control_data=d.get("control_data") or d.get("data"))
            session.add(det)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_controls(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(ConferenceControl).where(ConferenceControl.domain_id==domain_id)
    count_q=select(func.count()).select_from(ConferenceControl).where(ConferenceControl.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(ConferenceControl.control_name.ilike(like), ConferenceControl.control_description.ilike(like)))
        count_q=count_q.where(or_(ConferenceControl.control_name.ilike(like), ConferenceControl.control_description.ilike(like)))
    query=query.order_by(ConferenceControl.control_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
async def get_with_details(session: AsyncSession, uid: UUID, domain_id: UUID):
    r=await session.execute(select(ConferenceControl).where(ConferenceControl.conference_control_uuid==uid, ConferenceControl.domain_id==domain_id))
    ctrl=r.scalar_one_or_none()
    if not ctrl: return None, []
    details=(await session.execute(select(ConferenceControlDetail).where(ConferenceControlDetail.conference_control_uuid==uid))).scalars().all()
    return ctrl, details
