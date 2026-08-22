from fastapi import APIRouter, Depends
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session, set_tenant
router = APIRouter(prefix="/api/v1/domain_limits", tags=["domain_limits"])
@router.get("", response_model=dict)
async def domain_limits(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # counts per tenant for key resources
    from app.extensions.models import Extension
    from app.dialplans.models import Dialplan
    from app.gateways.models import Gateway
    from app.devices.models import Device
    from app.voicemails.models import Voicemail
    # default_settings limits (if present) else hardcoded 1000 fallback
    limits={}
    try:
        from app.domains.models import DomainSettings
        # domain_limits stored as default_settings category limit? fallback
        r=await session.execute(select(DomainSettings).where(DomainSettings.domain_id==domain_id))
        settings_rows=r.scalars().all()
        # map subcategory -> value for category == limits
        for s in settings_rows:
            if getattr(s,"category",None)=="limit" or "limit" in getattr(s,"subcategory","").lower():
                limits[s.subcategory]=s.value
    except: pass
    def limit_for(key, default=1000):
        return int(limits.get(key, default)) if limits.get(key) else default
    async def cnt(model):
        try:
            return (await session.execute(select(func.count()).select_from(model).where(model.domain_id==domain_id))).scalar_one()
        except: return 0
    ext_cnt=await cnt(Extension)
    dial_cnt=await cnt(Dialplan)
    gw_cnt=await cnt(Gateway)
    dev_cnt=await cnt(Device)
    vm_cnt=await cnt(Voicemail)
    return {
        "domain_id": str(domain_id),
        "limits": {
            "extensions": {"used": ext_cnt, "allowed": limit_for("extensions", 1000), "remaining": limit_for("extensions",1000)-ext_cnt},
            "dialplans": {"used": dial_cnt, "allowed": limit_for("dialplans", 1000), "remaining": limit_for("dialplans",1000)-dial_cnt},
            "gateways": {"used": gw_cnt, "allowed": limit_for("gateways", 100), "remaining": limit_for("gateways",100)-gw_cnt},
            "devices": {"used": dev_cnt, "allowed": limit_for("devices", 1000), "remaining": limit_for("devices",1000)-dev_cnt},
            "voicemails": {"used": vm_cnt, "allowed": limit_for("voicemails", 1000), "remaining": limit_for("voicemails",1000)-vm_cnt},
        },
        "_links": {"self": {"href": "/api/v1/domain_limits"}}
    }
