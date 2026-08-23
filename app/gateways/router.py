from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.gateways.models import Gateway


async def _sofia_rescan(profile: str | None):
    """Best-effort ESL sofia profile rescan, mirrors legacy gateway_edit.php."""
    if not profile:
        profile = "external"
    try:
        from app.freeswitch.esl_client import esl_client

        await esl_client.sofia_rescan(profile)
    except Exception:
        pass

router = APIRouter(prefix="/api/v1/gateways", tags=["gateways"])


@router.get("", dependencies=[Depends(require_permission("gateway_view"))])
async def list_gateways(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(Gateway).where(Gateway.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(Gateway).where(Gateway.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    rows = r.scalars().all()
    data = []
    for g in rows:
        d = {c.name: getattr(g, c.name) for c in g.__table__.columns}
        # convert UUIDs to str
        for k in ["gateway_uuid","domain_id"]:
            if k in d and d[k] is not None:
                d[k] = str(d[k])
        # datetime isoformat
        for k in ["created_at","updated_at"]:
            if k in d and d[k] is not None:
                try:
                    d[k] = d[k].isoformat()
                except Exception:
                    pass
        d["_links"] = {"self": {"href": f"/api/v1/gateways/{g.gateway_uuid}"}}
        data.append(d)
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/gateways?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_gateway(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    if not payload.get("gateway"):
        raise HTTPException(status_code=422, detail="gateway is required")
    # filter to model columns
    allowed = {c.name for c in Gateway.__table__.columns} - {"gateway_uuid","created_at","updated_at","domain_id"}
    data = {k: v for k, v in payload.items() if k in allowed}
    # translate "register" payload key to gateway_register field (DB column is "register")
    if "register" in data:
        data["gateway_register"] = data.pop("register")
    data["domain_id"] = domain_id
    # defaults for new fields if missing
    data.setdefault("expire_seconds", 800)
    data.setdefault("retry_seconds", 30)
    data.setdefault("context", "public")
    data.setdefault("profile", "external")
    gw = Gateway(**data)
    session.add(gw)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    # ESL sofia rescan for profile (legacy: sofia profile external rescan)
    await _sofia_rescan(getattr(gw, "profile", None) or data.get("profile") or "external")
    await session.refresh(gw)
    response.headers["Location"] = f"/api/v1/gateways/{gw.gateway_uuid}"
    # return full
    d = {c.name: getattr(gw, c.name) for c in gw.__table__.columns}
    for k in ["gateway_uuid","domain_id"]:
        d[k] = str(d[k])
    for k in ["created_at","updated_at"]:
        if d[k]:
            d[k] = d[k].isoformat()
    d["_links"] = {"self": {"href": f"/api/v1/gateways/{gw.gateway_uuid}"}}
    return d


@router.get("/{gateway_uuid}", dependencies=[Depends(require_permission("gateway_view"))])
async def get_gateway(gateway_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Gateway).where(Gateway.gateway_uuid == gateway_uuid, Gateway.domain_id == domain_id))
    g = r.scalar_one_or_none()
    if not g:
        raise HTTPException(status_code=404, detail="Gateway not found")
    d = {c.name: getattr(g, c.name) for c in g.__table__.columns}
    for k in ["gateway_uuid","domain_id"]:
        d[k] = str(d[k])
    for k in ["created_at","updated_at"]:
        if d[k]:
            d[k] = d[k].isoformat()
    d["_links"] = {"self": {"href": f"/api/v1/gateways/{g.gateway_uuid}"}}
    return d


@router.patch("/{gateway_uuid}")
async def patch_gateway(gateway_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Gateway).where(Gateway.gateway_uuid == gateway_uuid, Gateway.domain_id == domain_id))
    g = r.scalar_one_or_none()
    if not g:
        raise HTTPException(status_code=404, detail="Gateway not found")
    allowed = {c.name for c in Gateway.__table__.columns} - {"gateway_uuid","created_at","updated_at","domain_id"}
    # handle register translation
    if "register" in payload and "register" in allowed:
        g.gateway_register = bool(payload["register"])
    for k in allowed:
        if k == "register":
            continue
        if k in payload:
            setattr(g, k, payload[k])
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await _sofia_rescan(getattr(g, "profile", None) or "external")
    await session.refresh(g)
    d = {c.name: getattr(g, c.name) for c in g.__table__.columns}
    for k in ["gateway_uuid","domain_id"]:
        d[k] = str(d[k])
    for k in ["created_at","updated_at"]:
        if d[k]:
            d[k] = d[k].isoformat()
    d["_links"] = {"self": {"href": f"/api/v1/gateways/{g.gateway_uuid}"}}
    return d


@router.delete("/{gateway_uuid}", status_code=204)
async def delete_gateway(gateway_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Gateway).where(Gateway.gateway_uuid == gateway_uuid, Gateway.domain_id == domain_id))
    g = r.scalar_one_or_none()
    if not g:
        raise HTTPException(status_code=404, detail="Gateway not found")
    # capture profile before delete for rescan
    _profile = getattr(g, "profile", None)
    await session.delete(g)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await _sofia_rescan(_profile or "external")
    return Response(status_code=204)
