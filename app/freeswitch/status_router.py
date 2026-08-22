import time
import json
from datetime import datetime, timezone
from fastapi import APIRouter, Depends
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session, set_tenant
from app.config import settings
from app.freeswitch.esl_client import esl_client, parse_sofia_status
from app.freeswitch.cache import cache_get, cache_set

router = APIRouter(prefix="/api/v1/freeswitch", tags=["freeswitch-status"])
sip_status_router = APIRouter(prefix="/api/v1/sip_profiles", tags=["sip_profiles-status"])


async def _get_fs_status_cached() -> dict:
    """Get global FS connected status with 15s cache (tenant-agnostic)."""
    cache_key = "fs:status:global"
    cached = await cache_get(cache_key)
    if cached:
        try:
            return json.loads(cached)
        except Exception:
            pass
    # probe FS
    probe = await esl_client.check_status()
    result = {
        "connected": bool(probe.get("connected")),
        "host": settings.fs_esl_host,
        "port": settings.fs_esl_port,
        "latency_ms": probe.get("latency_ms"),
        "error": probe.get("error"),
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }
    try:
        await cache_set(cache_key, json.dumps(result), ttl=15)
    except Exception:
        pass
    return result


@router.get("/status")
async def freeswitch_status(user: User = Depends(get_current_user)):
    """Global FreeSWITCH ESL connectivity check. Auth required. Tenant-agnostic (infra)."""
    return await _get_fs_status_cached()


@sip_status_router.get("/status")
async def sip_profiles_status(
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    """Per-profile SIP status, tenant-aware.

    - Only returns profiles belonging to caller's domain_id (RLS + filter).
    - FS sofia status is parsed and filtered to those names; unknown if FS down or not found.
    - Result cached 15s per domain to avoid hammering ESL.
    """
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)

    cache_key = f"fs:sip_status:{domain_id}"
    cached = await cache_get(cache_key)
    if cached:
        try:
            return json.loads(cached)
        except Exception:
            pass

    # fetch tenant's profiles from DB
    from app.sip_profiles.models import SipProfile

    r = await session.execute(select(SipProfile).where(SipProfile.domain_id == domain_id))
    profiles = r.scalars().all()

    # get FS global connected + sofia map (also cached via _get_fs_status_cached + separate sofia cache)
    fs_global = await _get_fs_status_cached()
    connected = fs_global.get("connected", False)

    sofia_map: dict[str, str] = {}
    error = fs_global.get("error")
    if connected:
        # try cached sofia raw separately to avoid extra ESL call if recently fetched
        sofia_cache_key = "fs:sofia:raw"
        raw_cached = await cache_get(sofia_cache_key)
        if raw_cached is not None:
            sofia_map = parse_sofia_status(raw_cached)
        else:
            # direct ESL call
            raw = await esl_client.api("sofia status")
            low = raw.lower() if isinstance(raw, str) else ""
            if any(x in low for x in ("connection refused", "timeout", "error:", "auth failed")):
                connected = False
                error = raw.strip()[:300] if isinstance(raw, str) else error
                sofia_map = {}
            else:
                sofia_map = parse_sofia_status(raw)
                try:
                    await cache_set(sofia_cache_key, raw, ttl=15)
                except Exception:
                    pass

    data = []
    for p in profiles:
        # FS profile names are case-sensitive typically lowercased; we match case-insensitively
        # but preserve DB name
        fs_state = sofia_map.get(p.name)
        if fs_state is None:
            # try case-insensitive lookup
            for k, v in sofia_map.items():
                if k.lower() == p.name.lower():
                    fs_state = v
                    break
        state = fs_state if fs_state else ("UNKNOWN" if connected else "UNKNOWN")
        # If FS is down, we explicitly mark UNKNOWN, frontend shows grey
        data.append(
            {
                "sip_profile_uuid": str(p.sip_profile_uuid),
                "name": p.name,
                "sip_port": p.sip_port,
                "enabled": p.enabled,
                "fs_state": state,
                "running": state == "RUNNING",
            }
        )

    result = {
        "connected": connected,
        "error": error if not connected else None,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "profiles": data,
    }
    try:
        await cache_set(cache_key, json.dumps(result), ttl=15)
    except Exception:
        pass
    return result
