from uuid import UUID
from functools import lru_cache

from fastapi import Depends, HTTPException
from fastapi.responses import JSONResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request

from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import SessionLocal, get_session, set_tenant
from app.registry import discover_apps

ALWAYS_ENABLED = {"extensions", "dialplans", "gateways", "devices", "cdr", "domains"}

# paths that never gate (health/docs/auth/internal/provision/openapi)
SKIP_PREFIXES = (
    "/health",
    "/docs",
    "/redoc",
    "/openapi.json",
    "/api/v1/openapi.json",
    "/api/v1/health",
    "/api/v1/auth",
    "/api/internal",
    "/provision",
)

@lru_cache(maxsize=1)
def _known_apps() -> set[str]:
    try:
        return {a["name"] for a in discover_apps()}
    except Exception:
        return set()

@lru_cache(maxsize=1)
def _default_enabled_map() -> dict[str, bool]:
    try:
        return {a["name"]: bool(a.get("enabled_by_default", True)) for a in discover_apps()}
    except Exception:
        return {}


def _extract_domain_id(request: Request) -> UUID | None:
    # 1. already set by TenantMiddleware
    did = getattr(request.state, "domain_id", None)
    if did is not None:
        try:
            return did if isinstance(did, UUID) else UUID(str(did))
        except Exception:
            pass
    # 2. JWT Bearer
    auth = request.headers.get("authorization", "")
    if auth.startswith("Bearer "):
        token = auth.removeprefix("Bearer ").strip()
        try:
            import jwt

            payload = jwt.decode(token, options={"verify_signature": False})
            raw = payload.get("domain_id")
            if raw:
                return UUID(str(raw))
        except Exception:
            pass
    # 3. X-Domain-Id header (superadmin)
    hdr = request.headers.get("x-domain-id")
    if hdr:
        try:
            return UUID(hdr)
        except Exception:
            pass
    return None


class AppGateMiddleware(BaseHTTPMiddleware):
    """Central gate: check /api/v1/<app> prefix against DomainApp overrides.

    - extracts app_name as first segment after /api/v1/
    - skips ALWAYS_ENABLED (6 primary nav) and unknown prefixes
    - looks up DomainApp for domain_id, falls back to discover_apps default
    - returns 403 JSON if disabled
    """

    async def dispatch(self, request: Request, call_next: RequestResponseEndpoint):
        path = request.url.path
        # skip non-api and explicit skip prefixes
        if not path.startswith("/api/v1/"):
            return await call_next(request)
        for p in SKIP_PREFIXES:
            if path == p or path.startswith(p + "/"):
                return await call_next(request)
        # also skip exact health/docs variants
        if path in ("/health", "/docs", "/openapi.json", "/redoc"):
            return await call_next(request)

        remainder = path[len("/api/v1/"):]
        if not remainder:
            return await call_next(request)
        app_name = remainder.split("/")[0].split("?")[0]
        if not app_name:
            return await call_next(request)
        # skip always-enabled primary nav
        if app_name in ALWAYS_ENABLED:
            return await call_next(request)
        # only gate known apps (from registry)
        known = _known_apps()
        if known and app_name not in known:
            return await call_next(request)
        # need domain_id to check; unauthenticated -> let downstream return 401
        domain_id = _extract_domain_id(request)
        if domain_id is None:
            return await call_next(request)

        # DB check — support dependency_overrides for tests (in-memory sqlite)
        async def _check_enabled(session: AsyncSession) -> bool | None:
            from app.domains.models import DomainApp

            try:
                await set_tenant(session, domain_id)
            except Exception:
                pass
            r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id, DomainApp.app_name == app_name))
            row = r.scalar_one_or_none()
            if row is not None:
                return bool(row.enabled)
            return _default_enabled_map().get(app_name, True)

        try:
            # if tests override get_session, use that session so we see same in-memory DB
            from app.main import app as fastapi_app

            if get_session in fastapi_app.dependency_overrides:
                override = fastapi_app.dependency_overrides[get_session]
                gen = override()
                # async generator
                session = await gen.__anext__()
                try:
                    enabled = await _check_enabled(session)
                    if enabled is False:
                        return JSONResponse(status_code=403, content={"detail": f"App {app_name} is disabled for this domain"})
                finally:
                    try:
                        await gen.aclose()
                    except StopAsyncIteration:
                        pass
                    except Exception:
                        pass
            else:
                async with SessionLocal() as session:
                    enabled = await _check_enabled(session)
                    if enabled is False:
                        return JSONResponse(status_code=403, content={"detail": f"App {app_name} is disabled for this domain"})
        except Exception:
            # on DB error, fail open to avoid blocking; downstream may handle
            return await call_next(request)

        return await call_next(request)


def require_app_enabled(app_name: str):
    async def _check(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
        from app.domains.models import DomainApp
        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id, DomainApp.app_name == app_name))
        row = r.scalar_one_or_none()
        if row is not None:
            enabled = row.enabled
        else:
            apps = {a["name"]: a for a in discover_apps()}
            enabled = apps.get(app_name, {}).get("enabled_by_default", True)
        if not enabled:
            raise HTTPException(status_code=403, detail=f"App {app_name} is disabled for this domain")
        return True
    return Depends(_check)
