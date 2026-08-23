from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Response, Request
from fastapi.responses import PlainTextResponse
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.devices.models import Device
from app.devices.provision import render_provision

router = APIRouter(prefix="/api/v1/devices", tags=["devices"])
provision_router = APIRouter(tags=["provision"])


@router.get("")
async def list_devices(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(Device).where(Device.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(Device).where(Device.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    data = [{"device_uuid": str(d.device_uuid), "mac": d.mac, "vendor": d.vendor, "model": d.model, "label": d.label, "enabled": d.enabled, "_links": {"self": {"href": f"/api/v1/devices/{d.device_uuid}"}}} for d in r.scalars().all()]
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/devices?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_device(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("dialplan_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # normalize mac
    mac = payload.get("mac", "").lower()
    vendor = payload.get("vendor", "poly")
    d = Device(domain_id=domain_id, mac=mac, vendor=vendor, model=payload.get("model"), label=payload.get("label"), enabled=payload.get("enabled", True), extension_uuid=payload.get("extension_uuid"))
    session.add(d)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(d)
    response.headers["Location"] = f"/api/v1/devices/{d.device_uuid}"
    return {"device_uuid": str(d.device_uuid), "mac": d.mac, "vendor": d.vendor, "_links": {"self": {"href": f"/api/v1/devices/{d.device_uuid}"}}}


@router.get("/{device_uuid}")
async def get_device(device_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Device not found")
    return {"device_uuid": str(d.device_uuid), "mac": d.mac, "vendor": d.vendor, "model": d.model, "label": d.label, "enabled": d.enabled, "_links": {"self": {"href": f"/api/v1/devices/{d.device_uuid}"}}}


@router.patch("/{device_uuid}")
async def patch_device(device_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("dialplan_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Device not found")
    for k in ["mac", "vendor", "model", "label", "enabled", "extension_uuid"]:
        if k in payload:
            # normalize mac lower
            if k == "mac" and payload[k]:
                payload[k] = payload[k].lower()
            setattr(d, k, payload[k])
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(d)
    return {"device_uuid": str(d.device_uuid), "mac": d.mac, "vendor": d.vendor, "model": d.model, "label": d.label, "enabled": d.enabled, "_links": {"self": {"href": f"/api/v1/devices/{d.device_uuid}"}}}


@router.delete("/{device_uuid}", status_code=204)
async def delete_device(device_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("dialplan_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Device not found")
    await session.delete(d)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)


@provision_router.get("/provision/{mac}.cfg", response_class=PlainTextResponse)
async def get_provision(mac: str, session: AsyncSession = Depends(get_session)):
    # provision is unauthenticated but scoped to device lookup; for now render generic
    # try find device by mac
    from sqlalchemy import select
    r = await session.execute(select(Device).where(Device.mac.ilike(mac)))
    device = r.scalar_one_or_none()
    ctx = {"mac": mac, "vendor": device.vendor if device else "poly", "extension": "", "domain": ""}
    if device:
        # fetch extension if linked
        if device.extension_uuid:
            from app.extensions.models import Extension
            re = await session.execute(select(Extension).where(Extension.extension_uuid == device.extension_uuid))
            ext = re.scalar_one_or_none()
            if ext:
                ctx["extension"] = ext.extension
                ctx["domain"] = str(ext.domain_id)
                # fetch domain name
                from app.domains.models import Domain
                rd = await session.execute(select(Domain).where(Domain.domain_id == ext.domain_id))
                dom = rd.scalar_one_or_none()
                if dom:
                    ctx["domain"] = dom.domain_name
    out = render_provision(ctx["vendor"], ctx)
    return PlainTextResponse(out, media_type="text/plain")
