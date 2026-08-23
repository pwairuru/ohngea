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
    rows=r.scalars().all()
    data=[]
    for d in rows:
        dd={c.name: getattr(d,c.name) for c in d.__table__.columns}
        for k in ["device_uuid","domain_id","extension_uuid"]:
            if dd.get(k) is not None:
                dd[k]=str(dd[k])
        for k in ["created_at","updated_at"]:
            if dd.get(k):
                try:
                    dd[k]=dd[k].isoformat()
                except Exception:
                    pass
        dd["_links"]={"self":{"href": f"/api/v1/devices/{d.device_uuid}"}}
        data.append(dd)
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/devices?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_device(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("dialplan_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # normalize mac and build data from allowed columns
    allowed={c.name for c in Device.__table__.columns} - {"device_uuid","created_at","updated_at","domain_id"}
    data={k:v for k,v in payload.items() if k in allowed}
    if "mac" in data and data["mac"]:
        data["mac"]=data["mac"].lower()
    data["domain_id"]=domain_id
    if not data.get("mac"):
        raise HTTPException(status_code=422, detail="mac is required")
    if not data.get("vendor"):
        data["vendor"]="poly"
    # handle aliases: device_address -> mac, device_template etc already allowed
    # ensure extension_uuid is UUID if provided
    if "extension_uuid" in data and data["extension_uuid"] and isinstance(data["extension_uuid"], str):
        try:
            from uuid import UUID
            data["extension_uuid"]=UUID(data["extension_uuid"])
        except Exception:
            pass
    d = Device(**data)
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
    dd={c.name: getattr(d,c.name) for c in d.__table__.columns}
    for k in ["device_uuid","domain_id","extension_uuid"]:
        if dd.get(k) is not None:
            dd[k]=str(dd[k])
    for k in ["created_at","updated_at"]:
        if dd.get(k):
            try:
                dd[k]=dd[k].isoformat()
            except Exception:
                pass
    dd["_links"]={"self":{"href": f"/api/v1/devices/{d.device_uuid}"}}
    return dd


@router.patch("/{device_uuid}")
async def patch_device(device_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("dialplan_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Device not found")
    allowed={c.name for c in Device.__table__.columns} - {"device_uuid","created_at","updated_at","domain_id"}
    for k in allowed:
        if k in payload:
            v=payload[k]
            if k=="mac" and v:
                v=v.lower()
            if k=="extension_uuid" and v and isinstance(v, str):
                try:
                    from uuid import UUID
                    v=UUID(v)
                except Exception:
                    pass
            setattr(d, k, v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(d)
    dd={c.name: getattr(d,c.name) for c in d.__table__.columns}
    for k in ["device_uuid","domain_id","extension_uuid"]:
        if dd.get(k) is not None:
            dd[k]=str(dd[k])
    for k in ["created_at","updated_at"]:
        if dd.get(k):
            try:
                dd[k]=dd[k].isoformat()
            except Exception:
                pass
    dd["_links"]={"self":{"href": f"/api/v1/devices/{d.device_uuid}"}}
    return dd


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
