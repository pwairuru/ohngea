from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.dialplans.models import Dialplan, DialplanDetail
from app.freeswitch.cache import invalidate_fs_cache_for_session

router = APIRouter(prefix="/api/v1/dialplans", tags=["dialplans"])


@router.get("", dependencies=[Depends(require_permission("dialplan_view"))])
async def list_dialplans(
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    per_page = min(per_page, 100)
    offset = (page - 1) * per_page
    total = (await session.execute(select(func.count()).select_from(select(Dialplan).where(Dialplan.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(Dialplan).where(Dialplan.domain_id == domain_id).order_by(Dialplan.name).offset(offset).limit(per_page))
    items = r.scalars().all()
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/dialplans?page={page+1}&per_page={per_page}>; rel="next"'
    data = []
    for d in items:
        det = await session.execute(select(DialplanDetail).where(DialplanDetail.dialplan_uuid == d.dialplan_uuid).order_by(DialplanDetail.dialplan_detail_group, DialplanDetail.order))
        details = [{"tag": x.tag, "type": x.type, "data": x.data, "order": x.order, "dialplan_detail_break": x.dialplan_detail_break, "dialplan_detail_inline": x.dialplan_detail_inline, "dialplan_detail_group": x.dialplan_detail_group, "detail_enabled": x.detail_enabled} for x in det.scalars().all()]
        data.append({"dialplan_uuid": str(d.dialplan_uuid), "name": d.name, "context": d.context, "enabled": d.enabled, "dialplan_number": d.dialplan_number, "dialplan_destination": d.dialplan_destination, "dialplan_continue": d.dialplan_continue, "dialplan_order": d.dialplan_order, "dialplan_xml": d.dialplan_xml, "details": details, "_links": {"self": {"href": f"/api/v1/dialplans/{d.dialplan_uuid}"}}})
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_dialplan(
    payload: dict,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("dialplan_add")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # payload expects name, context, enabled, details[] — support legacy aliases
    name = payload.get("name") or payload.get("dialplan_name")
    context = payload.get("context") or payload.get("dialplan_context") or f"{domain_id}"
    if not name:
        raise HTTPException(status_code=400, detail="name required")
    enabled_val = payload.get("enabled", payload.get("dialplan_enabled", True))
    description_val = payload.get("description", payload.get("dialplan_description"))
    d = Dialplan(
        name=name,
        context=context,
        enabled=enabled_val if enabled_val is not None else True,
        domain_id=domain_id,
        description=description_val,
        dialplan_continue=payload.get("dialplan_continue", payload.get("continue", False)),
        dialplan_order=payload.get("dialplan_order", payload.get("order", 100)),
        hostname=payload.get("hostname"),
        app_uuid=payload.get("app_uuid"),
        dialplan_number=payload.get("dialplan_number"),
        dialplan_destination=payload.get("dialplan_destination"),
        dialplan_xml=payload.get("dialplan_xml"),
    )
    session.add(d)
    await session.flush()
    for idx, det in enumerate(payload.get("details", [])):
        dd = DialplanDetail(
            dialplan_uuid=d.dialplan_uuid,
            domain_id=det.get("domain_id", domain_id),
            tag=det.get("tag", det.get("dialplan_detail_tag", "condition")),
            type=det.get("type", det.get("dialplan_detail_type", "")),
            data=det.get("data", det.get("dialplan_detail_data", "")),
            order=det.get("order", det.get("dialplan_detail_order", idx)),
            dialplan_detail_break=det.get("dialplan_detail_break", det.get("break")),
            dialplan_detail_inline=det.get("dialplan_detail_inline", det.get("inline")),
            dialplan_detail_group=det.get("dialplan_detail_group", det.get("group", idx // 10)),
            detail_enabled=det.get("detail_enabled", det.get("dialplan_detail_enabled", True)),
        )
        session.add(dd)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(d)
    response.headers["Location"] = f"/api/v1/dialplans/{d.dialplan_uuid}"
    # fetch details for return
    r = await session.execute(select(DialplanDetail).where(DialplanDetail.dialplan_uuid == d.dialplan_uuid).order_by(DialplanDetail.dialplan_detail_group, DialplanDetail.order))
    details = r.scalars().all()
    return {"dialplan_uuid": str(d.dialplan_uuid), "name": d.name, "context": d.context, "enabled": d.enabled, "dialplan_number": d.dialplan_number, "dialplan_destination": d.dialplan_destination, "dialplan_continue": d.dialplan_continue, "dialplan_order": d.dialplan_order, "details": [{"tag": x.tag, "type": x.type, "data": x.data, "order": x.order, "dialplan_detail_break": x.dialplan_detail_break, "dialplan_detail_inline": x.dialplan_detail_inline, "dialplan_detail_group": x.dialplan_detail_group, "detail_enabled": x.detail_enabled} for x in details], "_links": {"self": {"href": f"/api/v1/dialplans/{d.dialplan_uuid}"}}}


@router.get("/{dialplan_uuid}", dependencies=[Depends(require_permission("dialplan_view"))])
async def get_dialplan(dialplan_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Dialplan).where(Dialplan.dialplan_uuid == dialplan_uuid, Dialplan.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Dialplan not found")
    det = await session.execute(select(DialplanDetail).where(DialplanDetail.dialplan_uuid == d.dialplan_uuid).order_by(DialplanDetail.dialplan_detail_group, DialplanDetail.order))
    details = [{"tag": x.tag, "type": x.type, "data": x.data, "order": x.order, "dialplan_detail_break": x.dialplan_detail_break, "dialplan_detail_inline": x.dialplan_detail_inline, "dialplan_detail_group": x.dialplan_detail_group, "detail_enabled": x.detail_enabled, "domain_id": str(x.domain_id) if x.domain_id else None} for x in det.scalars().all()]
    return {"dialplan_uuid": str(d.dialplan_uuid), "name": d.name, "context": d.context, "enabled": d.enabled, "dialplan_number": d.dialplan_number, "dialplan_destination": d.dialplan_destination, "dialplan_continue": d.dialplan_continue, "dialplan_order": d.dialplan_order, "dialplan_xml": d.dialplan_xml, "hostname": d.hostname, "app_uuid": d.app_uuid, "details": details, "_links": {"self": {"href": f"/api/v1/dialplans/{d.dialplan_uuid}"}}}


@router.patch("/{dialplan_uuid}")
async def patch_dialplan(dialplan_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("dialplan_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Dialplan).where(Dialplan.dialplan_uuid == dialplan_uuid, Dialplan.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Dialplan not found")
    for k in ["name", "context", "enabled", "description", "dialplan_continue", "dialplan_order", "hostname", "app_uuid", "dialplan_number", "dialplan_destination", "dialplan_xml"]:
        if k in payload:
            setattr(d, k, payload[k])
    # handle legacy aliases
    if "dialplan_name" in payload and "name" not in payload:
        d.name = payload["dialplan_name"]
    if "dialplan_context" in payload and "context" not in payload:
        d.context = payload["dialplan_context"]
    if "dialplan_enabled" in payload and "enabled" not in payload:
        d.enabled = bool(payload["dialplan_enabled"])
    if "dialplan_description" in payload and "description" not in payload:
        d.description = payload["dialplan_description"]
    # handle alias continue/order
    if "continue" in payload:
        d.dialplan_continue = bool(payload["continue"])
    if "order" in payload and "dialplan_order" not in payload:
        d.dialplan_order = payload["order"]
    # handle details update if provided: replace all
    if "details" in payload and isinstance(payload["details"], list):
        # delete existing
        existing = await session.execute(select(DialplanDetail).where(DialplanDetail.dialplan_uuid == d.dialplan_uuid))
        for old in existing.scalars().all():
            await session.delete(old)
        await session.flush()
        for idx, det in enumerate(payload["details"]):
            dd = DialplanDetail(
                dialplan_uuid=d.dialplan_uuid,
                domain_id=det.get("domain_id", domain_id),
                tag=det.get("tag", det.get("dialplan_detail_tag", "condition")),
                type=det.get("type", det.get("dialplan_detail_type", "")),
                data=det.get("data", det.get("dialplan_detail_data", "")),
                order=det.get("order", det.get("dialplan_detail_order", idx)),
                dialplan_detail_break=det.get("dialplan_detail_break", det.get("break")),
                dialplan_detail_inline=det.get("dialplan_detail_inline", det.get("inline")),
                dialplan_detail_group=det.get("dialplan_detail_group", det.get("group", idx // 10)),
                detail_enabled=det.get("detail_enabled", det.get("dialplan_detail_enabled", True)),
            )
            session.add(dd)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(d)
    return {"dialplan_uuid": str(d.dialplan_uuid), "name": d.name, "context": d.context, "enabled": d.enabled}


@router.delete("/{dialplan_uuid}", status_code=204)
async def delete_dialplan(dialplan_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("dialplan_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Dialplan).where(Dialplan.dialplan_uuid == dialplan_uuid, Dialplan.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Dialplan not found")
    await session.delete(d)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)
