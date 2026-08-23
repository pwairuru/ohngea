from typing import Optional
from uuid import UUID

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.call_forward.models import CallForward
from app.call_forward.schemas import CallForwardCreate
from app.freeswitch.cache import invalidate_fs_cache_for_session


async def create_call_forward(session: AsyncSession, data: CallForwardCreate, domain_id: UUID):
    cf = CallForward(
        domain_id=domain_id,
        extension_uuid=data.extension_uuid,
        forward_all_destination=data.forward_all_destination or data.forward_destination,
        forward_all_enabled=bool(data.forward_all_enabled if data.forward_all_enabled is not None else data.forward_enabled or False),
        forward_busy_destination=data.forward_busy_destination,
        forward_busy_enabled=bool(data.forward_busy_enabled or False),
        forward_no_answer_destination=data.forward_no_answer_destination,
        forward_no_answer_enabled=bool(data.forward_no_answer_enabled or False),
        forward_user_not_registered_destination=data.forward_user_not_registered_destination,
        forward_user_not_registered_enabled=bool(data.forward_user_not_registered_enabled or False),
        enabled=bool(data.enabled if data.enabled is not None else True),
        description=data.description,
    )
    session.add(cf)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(cf)
    return cf


async def list_call_forward(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str]):
    query = select(CallForward).where(CallForward.domain_id == domain_id)
    count_q = select(func.count()).select_from(CallForward).where(CallForward.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(CallForward.description.ilike(like), CallForward.forward_all_destination.ilike(like)))
        count_q = count_q.where(or_(CallForward.description.ilike(like), CallForward.forward_all_destination.ilike(like)))
    query = query.order_by(CallForward.created_at.desc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total


async def get_call_forward(session: AsyncSession, cf_uuid: UUID, domain_id: UUID):
    r = await session.execute(select(CallForward).where(CallForward.call_forward_uuid == cf_uuid, CallForward.domain_id == domain_id))
    return r.scalar_one_or_none()
