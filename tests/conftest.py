import pytest
from uuid import uuid4

import pytest_asyncio
from httpx import ASGITransport, AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.pool import StaticPool
from sqlmodel import SQLModel

from app.db import get_session
from app.main import app


@pytest.fixture(scope="session")
def anyio_backend():
    return "asyncio"


@pytest_asyncio.fixture
async def test_db():
    """Per-test in-memory SQLite DB with all tables created."""
    engine = create_async_engine(
        "sqlite+aiosqlite:///:memory:",
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )
    # Ensure all models are imported so metadata is populated
    try:
        import app.webrtc_settings.models  # noqa: F401
    except ImportError:
        pass
    # Import core models to ensure tables exist
    import app.domains.models  # noqa: F401
    import app.auth.models  # noqa: F401
    import app.extensions.models  # noqa: F401
    import app.sip_profiles.models  # noqa: F401
    import app.call_forward.models  # noqa: F401
    try:
        import app.webrtc.credentials_router  # noqa: F401
    except ImportError:
        pass

    async with engine.begin() as conn:
        await conn.run_sync(SQLModel.metadata.create_all)
    sm = async_sessionmaker(engine, expire_on_commit=False, class_=AsyncSession)
    yield sm
    await engine.dispose()


@pytest_asyncio.fixture
async def client(test_db):
    """Async HTTP client with DB dependency overridden to test_db."""
    async def override_get_session():
        async with test_db() as session:
            yield session

    app.dependency_overrides[get_session] = override_get_session
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        yield ac
    app.dependency_overrides.clear()


@pytest_asyncio.fixture
async def domain(test_db):
    """Primary tenant domain."""
    from app.domains.models import Domain

    async with test_db() as session:
        d = Domain(domain_name=f"test-{uuid4().hex[:8]}.example.com", enabled=True)
        session.add(d)
        await session.commit()
        await session.refresh(d)
        return d


@pytest_asyncio.fixture
async def other_domain(test_db):
    """Second tenant domain for isolation tests."""
    from app.domains.models import Domain

    async with test_db() as session:
        d = Domain(domain_name=f"other-{uuid4().hex[:8]}.example.com", enabled=True)
        session.add(d)
        await session.commit()
        await session.refresh(d)
        return d


async def _make_user(test_db, domain, username, perms):
    """Create user bound to domain and return (user, token)."""
    from app.auth.models import Group, GroupPermission, Permission, User, UserGroup
    from app.auth.service import create_token, hash_password

    async with test_db() as session:
        # Ensure permission rows exist for perms (except "*")
        for p in perms:
            if p == "*":
                continue
            r = await session.execute(
                __import__("sqlalchemy").select(Permission).where(Permission.name == p)
            )
            if not r.scalar_one_or_none():
                session.add(Permission(name=p))
        await session.commit()

        user = User(
            username=username,
            domain_id=domain.domain_id,
            password_hash=hash_password("password123"),
            enabled=True,
        )
        session.add(user)
        await session.commit()
        await session.refresh(user)

        # If perms is not wildcard-only, create group with those perms and link user
        non_wild = [p for p in perms if p != "*"]
        if non_wild:
            grp = Group(domain_id=domain.domain_id, name=f"grp-{username}-{uuid4().hex[:4]}")
            session.add(grp)
            await session.commit()
            await session.refresh(grp)
            # link permissions
            for pname in non_wild:
                r = await session.execute(
                    __import__("sqlalchemy").select(Permission).where(Permission.name == pname)
                )
                perm = r.scalar_one()
                session.add(GroupPermission(group_uuid=grp.group_uuid, permission_uuid=perm.permission_uuid))
            await session.commit()
            # link user to group
            session.add(UserGroup(user_uuid=user.user_uuid, group_uuid=grp.group_uuid, domain_id=domain.domain_id))
            await session.commit()

        token = create_token(user, permissions=perms)
        # Attach _perms/_domain_id like get_current_user does (for completeness)
        user._perms = perms
        user._domain_id = domain.domain_id
        return user, token


@pytest_asyncio.fixture
async def admin_user(test_db, domain):
    _, token = await _make_user(test_db, domain, f"admin-{uuid4().hex[:4]}", ["*"])
    # Re-fetch user for return if needed
    return token


@pytest_asyncio.fixture
async def auth_headers(test_db, domain):
    _, token = await _make_user(test_db, domain, f"admin-{uuid4().hex[:4]}", ["*"])
    return {"Authorization": f"Bearer {token}"}


@pytest_asyncio.fixture
async def auth_headers_user(test_db, domain):
    """Regular user without domain_edit (only extension_view)."""
    _, token = await _make_user(test_db, domain, f"user-{uuid4().hex[:4]}", ["extension_view"])
    return {"Authorization": f"Bearer {token}"}


@pytest_asyncio.fixture
async def other_auth_headers(test_db, other_domain):
    _, token = await _make_user(test_db, other_domain, f"admin2-{uuid4().hex[:4]}", ["*"])
    return {"Authorization": f"Bearer {token}"}


@pytest_asyncio.fixture
async def extension(test_db, domain):
    """Extension fixture for webrtc credential tests."""
    from app.extensions.models import Extension

    async with test_db() as session:
        ext = Extension(
            extension="1001",
            effective_caller_id_name="Test User",
            domain_id=domain.domain_id,
            password="secret123",
            enabled=True,
        )
        session.add(ext)
        await session.commit()
        await session.refresh(ext)
        return ext
