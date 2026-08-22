import json
from typing import Optional

try:
    # Prefer valkey, fallback to redis for backwards compat
    try:
        import valkey.asyncio as redis  # type: ignore[import-not-found]
    except ImportError:
        import redis.asyncio as redis  # type: ignore[no-redef]

    from app.config import settings

    _redis = None

    def get_redis():
        global _redis
        if _redis is None:
            # valkey_url is primary, fallback to redis_url
            url = getattr(settings, "valkey_url", getattr(settings, "redis_url", "valkey://localhost:6379/0"))
            _redis = redis.from_url(url, decode_responses=True)
        return _redis
except Exception:
    get_redis = lambda: None  # type: ignore


async def cache_get(key: str) -> Optional[str]:
    try:
        r = get_redis()
        if r:
            return await r.get(key)
    except Exception:
        pass
    return None


async def cache_set(key: str, value: str, ttl: int = 60):
    try:
        r = get_redis()
        if r:
            await r.setex(key, ttl, value)
    except Exception:
        pass


async def cache_delete(pattern: str):
    try:
        r = get_redis()
        if r:
            # pattern may be exact key
            await r.delete(pattern)
    except Exception:
        pass
