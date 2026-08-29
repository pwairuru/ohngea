from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.ivr_menus.models import IvrMenu, IvrMenuOption

router = APIRouter(prefix="/api/v1/ivr_menus", tags=["ivr_menus"])


@router.get("")
async def list_ivr(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(IvrMenu).where(IvrMenu.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(IvrMenu).where(IvrMenu.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    data = [{"ivr_menu_uuid": str(m.ivr_menu_uuid), "name": m.name, "extension": m.extension, "enabled": m.enabled, "_links": {"self": {"href": f"/api/v1/ivr_menus/{m.ivr_menu_uuid}"}}} for m in r.scalars().all()]
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/ivr_menus?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_ivr(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("ivr_menu_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    allowed={c.name for c in IvrMenu.__table__.columns} - {"ivr_menu_uuid","created_at","updated_at","domain_id"}
    data={k:v for k,v in payload.items() if k in allowed}
    data["domain_id"]=domain_id
    if not data.get("name"):
        raise HTTPException(status_code=422, detail="name required")
    m = IvrMenu(**data)
    session.add(m)
    await session.flush()
    for idx, opt in enumerate(payload.get("options", [])):
        o = IvrMenuOption(ivr_menu_uuid=m.ivr_menu_uuid, digits=opt.get("digits"), action=opt.get("action"), param=opt.get("param"), order=idx)
        session.add(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(m)
    response.headers["Location"] = f"/api/v1/ivr_menus/{m.ivr_menu_uuid}"
    return {"ivr_menu_uuid": str(m.ivr_menu_uuid), "name": m.name, "_links": {"self": {"href": f"/api/v1/ivr_menus/{m.ivr_menu_uuid}"}}}


@router.get("/{ivr_menu_uuid}")
async def get_ivr(ivr_menu_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(IvrMenu).where(IvrMenu.ivr_menu_uuid == ivr_menu_uuid, IvrMenu.domain_id == domain_id))
    m = r.scalar_one_or_none()
    if not m:
        raise HTTPException(status_code=404, detail="IVR not found")
    opts = await session.execute(select(IvrMenuOption).where(IvrMenuOption.ivr_menu_uuid == m.ivr_menu_uuid).order_by(IvrMenuOption.order))
    options = [{"digits": o.digits, "action": o.action, "param": o.param} for o in opts.scalars().all()]
    return {"ivr_menu_uuid": str(m.ivr_menu_uuid), "name": m.name, "extension": m.extension, "enabled": m.enabled, "options": options, "_links": {"self": {"href": f"/api/v1/ivr_menus/{m.ivr_menu_uuid}"}}}


@router.patch("/{ivr_menu_uuid}")
async def patch_ivr(ivr_menu_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("ivr_menu_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(IvrMenu).where(IvrMenu.ivr_menu_uuid == ivr_menu_uuid, IvrMenu.domain_id == domain_id))
    m = r.scalar_one_or_none()
    if not m:
        raise HTTPException(status_code=404, detail="IVR not found")
    # allow all IvrMenu columns (dialplan_uuid, greet_long etc.) via table.columns detection like gateways router
    allowed = {c.name for c in IvrMenu.__table__.columns} - {"ivr_menu_uuid", "created_at", "updated_at", "domain_id"}
    for k in allowed:
        if k in payload:
            setattr(m, k, payload[k])
    if "options" in payload and isinstance(payload["options"], list):
        # replace options
        old = await session.execute(select(IvrMenuOption).where(IvrMenuOption.ivr_menu_uuid == m.ivr_menu_uuid))
        for o in old.scalars().all():
            await session.delete(o)
        for idx, opt in enumerate(payload["options"]):
            o = IvrMenuOption(ivr_menu_uuid=m.ivr_menu_uuid, digits=opt.get("digits"), action=opt.get("action"), param=opt.get("param"), order=idx)
            session.add(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(m)
    return {"ivr_menu_uuid": str(m.ivr_menu_uuid), "name": m.name, "extension": m.extension, "enabled": m.enabled, "_links": {"self": {"href": f"/api/v1/ivr_menus/{m.ivr_menu_uuid}"}}}


@router.delete("/{ivr_menu_uuid}", status_code=204)
async def delete_ivr(ivr_menu_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("ivr_menu_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(IvrMenu).where(IvrMenu.ivr_menu_uuid == ivr_menu_uuid, IvrMenu.domain_id == domain_id))
    m = r.scalar_one_or_none()
    if not m:
        raise HTTPException(status_code=404, detail="IVR not found")
    await session.delete(m)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)
