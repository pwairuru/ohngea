import json
from typing import Optional

try:
    import redis.asyncio as redis
    from app.config import settings

    _redis = None

    def get_redis():
        global _redis
        if _redis is None:
            _redis = redis.from_url(settings.redis_url, decode_responses=True)
        return _redis
except Exception:
    get_redis = lambda: None


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
