from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.sip_profiles.models import SipProfile

router = APIRouter(prefix="/api/v1/sip_profiles", tags=["sip_profiles"])


@router.get("")
async def list_profiles(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(SipProfile).where(SipProfile.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(SipProfile).where(SipProfile.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    data = [{"sip_profile_uuid": str(p.sip_profile_uuid), "name": p.name, "enabled": p.enabled, "_links": {"self": {"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}} for p in r.scalars().all()]
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/sip_profiles?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_profile(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    p = SipProfile(domain_id=domain_id, name=payload.get("name"), enabled=payload.get("enabled", True), sip_port=payload.get("sip_port", 5060))
    session.add(p)
    await session.commit()
    await session.refresh(p)
    response.headers["Location"] = f"/api/v1/sip_profiles/{p.sip_profile_uuid}"
    return {"sip_profile_uuid": str(p.sip_profile_uuid), "name": p.name, "_links": {"self": {"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}}
