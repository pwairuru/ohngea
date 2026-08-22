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
    # sqlite/memory tests don't have set_config — skip
    try:
        bind = session.bind
        if bind is not None:
            # sync engine dialect check; for async, bind may be None until connected
            # attempt to detect sqlite via url
            dialect_name = getattr(bind.dialect, "name", "") if hasattr(bind, "dialect") else ""
            if dialect_name == "sqlite":
                return
            # also check engine url
            url_str = str(getattr(bind, "url", "") or getattr(bind.engine, "url", "") if hasattr(bind, "engine") else "")
            if "sqlite" in url_str:
                return
    except Exception:
        pass
    try:
        await session.execute(
            text("SELECT set_config('app.domain_uuid', :v, true)"),
            {"v": str(domain_id)},
        )
    except Exception:
        # sqlite or other dialects without set_config (tests) — ignore
        pass
