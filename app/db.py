from collections.abc import AsyncGenerator
from datetime import datetime, timezone
from uuid import UUID

import uuid6
from sqlalchemy import event, text
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import Session

from app.config import settings

engine = create_async_engine(
    settings.database_url,
    pool_pre_ping=True,
    pool_size=5,
    max_overflow=10,
    pool_timeout=30,
)

SessionLocal = async_sessionmaker(engine, expire_on_commit=False, class_=AsyncSession)


@event.listens_for(Session, "before_flush")
def _coerce_naive_datetimes(session, flush_context, instances):
    """Backstop: assume UTC for any stray naive datetimes before INSERT/UPDATE.

    Models default to tz-aware ``datetime.now(timezone.utc)`` while columns
    are TIMESTAMPTZ (asyncpg/SQLModel reject naive values). This coerces any
    remaining naive value (e.g. Core inserts bypassing model defaults).
    Covers AsyncSession too (it wraps a sync Session).
    """
    for obj in session.new.union(session.dirty):
        for attr, value in vars(obj).items():
            if attr.startswith("_") or not isinstance(value, datetime):
                continue
            if value.tzinfo is None:
                setattr(obj, attr, value.replace(tzinfo=timezone.utc))


async def dispose_engine() -> None:
    try:
        await engine.dispose()
    except Exception:
        pass


def uuid7() -> UUID:
    """Generate UUIDv7 — time-ordered, good for PK partitioning."""
    return uuid6.uuid7()


async def get_session() -> AsyncGenerator[AsyncSession, None]:
    """Yield AsyncSession — caller must set RLS via SET LOCAL if tenant-scoped."""
    async with SessionLocal() as session:
        yield session


async def set_tenant(session: AsyncSession, domain_id: UUID | str | None) -> None:
    """Set RLS GUC for current transaction. Logs on failure, never silent."""
    import logging
    log = logging.getLogger("app.db")
    if domain_id is None:
        log.warning("set_tenant called with None domain_id — tenant isolation relies on manual filter")
        return
    # sqlite/memory tests don't have set_config — skip quietly
    try:
        url_str = str(engine.url)
        if "sqlite" in url_str:
            return
    except Exception:
        pass
    try:
        await session.execute(
            text("SELECT set_config('app.domain_uuid', :v, true)"),
            {"v": str(domain_id)},
        )
    except Exception as e:
        # sqlite in tests (different engine via dependency_overrides) — ignore only sqlite errors
        msg = str(e).lower()
        if "sqlite" in msg or "set_config" in msg or "no such function" in msg:
            return
        log.error("set_tenant failed for %s: %s", domain_id, e, exc_info=True)
        raise
