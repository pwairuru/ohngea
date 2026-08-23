from datetime import datetime
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache_for_session
from app.webrtc_settings.models import WebRTCSettings

router = APIRouter(prefix="/api/v1/domains", tags=["webrtc_settings"])


@router.get("/{domain_id}/webrtc_settings")
async def get_webrtc_settings(
    domain_id: UUID,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    caller_domain = getattr(user, "_domain_id", user.domain_id)
    if str(caller_domain) != str(domain_id) and "superadmin" not in getattr(user, "_perms", []):
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


@router.put("/{domain_id}/webrtc_settings")
async def put_webrtc_settings(
    domain_id: UUID,
    payload: dict,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    caller_domain = getattr(user, "_domain_id", user.domain_id)
    if str(caller_domain) != str(domain_id) and "superadmin" not in getattr(user, "_perms", []):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    # permission check: allow domain_edit, domain_add, or superadmin/wildcard
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions

        perms = await get_user_permissions(session, user)
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
        row.updated_at = datetime.utcnow()
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
    except Exception:
        pass
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
