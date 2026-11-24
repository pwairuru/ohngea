import time
import logging
import uuid
from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request

logger = logging.getLogger("audit")


class AuditMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next: RequestResponseEndpoint):
        req_id = request.headers.get("x-request-id") or uuid.uuid4().hex[:12]
        request.state.request_id = req_id
        start = time.time()
        # skip noisy health/docs
        noisy = request.url.path in ("/health", "/api/v1/health", "/docs", "/openapi.json", "/api/v1/openapi.json")
        try:
            response = await call_next(request)
        except Exception as e:
            logger.error("req=%s %s %s failed: %s", req_id, request.method, request.url.path, e, exc_info=True)
            raise
        dur = (time.time() - start) * 1000
        if not noisy:
            domain = getattr(request.state, "domain_id", "-")
            logger.info("req=%s %s %s -> %s %.1fms domain=%s", req_id, request.method, request.url.path, response.status_code, dur, domain)
        response.headers["X-Request-ID"] = req_id
        response.headers["X-Request-Duration-ms"] = str(int(dur))
        return response
