from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.streams.models import Stream
from app.streams.schemas import StreamCreate

async def create_stream(session: AsyncSession, data: StreamCreate, domain_id: UUID) -> Stream:
    name=data.stream_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="stream_name required")
    existing=await session.execute(select(Stream).where(Stream.domain_id==domain_id, Stream.stream_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Stream already exists")
    obj=Stream(domain_id=domain_id, stream_name=name, stream_location=data.stream_location or data.location, stream_enabled=data.stream_enabled if data.stream_enabled is not None else (data.enabled if data.enabled is not None else True), stream_description=data.stream_description or data.description)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj

async def list_streams(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(Stream).where(Stream.domain_id==domain_id)
    count_q=select(func.count()).select_from(Stream).where(Stream.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(Stream.stream_name.ilike(like), Stream.stream_description.ilike(like)))
        count_q=count_q.where(or_(Stream.stream_name.ilike(like), Stream.stream_description.ilike(like)))
    query=query.order_by(Stream.stream_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
