from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.call_centers.models import CallCenter, CallCenterAgent
from app.call_centers.schemas import CallCenterCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_cc(session: AsyncSession, data: CallCenterCreate, domain_id: UUID):
    name=data.queue_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="queue_name required")
    existing=await session.execute(select(CallCenter).where(CallCenter.domain_id==domain_id, CallCenter.queue_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Call center already exists")
    obj=CallCenter(domain_id=domain_id, queue_name=name, queue_extension=data.queue_extension or data.extension, queue_strategy=data.queue_strategy or "longest-idle-agent", queue_enabled=data.queue_enabled if data.queue_enabled is not None else (data.enabled if data.enabled is not None else True), queue_description=data.queue_description or data.description)
    session.add(obj)
    await session.flush()
    for a in (data.agents or []):
        if isinstance(a, dict):
            ag=CallCenterAgent(domain_id=domain_id, call_center_queue_uuid=obj.call_center_queue_uuid, agent_name=a.get("agent_name") or a.get("name") or "agent", agent_type=a.get("agent_type","callback"), agent_contact=a.get("agent_contact") or a.get("contact"))
            session.add(ag)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_cc(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(CallCenter).where(CallCenter.domain_id==domain_id)
    count_q=select(func.count()).select_from(CallCenter).where(CallCenter.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(CallCenter.queue_name.ilike(like), CallCenter.queue_description.ilike(like)))
        count_q=count_q.where(or_(CallCenter.queue_name.ilike(like), CallCenter.queue_description.ilike(like)))
    query=query.order_by(CallCenter.queue_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
async def get_cc_with_agents(session: AsyncSession, qid: UUID, domain_id: UUID):
    r=await session.execute(select(CallCenter).where(CallCenter.call_center_queue_uuid==qid, CallCenter.domain_id==domain_id))
    cc=r.scalar_one_or_none()
    if not cc: return None, []
    agents=(await session.execute(select(CallCenterAgent).where(CallCenterAgent.call_center_queue_uuid==qid))).scalars().all()
    return cc, agents
