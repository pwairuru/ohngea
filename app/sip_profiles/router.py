from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache_for_session
from app.sip_profiles.models import SipProfile

router = APIRouter(prefix="/api/v1/sip_profiles", tags=["sip_profiles"])


@router.get("")
async def list_profiles(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(SipProfile).where(SipProfile.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(SipProfile).where(SipProfile.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    rows=r.scalars().all()
    data=[]
    for p in rows:
        d={c.name: getattr(p,c.name) for c in p.__table__.columns}
        for k in ["sip_profile_uuid","domain_id"]:
            if d.get(k): d[k]=str(d[k])
        for k in ["created_at","updated_at"]:
            if d.get(k):
                try: d[k]=d[k].isoformat()
                except Exception: pass
        d["_links"]={"self":{"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}
        data.append(d)
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/sip_profiles?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_profile(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    allowed={c.name for c in SipProfile.__table__.columns} - {"sip_profile_uuid","created_at","updated_at","domain_id"}
    data={k:v for k,v in payload.items() if k in allowed}
    data["domain_id"]=domain_id
    if not data.get("name"):
        raise HTTPException(status_code=422, detail="name required")
    p = SipProfile(**data)
    session.add(p)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    # regenerate sofia.conf.xml from DB
    try:
        from app.freeswitch.sofia_config import regenerate_sofia_conf
        await regenerate_sofia_conf(session)
    except Exception:
        pass
    await session.refresh(p)
    response.headers["Location"] = f"/api/v1/sip_profiles/{p.sip_profile_uuid}"
    return {"sip_profile_uuid": str(p.sip_profile_uuid), "name": p.name, "sip_port": p.sip_port, "sip_ip": p.sip_ip, "ext_sip_ip": getattr(p, "ext_sip_ip", None), "ext_rtp_ip": getattr(p, "ext_rtp_ip", None), "realm": getattr(p, "realm", None), "context": getattr(p, "context", None), "wss_port": p.wss_port, "wss_binding": p.wss_binding, "tls_cert_path": p.tls_cert_path, "tls_key_path": p.tls_key_path, "enabled": p.enabled, "_links": {"self": {"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}}


@router.get("/{sip_profile_uuid}")
async def get_profile(sip_profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    p = r.scalar_one_or_none()
    if not p:
        raise HTTPException(status_code=404, detail="SIP profile not found")
    d={c.name: getattr(p,c.name) for c in p.__table__.columns}
    for k in ["sip_profile_uuid","domain_id"]:
        if d.get(k): d[k]=str(d[k])
    for k in ["created_at","updated_at"]:
        if d.get(k):
            try: d[k]=d[k].isoformat()
            except Exception: pass
    d["_links"]={"self":{"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}
    return d


@router.patch("/{sip_profile_uuid}")
async def patch_profile(sip_profile_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    p = r.scalar_one_or_none()
    if not p:
        raise HTTPException(status_code=404, detail="SIP profile not found")
    for k in ["name", "sip_port", "sip_ip", "ext_sip_ip", "ext_rtp_ip", "realm", "context", "challenge_realm", "enabled", "wss_port", "wss_binding", "tls_cert_path", "tls_key_path", "rtp_ip", "apply_inbound_acl", "local_network_acl", "description"]:
        if k in payload:
            setattr(p, k, payload[k])
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    try:
        from app.freeswitch.sofia_config import regenerate_sofia_conf
        await regenerate_sofia_conf(session)
    except Exception:
        pass
    await session.refresh(p)
    return {"sip_profile_uuid": str(p.sip_profile_uuid), "name": p.name, "sip_port": p.sip_port, "sip_ip": p.sip_ip, "ext_sip_ip": getattr(p, "ext_sip_ip", None), "ext_rtp_ip": getattr(p, "ext_rtp_ip", None), "realm": getattr(p, "realm", None), "context": getattr(p, "context", None), "wss_port": p.wss_port, "wss_binding": p.wss_binding, "tls_cert_path": p.tls_cert_path, "tls_key_path": p.tls_key_path, "enabled": p.enabled, "_links": {"self": {"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}}


@router.delete("/{sip_profile_uuid}", status_code=204)
async def delete_profile(sip_profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    p = r.scalar_one_or_none()
    if not p:
        raise HTTPException(status_code=404, detail="SIP profile not found")
    await session.delete(p)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    try:
        from app.freeswitch.sofia_config import regenerate_sofia_conf
        await regenerate_sofia_conf(session)
    except Exception:
        pass
    return Response(status_code=204)
