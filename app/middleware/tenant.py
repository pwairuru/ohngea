from uuid import UUID

from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request
from sqlalchemy import text

from app.db import SessionLocal


class TenantMiddleware(BaseHTTPMiddleware):
    """Set RLS GUC `app.domain_uuid` per request from JWT domain_id.

    Must run after auth: it reads `request.state.domain_id` set by auth dep,
    or decodes JWT directly for non-auth routes (e.g. health excluded).
    For simplicity, we set the GUC inside the request's DB transaction
    via a one-off session that does SET LOCAL — but SET LOCAL is transaction-local,
    so we really need to ensure every DB session in the request does it.

    Simpler approach: the `get_session` dependency itself will call `set_tenant`
    if `request.state.domain_id` is present. This middleware just extracts domain_id
    from Authorization header early and stores it on request.state.
    """

    async def dispatch(self, request: Request, call_next: RequestResponseEndpoint):
        # Skip health / docs / internal FS
        if request.url.path in ("/health", "/docs", "/openapi.json", "/redoc"):
            return await call_next(request)
        # try extract domain_id from JWT without verifying (verify later in deps)
        auth = request.headers.get("authorization", "")
        if auth.startswith("Bearer "):
            token = auth.removeprefix("Bearer ").strip()
            try:
                import jwt
                from app.config import settings

                payload = jwt.decode(token, options={"verify_signature": False})
                did = payload.get("domain_id")
                if did:
                    try:
                        request.state.domain_id = UUID(did)
                    except Exception:
                        pass
            except Exception:
                pass
        # also allow X-Domain-Id header for superadmin
        hdr = request.headers.get("x-domain-id")
        if hdr:
            try:
                request.state.domain_id = UUID(hdr)
            except Exception:
                pass
        return await call_next(request)
