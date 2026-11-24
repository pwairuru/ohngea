"""Simple in-memory rate limiter for auth/provision/cdr endpoints.

No new dependency. Per-IP sliding window. Returns 429 with Retry-After.
Failed-login lockout tracked separately in auth router.
"""
from __future__ import annotations

import time
from collections import defaultdict, deque

from fastapi.responses import JSONResponse
from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request

# path prefix -> (limit, window_seconds)
LIMITS: dict[str, tuple[int, int]] = {
    "/api/v1/auth/login": (5, 60),
    "/auth/login": (5, 60),
    "/api/v1/auth/refresh": (10, 60),
    "/provision/": (30, 60),
    "/api/internal/freeswitch/xml": (60, 60),
    "/api/v1/cdr/import": (60, 60),
}

_hits: dict[str, deque[float]] = defaultdict(deque)


def _key(ip: str, route: str) -> str:
    return f"{ip}|{route}"


class RateLimitMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next: RequestResponseEndpoint):
        path = request.url.path
        matched: tuple[int, int] | None = None
        for prefix, lim in LIMITS.items():
            if path == prefix or path.startswith(prefix):
                matched = lim
                break
        if matched is None:
            return await call_next(request)
        limit, window = matched
        ip = request.client.host if request.client else "unknown"
        k = _key(ip, prefix)
        now = time.time()
        dq = _hits[k]
        while dq and dq[0] <= now - window:
            dq.popleft()
        if len(dq) >= limit:
            retry = int(dq[0] + window - now) + 1
            return JSONResponse(
                status_code=429,
                content={"detail": "Too many requests"},
                headers={"Retry-After": str(retry)},
            )
        dq.append(now)
        return await call_next(request)


# Login lockout: username -> [fail timestamps]
_failed: dict[str, deque[float]] = defaultdict(deque)
LOCKOUT_AFTER = 5
LOCKOUT_WINDOW = 15 * 60


def is_locked(username: str) -> bool:
    dq = _failed.get(username.lower(), deque())
    now = time.time()
    while dq and dq[0] <= now - LOCKOUT_WINDOW:
        dq.popleft()
    return len(dq) >= LOCKOUT_AFTER


def record_failure(username: str) -> None:
    _failed[username.lower()].append(time.time())


def record_success(username: str) -> None:
    _failed.pop(username.lower(), None)
