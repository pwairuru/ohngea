from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1/webrtc", tags=["webrtc"])


@router.post("/credentials")
async def get_credentials(payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    from app.webrtc_settings.models import WebRTCSettings
    from app.extensions.models import Extension
    from app.domains.models import Domain
    r = await session.execute(select(WebRTCSettings).where(WebRTCSettings.domain_id == domain_id))
    settings = r.scalar_one_or_none()
    if not settings or not settings.enabled:
        raise HTTPException(status_code=400, detail="WebPhone disabled for this domain")
    auth_mode = settings.auth_mode
    wss_url = settings.wss_url
    if not wss_url:
        raise HTTPException(status_code=400, detail="wss_url not configured")
    # sip_domain fallback
    sip_domain = settings.sip_domain
    if not sip_domain:
        r2 = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
        dom = r2.scalar_one_or_none()
        sip_domain = dom.domain_name if dom else "example.com"
    # handle extension mode
    ext_val = payload.get("extension")
    webrtc_user = payload.get("webrtc_user")
    webrtc_pass = payload.get("webrtc_pass")
    if ext_val:
        if auth_mode not in ("extension", "both"):
            raise HTTPException(status_code=400, detail="extension auth disabled")
        r3 = await session.execute(select(Extension).where(Extension.domain_id == domain_id, Extension.extension == str(ext_val)))
        ext = r3.scalar_one_or_none()
        if not ext or not ext.enabled:
            raise HTTPException(status_code=404, detail="Extension not found")
        # optional: check user owns extension later; phase1 allow any in domain
        return {"sip_uri": f"sip:{ext.extension}@{sip_domain}", "wss_url": wss_url, "auth_user": str(ext.extension), "password": ext.password or "1234", "display_name": ext.effective_caller_id_name or str(ext.extension), "expires_in": 3600, "ice_servers": settings.ice_servers or [{"urls": "stun:stun.l.google.com:19302"}]}
    elif webrtc_user:
        if auth_mode not in ("webrtc", "both"):
            raise HTTPException(status_code=400, detail="webrtc auth disabled")
        # phase1 echo
        pwd = webrtc_pass or ""
        return {"sip_uri": f"sip:{webrtc_user}@{sip_domain}", "wss_url": wss_url, "auth_user": webrtc_user, "password": pwd, "display_name": webrtc_user, "expires_in": 3600, "ice_servers": settings.ice_servers or [{"urls": "stun:stun.l.google.com:19302"}]}
    else:
        raise HTTPException(status_code=422, detail="Provide extension or webrtc_user")
