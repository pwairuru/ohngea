from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.call_flows.models import CallFlow
from app.call_flows.schemas import CallFlowCreate, CallFlowRead, CallFlowUpdate
from app.call_flows.service import create_call_flow, list_call_flows

router = APIRouter(prefix="/api/v1/call_flows", tags=["call_flows"])


def _links(cf: CallFlow):
    base = f"/api/v1/call_flows/{cf.call_flow_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/call_flows"}}


def _serialize(cf: CallFlow):
    d = CallFlowRead.model_validate(cf, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(cf)
    d["call_flow_uuid"] = str(cf.call_flow_uuid)
    d["call_flow_name"] = cf.call_flow_name
    d["call_flow_extension"] = cf.call_flow_extension
    return d


@router.get("", response_model=dict)
async def list_cf(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("call_flow_name"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_call_flow_extension: Optional[str] = Query(None, alias="filter[call_flow_extension]"),
    filter_enabled: Optional[bool] = Query(None, alias="filter[enabled]"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    ext = filter_call_flow_extension or filter_extension
    items, total = await list_call_flows(session, domain_id, page, per_page, q, sort, order, ext, filter_enabled)
    response.headers["X-Total-Count"] = str(total)
    base_url = str(request.url).split("?")[0]
    if page * per_page < total:
        from urllib.parse import urlencode
        qs = dict(request.query_params)
        qs["page"] = str(page + 1)
        qs["per_page"] = str(per_page)
        response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="next"'
        if page > 1:
            qs["page"] = str(page - 1)
            response.headers["Link"] += f', <{base_url}?{urlencode(qs)}>; rel="prev"'
    else:
        if page > 1:
            from urllib.parse import urlencode
            qs = dict(request.query_params)
            qs["page"] = str(page - 1)
            response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="prev"'
    data = [_serialize(c) for c in items]
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/health", response_model=dict)
async def health():
    return {"status": "ok", "mode": "core", "call_flows": "enabled"}


@router.post("", response_model=dict, status_code=201)
async def create_cf(payload: CallFlowCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_flow_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    if not payload.domain_id:
        payload.domain_id = domain_id
    await set_tenant(session, domain_id)
    cf = await create_call_flow(session, payload)
    response.headers["Location"] = f"/api/v1/call_flows/{cf.call_flow_uuid}"
    return _serialize(cf)


@router.get("/{call_flow_uuid}", response_model=dict)
async def get_cf(call_flow_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallFlow).where(CallFlow.call_flow_uuid == call_flow_uuid, CallFlow.domain_id == domain_id))
    cf = r.scalar_one_or_none()
    if not cf:
        raise HTTPException(status_code=404, detail="Call flow not found")
    return _serialize(cf)


@router.patch("/{call_flow_uuid}", response_model=dict)
async def patch_cf(call_flow_uuid: UUID, payload: CallFlowUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_flow_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallFlow).where(CallFlow.call_flow_uuid == call_flow_uuid, CallFlow.domain_id == domain_id))
    cf = r.scalar_one_or_none()
    if not cf:
        raise HTTPException(status_code=404, detail="Call flow not found")
    update = payload.model_dump(exclude_unset=True)
    # map legacy keys
    if "call_flow_name" in update:
        cf.call_flow_name = update["call_flow_name"]
    if "call_flow_extension" in update:
        dup = await session.execute(select(CallFlow).where(CallFlow.domain_id == domain_id, CallFlow.call_flow_extension == update["call_flow_extension"], CallFlow.call_flow_uuid != call_flow_uuid))
        if dup.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Call flow extension already exists")
        cf.call_flow_extension = update["call_flow_extension"]
    for k in ["call_flow_feature_code", "call_flow_context", "call_flow_enabled", "call_flow_description"]:
        if k in update:
            setattr(cf, k, update[k])
    await session.commit()
    await session.refresh(cf)
    return _serialize(cf)


@router.delete("/{call_flow_uuid}", status_code=204)
async def delete_cf(call_flow_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_flow_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallFlow).where(CallFlow.call_flow_uuid == call_flow_uuid, CallFlow.domain_id == domain_id))
    cf = r.scalar_one_or_none()
    if not cf:
        raise HTTPException(status_code=404, detail="Call flow not found")
    await session.delete(cf)
    await session.commit()
    return Response(status_code=204)
