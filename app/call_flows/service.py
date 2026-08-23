from typing import Optional
from uuid import UUID

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.call_flows.models import CallFlow
from app.call_flows.schemas import CallFlowCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session


async def create_call_flow(session: AsyncSession, data: CallFlowCreate) -> CallFlow:
    name = data.call_flow_name or data.name
    extension = data.call_flow_extension or data.extension
    domain_id = data.domain_id
    if not domain_id or not name or not extension:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="domain_id, call_flow_name and call_flow_extension required")
    existing = await session.execute(select(CallFlow).where(CallFlow.domain_id == domain_id, CallFlow.call_flow_extension == extension))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Call flow extension already exists")
    cf = CallFlow(
        domain_id=domain_id,
        call_flow_name=name,
        call_flow_extension=extension,
        call_flow_feature_code=data.call_flow_feature_code or data.feature_code,
        call_flow_context=data.call_flow_context or data.context or "public",
        call_flow_enabled=data.call_flow_enabled if data.call_flow_enabled is not None else (data.enabled if data.enabled is not None else True),
        call_flow_description=data.call_flow_description or data.description,
    )
    session.add(cf)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(cf)
    return cf


async def list_call_flows(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str], sort: str, order: str, filter_extension=None, filter_enabled=None):
    query = select(CallFlow).where(CallFlow.domain_id == domain_id)
    count_q = select(func.count()).select_from(CallFlow).where(CallFlow.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(CallFlow.call_flow_name.ilike(like), CallFlow.call_flow_extension.ilike(like), CallFlow.call_flow_description.ilike(like)))
        count_q = count_q.where(or_(CallFlow.call_flow_name.ilike(like), CallFlow.call_flow_extension.ilike(like), CallFlow.call_flow_description.ilike(like)))
    if filter_extension:
        query = query.where(CallFlow.call_flow_extension == filter_extension)
        count_q = count_q.where(CallFlow.call_flow_extension == filter_extension)
    if filter_enabled is not None:
        query = query.where(CallFlow.call_flow_enabled == filter_enabled)
        count_q = count_q.where(CallFlow.call_flow_enabled == filter_enabled)
    allowed = {"call_flow_name": CallFlow.call_flow_name, "call_flow_extension": CallFlow.call_flow_extension, "created_at": CallFlow.created_at, "name": CallFlow.call_flow_name, "extension": CallFlow.call_flow_extension}
    col = allowed.get(sort, CallFlow.call_flow_name)
    if order == "desc":
        query = query.order_by(col.desc())
    else:
        query = query.order_by(col.asc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total
