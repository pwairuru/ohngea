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
    # Supports exact key or glob pattern; uses SCAN for patterns containing * or ?
    try:
        r = get_redis()
        if not r:
            return
        if "*" in pattern or "?" in pattern:
            cursor = "0"
            while True:
                cursor, keys = await r.scan(cursor=cursor, match=pattern, count=200)
                if keys:
                    await r.delete(*keys)
                if cursor == "0" or cursor == 0:
                    break
        else:
            await r.delete(pattern)
    except Exception:
        pass

async def cache_delete_many(keys: list[str]):
    try:
        r = get_redis()
        if r and keys:
            await r.delete(*keys)
    except Exception:
        pass


async def invalidate_fs_cache(domain_id=None, domain_name: Optional[str] = None):
    """Invalidate all FS xml_curl cache keys for a domain and trigger reloadxml.

    Best-effort: never raises. Called after every CRUD that affects FS config.
    Mirrors logic in app/extensions/service.py::_invalidate_fs_cache but is
    reusable for all apps (dialplans, gateways, ACLs, ring_groups, etc.).
    """
    # Delete cache keys for domain
    try:
        if domain_name:
            # per-domain xml keys: xml:directory:<domain>, xml:dialplan:<domain>, etc.
            for pattern in [
                f"xml:directory:*@{domain_name}",
                f"xml:directory:{domain_name}",
                f"xml:dialplan:{domain_name}",
                f"xml:configuration:*{domain_name}*",
                f"xml:*:{domain_name}",
                f"xml:*:{domain_name}:*",
            ]:
                try:
                    await cache_delete(pattern)
                except Exception:
                    pass
            # also exact deletes (cache_delete handles exact)
            for key in [f"xml:directory:{domain_name}", f"xml:dialplan:{domain_name}", f"xml:configuration:{domain_name}"]:
                try:
                    await cache_delete(key)
                except Exception:
                    pass
            # wildcard purge for ACL/gateway configs that use key_value like acl.conf
            try:
                await cache_delete("xml:configuration:*")
            except Exception:
                pass
        if domain_id:
            for key in [f"xml:{domain_id}:directory", f"xml:{domain_id}:dialplan"]:
                try:
                    await cache_delete(key)
                except Exception:
                    pass
        # fallback global invalidate if no domain info (purge all xml)
        if not domain_name and not domain_id:
            try:
                await cache_delete("xml:*")
            except Exception:
                pass
    except Exception:
        pass
    # ESL reloadxml best-effort
    try:
        from app.freeswitch.esl_client import esl_client

        await esl_client.reloadxml()
    except Exception:
        pass


async def invalidate_fs_cache_for_session(session, domain_id):
    """Helper that looks up domain_name from DB then invalidates."""
    domain_name = None
    if domain_id:
        try:
            from sqlalchemy import select

            from app.domains.models import Domain

            r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
            dom = r.scalar_one_or_none()
            if dom:
                domain_name = dom.domain_name
        except Exception:
            domain_name = None
    await invalidate_fs_cache(domain_id, domain_name)
