from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache_for_session
from app.sip_profiles.models import SipProfile, SipProfileDomain, SipProfileSetting

router = APIRouter(prefix="/api/v1/sip_profiles", tags=["sip_profiles"])


def _ser(obj):
    d = {c.name: getattr(obj, c.name) for c in obj.__table__.columns}
    for k in ["sip_profile_uuid", "domain_id", "sip_profile_domain_uuid", "sip_profile_setting_uuid"]:
        if d.get(k):
            d[k] = str(d[k])
    for k in ["created_at", "updated_at"]:
        if d.get(k):
            try:
                d[k] = d[k].isoformat()
            except Exception:
                pass
    return d


@router.get("")
async def list_profiles(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(SipProfile).where(SipProfile.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(SipProfile).where(SipProfile.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    rows=r.scalars().all()
    data=[]
    for p in rows:
        d=_ser(p)
        d["_links"]={"self":{"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}
        data.append(d)
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/sip_profiles?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_profile(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_add"))):
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
    d=_ser(p)
    d["_links"]={"self":{"href": f"/api/v1/sip_profiles/{p.sip_profile_uuid}"}}
    return d


@router.patch("/{sip_profile_uuid}")
async def patch_profile(sip_profile_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    p = r.scalar_one_or_none()
    if not p:
        raise HTTPException(status_code=404, detail="SIP profile not found")
    for k in ["name", "sip_port", "sip_ip", "ext_sip_ip", "ext_rtp_ip", "realm", "context", "challenge_realm", "enabled", "wss_port", "wss_binding", "tls_cert_path", "tls_key_path", "rtp_ip", "apply_inbound_acl", "local_network_acl", "description", "hostname"]:
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
async def delete_profile(sip_profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_delete"))):
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


# --- sip_profile_domains ---
@router.get("/{sip_profile_uuid}/domains")
async def list_profile_domains(sip_profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # verify parent exists
    rp = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    if not rp.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="SIP profile not found")
    q = await session.execute(select(SipProfileDomain).where(SipProfileDomain.sip_profile_uuid == sip_profile_uuid, SipProfileDomain.domain_id == domain_id).order_by(SipProfileDomain.domain_name))
    rows = q.scalars().all()
    return {"data": [_ser(o) for o in rows], "total": len(rows)}


@router.post("/{sip_profile_uuid}/domains", status_code=201)
async def create_profile_domain(sip_profile_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_domain_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    rp = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    if not rp.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="SIP profile not found")
    allowed = {c.name for c in SipProfileDomain.__table__.columns} - {"sip_profile_domain_uuid", "created_at", "updated_at", "domain_id", "sip_profile_uuid"}
    data = {k: v for k, v in payload.items() if k in allowed}
    if not data.get("domain_name"):
        raise HTTPException(status_code=422, detail="domain_name required")
    data["domain_id"] = domain_id
    data["sip_profile_uuid"] = sip_profile_uuid
    obj = SipProfileDomain(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/sip_profiles/{sip_profile_uuid}/domains/{obj.sip_profile_domain_uuid}"
    return _ser(obj)


@router.patch("/{sip_profile_uuid}/domains/{domain_uuid}")
async def patch_profile_domain(sip_profile_uuid: UUID, domain_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_domain_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfileDomain).where(SipProfileDomain.sip_profile_domain_uuid == domain_uuid, SipProfileDomain.sip_profile_uuid == sip_profile_uuid, SipProfileDomain.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="SIP profile domain not found")
    allowed = {c.name for c in SipProfileDomain.__table__.columns} - {"sip_profile_domain_uuid", "created_at", "updated_at", "domain_id", "sip_profile_uuid"}
    for k in allowed:
        if k in payload:
            setattr(obj, k, payload[k])
    await session.commit()
    await session.refresh(obj)
    return _ser(obj)


@router.delete("/{sip_profile_uuid}/domains/{domain_uuid}", status_code=204)
async def delete_profile_domain(sip_profile_uuid: UUID, domain_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_domain_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfileDomain).where(SipProfileDomain.sip_profile_domain_uuid == domain_uuid, SipProfileDomain.sip_profile_uuid == sip_profile_uuid, SipProfileDomain.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="SIP profile domain not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


# --- sip_profile_settings ---
@router.get("/{sip_profile_uuid}/settings")
async def list_profile_settings(sip_profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    rp = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    if not rp.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="SIP profile not found")
    q = await session.execute(select(SipProfileSetting).where(SipProfileSetting.sip_profile_uuid == sip_profile_uuid, SipProfileSetting.domain_id == domain_id).order_by(SipProfileSetting.setting_name))
    rows = q.scalars().all()
    return {"data": [_ser(o) for o in rows], "total": len(rows)}


@router.post("/{sip_profile_uuid}/settings", status_code=201)
async def create_profile_setting(sip_profile_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_setting_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    rp = await session.execute(select(SipProfile).where(SipProfile.sip_profile_uuid == sip_profile_uuid, SipProfile.domain_id == domain_id))
    if not rp.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="SIP profile not found")
    allowed = {c.name for c in SipProfileSetting.__table__.columns} - {"sip_profile_setting_uuid", "created_at", "updated_at", "domain_id", "sip_profile_uuid"}
    data = {k: v for k, v in payload.items() if k in allowed}
    if not data.get("setting_name"):
        raise HTTPException(status_code=422, detail="setting_name required")
    data["domain_id"] = domain_id
    data["sip_profile_uuid"] = sip_profile_uuid
    obj = SipProfileSetting(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/sip_profiles/{sip_profile_uuid}/settings/{obj.sip_profile_setting_uuid}"
    return _ser(obj)


@router.patch("/{sip_profile_uuid}/settings/{setting_uuid}")
async def patch_profile_setting(sip_profile_uuid: UUID, setting_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_setting_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfileSetting).where(SipProfileSetting.sip_profile_setting_uuid == setting_uuid, SipProfileSetting.sip_profile_uuid == sip_profile_uuid, SipProfileSetting.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="SIP profile setting not found")
    allowed = {c.name for c in SipProfileSetting.__table__.columns} - {"sip_profile_setting_uuid", "created_at", "updated_at", "domain_id", "sip_profile_uuid"}
    for k in allowed:
        if k in payload:
            setattr(obj, k, payload[k])
    await session.commit()
    await session.refresh(obj)
    return _ser(obj)


@router.delete("/{sip_profile_uuid}/settings/{setting_uuid}", status_code=204)
async def delete_profile_setting(sip_profile_uuid: UUID, setting_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("sip_profile_setting_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(SipProfileSetting).where(SipProfileSetting.sip_profile_setting_uuid == setting_uuid, SipProfileSetting.sip_profile_uuid == sip_profile_uuid, SipProfileSetting.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="SIP profile setting not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)
