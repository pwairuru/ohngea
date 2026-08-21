from collections.abc import AsyncGenerator

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from app.config import settings

engine = create_async_engine(settings.database_url, pool_pre_ping=True)

SessionLocal = async_sessionmaker(engine, expire_on_commit=False, class_=AsyncSession)


async def get_session() -> AsyncGenerator[AsyncSession, None]:
    """Yield an AsyncSession with RLS hook placeholder.

    Future TenantMiddleware will do ``SET LOCAL app.domain_uuid`` on the
    same transaction. For now this is a plain session dependency for FastAPI
    ``Depends(get_session)``. Keeps interface stable for Task 2+.
    """
    async with SessionLocal() as session:
        yield session
