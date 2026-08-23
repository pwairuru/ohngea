from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.fax.models import Fax
from app.fax.schemas import FaxCreate, FaxRead, FaxUpdate
from app.fax.service import create_fax, list_faxes

router = APIRouter(prefix="/api/v1/fax", tags=["fax"])


def _links(fax: Fax):
    base = f"/api/v1/fax/{fax.fax_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/fax"}}


def _serialize(fax: Fax):
    d = FaxRead.model_validate(fax, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(fax)
    d["fax_uuid"] = str(fax.fax_uuid)
    d["fax_extension"] = fax.fax_extension
    d["fax_name"] = fax.fax_name
    d["fax_email"] = fax.fax_email
    return d


@router.get("", response_model=dict)
async def list_fax(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("fax_extension"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_fax_extension: Optional[str] = Query(None, alias="filter[fax_extension]"),
    filter_enabled: Optional[bool] = Query(None, alias="filter[enabled]"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    ext = filter_fax_extension or filter_extension
    items, total = await list_faxes(session, domain_id, page, per_page, q, sort, order, ext, filter_enabled)
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
    data = [_serialize(f) for f in items]
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/health", response_model=dict)
async def health():
    return {"status": "ok", "mode": "core", "fax": "enabled"}


@router.post("", response_model=dict, status_code=201)
async def create_fax_endpoint(payload: FaxCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fax_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    # ensure domain_id set
    if not payload.domain_id:
        payload.domain_id = domain_id
    await set_tenant(session, domain_id)
    fax = await create_fax(session, payload)
    response.headers["Location"] = f"/api/v1/fax/{fax.fax_uuid}"
    return _serialize(fax)


@router.get("/{fax_uuid}", response_model=dict)
async def get_fax(fax_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Fax).where(Fax.fax_uuid == fax_uuid, Fax.domain_id == domain_id))
    fax = r.scalar_one_or_none()
    if not fax:
        raise HTTPException(status_code=404, detail="Fax not found")
    return _serialize(fax)


@router.patch("/{fax_uuid}", response_model=dict)
async def patch_fax(fax_uuid: UUID, payload: FaxUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fax_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Fax).where(Fax.fax_uuid == fax_uuid, Fax.domain_id == domain_id))
    fax = r.scalar_one_or_none()
    if not fax:
        raise HTTPException(status_code=404, detail="Fax not found")
    update = payload.model_dump(exclude_unset=True, exclude={"domain_id"})
    for k, v in update.items():
        if hasattr(fax, k):
            setattr(fax, k, v)
    # handle extension alias
    if payload.fax_extension and payload.fax_extension != fax.fax_extension:
        # check duplicate
        dup = await session.execute(select(Fax).where(Fax.domain_id == domain_id, Fax.fax_extension == payload.fax_extension, Fax.fax_uuid != fax_uuid))
        if dup.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Fax extension already exists")
        fax.fax_extension = payload.fax_extension
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(fax)
    return _serialize(fax)


@router.delete("/{fax_uuid}", status_code=204)
async def delete_fax(fax_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fax_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Fax).where(Fax.fax_uuid == fax_uuid, Fax.domain_id == domain_id))
    fax = r.scalar_one_or_none()
    if not fax:
        raise HTTPException(status_code=404, detail="Fax not found")
    await session.delete(fax)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)
