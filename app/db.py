from collections.abc import AsyncGenerator
from uuid import UUID

import uuid6
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.config import settings

engine = create_async_engine(settings.database_url, pool_pre_ping=True)

SessionLocal = async_sessionmaker(engine, expire_on_commit=False, class_=AsyncSession)


def uuid7() -> UUID:
    """Generate UUIDv7 — time-ordered, good for PK partitioning."""
    return uuid6.uuid7()


async def get_session() -> AsyncGenerator[AsyncSession, None]:
    """Yield AsyncSession — caller must set RLS via SET LOCAL if tenant-scoped."""
    async with SessionLocal() as session:
        yield session


async def set_tenant(session: AsyncSession, domain_id: UUID | str | None) -> None:
    """Set RLS GUC for current transaction. No-op if domain_id is None."""
    if domain_id is None:
        return
    await session.execute(
        text("SELECT set_config('app.domain_uuid', :v, true)"),
        {"v": str(domain_id)},
    )
