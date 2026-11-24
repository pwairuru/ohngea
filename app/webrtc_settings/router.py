from datetime import datetime, timezone
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache_for_session
from app.webrtc_settings.models import WebRTCSettings

router = APIRouter(prefix="/api/v1/webrtc-settings", tags=["webrtc_settings"])


@router.get("/domains/{domain_id}/settings")
async def get_webrtc_settings(
    domain_id: UUID,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    from app.auth.service import get_user_permissions as _gup
    caller_domain = getattr(user, "_domain_id", user.domain_id)
    _perms = await _gup(session, user)
    if str(caller_domain) != str(domain_id) and "*" not in _perms:
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    r = await session.execute(select(WebRTCSettings).where(WebRTCSettings.domain_id == domain_id))
    row = r.scalar_one_or_none()
    if not row:
        return {
            "domain_id": str(domain_id),
            "enabled": False,
            "auth_mode": "extension",
            "wss_url": None,
            "sip_domain": None,
            "ice_servers": None,
        }
    return {
        "domain_id": str(row.domain_id),
        "enabled": row.enabled,
        "auth_mode": row.auth_mode,
        "wss_url": row.wss_url,
        "sip_domain": row.sip_domain,
        "ice_servers": row.ice_servers,
        "updated_at": row.updated_at.isoformat(),
    }


@router.put("/domains/{domain_id}/settings")
async def put_webrtc_settings(
    domain_id: UUID,
    payload: dict,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    from app.auth.service import get_user_permissions
    caller_domain = getattr(user, "_domain_id", user.domain_id)
    perms = await get_user_permissions(session, user)
    if str(caller_domain) != str(domain_id) and "*" not in perms:
        raise HTTPException(status_code=403, detail="Domain mismatch")
    # permission check: allow domain_edit, domain_add, or superadmin/wildcard
    if "*" not in perms and "domain_edit" not in perms and "domain_add" not in perms and "superadmin" not in perms:
        raise HTTPException(status_code=403, detail="Missing permission: domain_edit")
    # validation via model
    from app.webrtc_settings.models import WebRTCSettingsBase

    try:
        validated = WebRTCSettingsBase(**payload)
    except Exception as e:
        raise HTTPException(status_code=422, detail=str(e))
    await set_tenant(session, domain_id)
    r = await session.execute(select(WebRTCSettings).where(WebRTCSettings.domain_id == domain_id))
    row = r.scalar_one_or_none()
    if row:
        row.enabled = validated.enabled
        row.auth_mode = validated.auth_mode
        row.wss_url = validated.wss_url
        row.sip_domain = validated.sip_domain
        row.ice_servers = validated.ice_servers
        row.updated_at = datetime.now(timezone.utc)
    else:
        row = WebRTCSettings(
            domain_id=domain_id,
            enabled=validated.enabled,
            auth_mode=validated.auth_mode,
            wss_url=validated.wss_url,
            sip_domain=validated.sip_domain,
            ice_servers=validated.ice_servers,
        )
        session.add(row)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(row)
    return {
        "domain_id": str(row.domain_id),
        "enabled": row.enabled,
        "auth_mode": row.auth_mode,
        "wss_url": row.wss_url,
        "sip_domain": row.sip_domain,
        "ice_servers": row.ice_servers,
        "updated_at": row.updated_at.isoformat(),
    }


# Legacy compat router for old prefix /api/v1/domains/{id}/webrtc_settings
from fastapi import APIRouter as _AR
from fastapi.responses import RedirectResponse as _RR2
legacy_router = _AR(tags=["webrtc_settings"])

@legacy_router.get("/api/v1/domains/{domain_id}/webrtc_settings", include_in_schema=False)
async def _legacy_get(domain_id: UUID):
    return _RR2(url=f"/api/v1/webrtc-settings/domains/{domain_id}/settings", status_code=307)

@legacy_router.put("/api/v1/domains/{domain_id}/webrtc_settings", include_in_schema=False)
async def _legacy_put(domain_id: UUID):
    return _RR2(url=f"/api/v1/webrtc-settings/domains/{domain_id}/settings", status_code=307)
