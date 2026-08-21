from uuid import UUID
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from fastapi import HTTPException

from app.extensions.models import Extension
from app.extensions.schemas import ExtensionCreate, ExtensionUpdate


async def create_extension(session: AsyncSession, data: ExtensionCreate) -> Extension:
    # check duplicate extension per domain
    existing = await session.execute(select(Extension).where(Extension.domain_id == data.domain_id, Extension.extension == data.extension))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Extension already exists")
    ext = Extension(**data.model_dump())
    session.add(ext)
    await session.commit()
    await session.refresh(ext)
    # invalidate cache if redis available (best effort)
    try:
        import redis.asyncio as redis
        from app.config import settings
        r = redis.from_url(settings.redis_url)
        await r.delete(f"xml:{data.domain_id}:directory")
        await r.aclose()
    except Exception:
        pass
    return ext


async def list_extensions(
    session: AsyncSession, domain_id: UUID, page: int = 1, per_page: int = 20, q: str | None = None, sort: str = "extension", order: str = "asc"
) -> tuple[list[Extension], int]:
    per_page = min(max(per_page, 1), 100)
    offset = (page - 1) * per_page
    base = select(Extension).where(Extension.domain_id == domain_id)
    if q:
        base = base.where((Extension.extension.ilike(f"%{q}%")) | (Extension.effective_caller_id_name.ilike(f"%{q}%")))
    count_q = select(func.count()).select_from(base.subquery())
    total = (await session.execute(count_q)).scalar() or 0
    # sorting
    col = getattr(Extension, sort, Extension.extension)
    if order == "desc":
        base = base.order_by(col.desc())
    else:
        base = base.order_by(col.asc())
    base = base.offset(offset).limit(per_page)
    result = await session.execute(base)
    return result.scalars().all(), total
