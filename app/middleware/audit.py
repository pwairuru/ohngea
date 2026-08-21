import time
import logging
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request

logger = logging.getLogger("audit")


class AuditMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request, call_next):
        start = time.time()
        response = await call_next(request)
        dur = (time.time() - start) * 1000
        logger.info(f"{request.method} {request.url.path} -> {response.status_code} {dur:.1f}ms")
        response.headers["X-Request-Duration-ms"] = str(int(dur))
        return response
