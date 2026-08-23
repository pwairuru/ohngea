from typing import Optional
from uuid import UUID

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.call_block.models import CallBlock
from app.call_block.schemas import CallBlockCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session


async def create_call_block(session: AsyncSession, data: CallBlockCreate, domain_id: UUID) -> CallBlock:
    name = data.call_block_name or data.name or f"block-{data.call_block_number or data.number}"
    number = data.call_block_number or data.number
    if not number:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="call_block_number required")
    existing = await session.execute(select(CallBlock).where(CallBlock.domain_id == domain_id, CallBlock.call_block_number == number))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Number already blocked")
    cb = CallBlock(
        domain_id=domain_id,
        call_block_name=name,
        call_block_number=number,
        call_block_enabled=data.call_block_enabled if data.call_block_enabled is not None else (data.enabled if data.enabled is not None else True),
        call_block_description=data.call_block_description or data.description,
        call_block_action=data.call_block_action or "reject",
    )
    session.add(cb)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(cb)
    return cb


async def list_call_block(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str]):
    query = select(CallBlock).where(CallBlock.domain_id == domain_id)
    count_q = select(func.count()).select_from(CallBlock).where(CallBlock.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(CallBlock.call_block_name.ilike(like), CallBlock.call_block_number.ilike(like)))
        count_q = count_q.where(or_(CallBlock.call_block_name.ilike(like), CallBlock.call_block_number.ilike(like)))
    query = query.order_by(CallBlock.created_at.desc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total
