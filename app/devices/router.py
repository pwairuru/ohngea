from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Response, Request
from fastapi.responses import PlainTextResponse
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.devices.models import Device, DeviceKey, DeviceSetting, DeviceProfile, DeviceProfileKey, DeviceProfileSetting
from app.devices.provision import render_provision

router = APIRouter(prefix="/api/v1/devices", tags=["devices"])
provision_router = APIRouter(tags=["provision"])
profile_router = APIRouter(prefix="/api/v1/device_profiles", tags=["device_profiles"])


def _serialize(obj):
    dd = {c.name: getattr(obj, c.name) for c in obj.__table__.columns}
    for k in ["device_uuid", "domain_id", "extension_uuid", "device_profile_uuid", "device_key_uuid", "device_setting_uuid", "device_line_uuid", "device_profile_key_uuid", "device_profile_setting_uuid"]:
        if dd.get(k) is not None:
            dd[k] = str(dd[k])
    for k in ["created_at", "updated_at"]:
        if dd.get(k):
            try:
                dd[k] = dd[k].isoformat()
            except Exception:
                pass
    return dd


@router.get("", dependencies=[Depends(require_permission("device_view"))])
async def list_devices(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(Device).where(Device.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(Device).where(Device.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    rows=r.scalars().all()
    data=[]
    for d in rows:
        dd=_serialize(d)
        dd["_links"]={"self":{"href": f"/api/v1/devices/{d.device_uuid}"}}
        data.append(dd)
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/devices?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_device(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    allowed={c.name for c in Device.__table__.columns} - {"device_uuid","created_at","updated_at","domain_id"}
    data={k:v for k,v in payload.items() if k in allowed}
    if "mac" in data and data["mac"]:
        data["mac"]=data["mac"].lower()
    data["domain_id"]=domain_id
    if not data.get("mac"):
        raise HTTPException(status_code=422, detail="mac is required")
    if not data.get("vendor"):
        data["vendor"]="poly"
    if "extension_uuid" in data and data["extension_uuid"] and isinstance(data["extension_uuid"], str):
        try:
            from uuid import UUID
            data["extension_uuid"]=UUID(data["extension_uuid"])
        except Exception:
            pass
    if "device_profile_uuid" in data and data["device_profile_uuid"] and isinstance(data["device_profile_uuid"], str):
        try:
            data["device_profile_uuid"]=UUID(data["device_profile_uuid"])
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


@router.get("/{device_uuid}", dependencies=[Depends(require_permission("device_view"))])
async def get_device(device_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Device not found")
    dd=_serialize(d)
    dd["_links"]={"self":{"href": f"/api/v1/devices/{d.device_uuid}"}}
    return dd


@router.patch("/{device_uuid}")
async def patch_device(device_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_edit"))):
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
            if k in ("extension_uuid","device_profile_uuid") and v and isinstance(v, str):
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
    dd=_serialize(d)
    dd["_links"]={"self":{"href": f"/api/v1/devices/{d.device_uuid}"}}
    return dd


@router.delete("/{device_uuid}", status_code=204)
async def delete_device(device_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_delete"))):
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


# --- device keys CRUD ---
@router.get("/{device_uuid}/keys", dependencies=[Depends(require_permission("device_view"))])
async def list_device_keys(device_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # verify device exists and belongs to domain
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device not found")
    q = await session.execute(select(DeviceKey).where(DeviceKey.device_uuid == device_uuid, DeviceKey.domain_id == domain_id).order_by(DeviceKey.device_key_id))
    rows = q.scalars().all()
    return {"data": [_serialize(o) for o in rows], "total": len(rows)}


@router.post("/{device_uuid}/keys", status_code=201)
async def create_device_key(device_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device not found")
    allowed={c.name for c in DeviceKey.__table__.columns} - {"device_key_uuid","created_at","updated_at","domain_id","device_uuid"}
    data={k:v for k,v in payload.items() if k in allowed}
    data["domain_id"]=domain_id
    data["device_uuid"]=device_uuid
    obj = DeviceKey(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/devices/{device_uuid}/keys/{obj.device_key_uuid}"
    return _serialize(obj)


@router.patch("/{device_uuid}/keys/{key_uuid}")
async def patch_device_key(device_uuid: UUID, key_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceKey).where(DeviceKey.device_key_uuid == key_uuid, DeviceKey.device_uuid == device_uuid, DeviceKey.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device key not found")
    allowed={c.name for c in DeviceKey.__table__.columns} - {"device_key_uuid","created_at","updated_at","domain_id","device_uuid"}
    for k in allowed:
        if k in payload:
            setattr(obj, k, payload[k])
    await session.commit()
    await session.refresh(obj)
    return _serialize(obj)


@router.delete("/{device_uuid}/keys/{key_uuid}", status_code=204)
async def delete_device_key(device_uuid: UUID, key_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceKey).where(DeviceKey.device_key_uuid == key_uuid, DeviceKey.device_uuid == device_uuid, DeviceKey.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device key not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


# --- device settings CRUD ---
@router.get("/{device_uuid}/settings", dependencies=[Depends(require_permission("device_view"))])
async def list_device_settings(device_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device not found")
    q = await session.execute(select(DeviceSetting).where(DeviceSetting.device_uuid == device_uuid, DeviceSetting.domain_id == domain_id).order_by(DeviceSetting.device_setting_name))
    rows = q.scalars().all()
    return {"data": [_serialize(o) for o in rows], "total": len(rows)}


@router.post("/{device_uuid}/settings", status_code=201)
async def create_device_setting(device_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Device).where(Device.device_uuid == device_uuid, Device.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device not found")
    allowed={c.name for c in DeviceSetting.__table__.columns} - {"device_setting_uuid","created_at","updated_at","domain_id","device_uuid"}
    data={k:v for k,v in payload.items() if k in allowed}
    if not data.get("device_setting_name"):
        raise HTTPException(status_code=422, detail="device_setting_name required")
    data["domain_id"]=domain_id
    data["device_uuid"]=device_uuid
    obj = DeviceSetting(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/devices/{device_uuid}/settings/{obj.device_setting_uuid}"
    return _serialize(obj)


@router.patch("/{device_uuid}/settings/{setting_uuid}")
async def patch_device_setting(device_uuid: UUID, setting_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceSetting).where(DeviceSetting.device_setting_uuid == setting_uuid, DeviceSetting.device_uuid == device_uuid, DeviceSetting.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device setting not found")
    allowed={c.name for c in DeviceSetting.__table__.columns} - {"device_setting_uuid","created_at","updated_at","domain_id","device_uuid"}
    for k in allowed:
        if k in payload:
            setattr(obj, k, payload[k])
    await session.commit()
    await session.refresh(obj)
    return _serialize(obj)


@router.delete("/{device_uuid}/settings/{setting_uuid}", status_code=204)
async def delete_device_setting(device_uuid: UUID, setting_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceSetting).where(DeviceSetting.device_setting_uuid == setting_uuid, DeviceSetting.device_uuid == device_uuid, DeviceSetting.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device setting not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


# --- device_profiles CRUD (top-level) ---
@profile_router.get("", dependencies=[Depends(require_permission("device_view"))])
async def list_device_profiles(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(DeviceProfile).where(DeviceProfile.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    rows = r.scalars().all()
    data = [_serialize(o) for o in rows]
    for d in data:
        d["_links"] = {"self": {"href": f"/api/v1/device_profiles/{d['device_profile_uuid']}"}}
    response.headers["X-Total-Count"] = str(total)
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@profile_router.post("", status_code=201)
async def create_device_profile(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    allowed={c.name for c in DeviceProfile.__table__.columns} - {"device_profile_uuid","created_at","updated_at","domain_id"}
    data={k:v for k,v in payload.items() if k in allowed}
    if not data.get("device_profile_name"):
        raise HTTPException(status_code=422, detail="device_profile_name required")
    data["domain_id"]=domain_id
    obj = DeviceProfile(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/device_profiles/{obj.device_profile_uuid}"
    return _serialize(obj)


@profile_router.get("/{profile_uuid}", dependencies=[Depends(require_permission("device_view"))])
async def get_device_profile(profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.device_profile_uuid == profile_uuid, DeviceProfile.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device profile not found")
    dd=_serialize(obj)
    dd["_links"]={"self":{"href": f"/api/v1/device_profiles/{profile_uuid}"}}
    return dd


@profile_router.patch("/{profile_uuid}")
async def patch_device_profile(profile_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.device_profile_uuid == profile_uuid, DeviceProfile.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device profile not found")
    allowed={c.name for c in DeviceProfile.__table__.columns} - {"device_profile_uuid","created_at","updated_at","domain_id"}
    for k in allowed:
        if k in payload:
            setattr(obj, k, payload[k])
    await session.commit()
    await session.refresh(obj)
    return _serialize(obj)


@profile_router.delete("/{profile_uuid}", status_code=204)
async def delete_device_profile(profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.device_profile_uuid == profile_uuid, DeviceProfile.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device profile not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


@profile_router.get("/{profile_uuid}/keys", dependencies=[Depends(require_permission("device_view"))])
async def list_device_profile_keys(profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.device_profile_uuid == profile_uuid, DeviceProfile.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device profile not found")
    q = await session.execute(select(DeviceProfileKey).where(DeviceProfileKey.device_profile_uuid == profile_uuid, DeviceProfileKey.domain_id == domain_id).order_by(DeviceProfileKey.profile_key_id))
    rows = q.scalars().all()
    return {"data": [_serialize(o) for o in rows], "total": len(rows)}


@profile_router.post("/{profile_uuid}/keys", status_code=201)
async def create_device_profile_key(profile_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.device_profile_uuid == profile_uuid, DeviceProfile.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device profile not found")
    allowed={c.name for c in DeviceProfileKey.__table__.columns} - {"device_profile_key_uuid","created_at","updated_at","domain_id","device_profile_uuid"}
    data={k:v for k,v in payload.items() if k in allowed}
    data["domain_id"]=domain_id
    data["device_profile_uuid"]=profile_uuid
    obj = DeviceProfileKey(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/device_profiles/{profile_uuid}/keys/{obj.device_profile_key_uuid}"
    return _serialize(obj)


@profile_router.patch("/{profile_uuid}/keys/{key_uuid}")
async def patch_device_profile_key(profile_uuid: UUID, key_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfileKey).where(DeviceProfileKey.device_profile_key_uuid == key_uuid, DeviceProfileKey.device_profile_uuid == profile_uuid, DeviceProfileKey.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device profile key not found")
    allowed={c.name for c in DeviceProfileKey.__table__.columns} - {"device_profile_key_uuid","created_at","updated_at","domain_id","device_profile_uuid"}
    for k in allowed:
        if k in payload:
            setattr(obj, k, payload[k])
    await session.commit()
    await session.refresh(obj)
    return _serialize(obj)


@profile_router.delete("/{profile_uuid}/keys/{key_uuid}", status_code=204)
async def delete_device_profile_key(profile_uuid: UUID, key_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfileKey).where(DeviceProfileKey.device_profile_key_uuid == key_uuid, DeviceProfileKey.device_profile_uuid == profile_uuid, DeviceProfileKey.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device profile key not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


@profile_router.get("/{profile_uuid}/settings", dependencies=[Depends(require_permission("device_view"))])
async def list_device_profile_settings(profile_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.device_profile_uuid == profile_uuid, DeviceProfile.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device profile not found")
    q = await session.execute(select(DeviceProfileSetting).where(DeviceProfileSetting.device_profile_uuid == profile_uuid, DeviceProfileSetting.domain_id == domain_id).order_by(DeviceProfileSetting.profile_setting_name))
    rows = q.scalars().all()
    return {"data": [_serialize(o) for o in rows], "total": len(rows)}


@profile_router.post("/{profile_uuid}/settings", status_code=201)
async def create_device_profile_setting(profile_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfile).where(DeviceProfile.device_profile_uuid == profile_uuid, DeviceProfile.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Device profile not found")
    allowed={c.name for c in DeviceProfileSetting.__table__.columns} - {"device_profile_setting_uuid","created_at","updated_at","domain_id","device_profile_uuid"}
    data={k:v for k,v in payload.items() if k in allowed}
    if not data.get("profile_setting_name"):
        raise HTTPException(status_code=422, detail="profile_setting_name required")
    data["domain_id"]=domain_id
    data["device_profile_uuid"]=profile_uuid
    obj = DeviceProfileSetting(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/device_profiles/{profile_uuid}/settings/{obj.device_profile_setting_uuid}"
    return _serialize(obj)


@profile_router.patch("/{profile_uuid}/settings/{setting_uuid}")
async def patch_device_profile_setting(profile_uuid: UUID, setting_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfileSetting).where(DeviceProfileSetting.device_profile_setting_uuid == setting_uuid, DeviceProfileSetting.device_profile_uuid == profile_uuid, DeviceProfileSetting.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device profile setting not found")
    allowed={c.name for c in DeviceProfileSetting.__table__.columns} - {"device_profile_setting_uuid","created_at","updated_at","domain_id","device_profile_uuid"}
    for k in allowed:
        if k in payload:
            setattr(obj, k, payload[k])
    await session.commit()
    await session.refresh(obj)
    return _serialize(obj)


@profile_router.delete("/{profile_uuid}/settings/{setting_uuid}", status_code=204)
async def delete_device_profile_setting(profile_uuid: UUID, setting_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("device_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(DeviceProfileSetting).where(DeviceProfileSetting.device_profile_setting_uuid == setting_uuid, DeviceProfileSetting.device_profile_uuid == profile_uuid, DeviceProfileSetting.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Device profile setting not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


@provision_router.get("/provision/{mac}.cfg", response_class=PlainTextResponse)
async def get_provision(mac: str, session: AsyncSession = Depends(get_session)):
    # provision is unauthenticated but scoped to device lookup; Jinga collapsed vendor matrix
    from sqlalchemy import select

    r = await session.execute(select(Device).where(Device.mac.ilike(mac)))
    device = r.scalar_one_or_none()
    vendor = (device.vendor if device and device.vendor else "poly")
    model = (device.model if device and device.model else "")
    # prefer device_vendor/device_model legacy aliases if vendor not set
    if device and not vendor and getattr(device, "device_vendor", None):
        vendor = device.device_vendor
    if device and not model and getattr(device, "device_model", None):
        model = device.device_model
    ctx = {"mac": mac, "vendor": vendor, "model": model, "extension": "", "domain": ""}
    if device:
        # include device meta for templates
        ctx["label"] = getattr(device, "label", "") or ""
        ctx["device_template"] = getattr(device, "device_template", "") or ""
        if device.extension_uuid:
            from app.extensions.models import Extension

            re = await session.execute(select(Extension).where(Extension.extension_uuid == device.extension_uuid))
            ext = re.scalar_one_or_none()
            if ext:
                ctx["extension"] = ext.extension
                ctx["domain"] = str(ext.domain_id)
                from app.domains.models import Domain

                rd = await session.execute(select(Domain).where(Domain.domain_id == ext.domain_id))
                dom = rd.scalar_one_or_none()
                if dom:
                    ctx["domain"] = dom.domain_name
                ctx["password"] = getattr(ext, "password", "") or ""
                ctx["effective_caller_id_name"] = getattr(ext, "effective_caller_id_name", "") or ""
    # include normalized mac variants for templates
    ctx["mac_upper"] = mac.upper()
    ctx["mac_lower"] = mac.lower()
    out = render_provision(vendor, ctx, model=model)
    return PlainTextResponse(out, media_type="text/plain")
