from typing import Optional
from uuid import UUID

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.music_on_hold.models import MusicOnHold
from app.music_on_hold.schemas import MusicOnHoldCreate


async def create_moh(session: AsyncSession, data: MusicOnHoldCreate, domain_id: UUID) -> MusicOnHold:
    name = data.music_name or data.name
    path = data.music_path or data.path
    if not name or not path:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="music_name and music_path required")
    existing = await session.execute(select(MusicOnHold).where(MusicOnHold.domain_id == domain_id, MusicOnHold.music_name == name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Music on hold name already exists")
    moh = MusicOnHold(
        domain_id=domain_id,
        music_name=name,
        music_path=path,
        music_rate=data.music_rate or data.rate or 8000,
        music_shuffle=data.music_shuffle or "false",
        enabled=data.enabled if data.enabled is not None else True,
        description=data.description,
    )
    session.add(moh)
    await session.commit()
    await session.refresh(moh)
    return moh


async def list_moh(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str], sort: str, order: str):
    query = select(MusicOnHold).where(MusicOnHold.domain_id == domain_id)
    count_q = select(func.count()).select_from(MusicOnHold).where(MusicOnHold.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(MusicOnHold.music_name.ilike(like), MusicOnHold.description.ilike(like)))
        count_q = count_q.where(or_(MusicOnHold.music_name.ilike(like), MusicOnHold.description.ilike(like)))
    allowed = {"music_name": MusicOnHold.music_name, "name": MusicOnHold.music_name, "created_at": MusicOnHold.created_at}
    col = allowed.get(sort, MusicOnHold.music_name)
    query = query.order_by(col.desc() if order == "desc" else col.asc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total
