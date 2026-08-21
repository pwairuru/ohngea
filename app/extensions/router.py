from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, get_tenant_id, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.extensions.models import Extension
from app.extensions.schemas import ExtensionCreate, ExtensionRead, ExtensionUpdate
from app.extensions.service import create_extension, list_extensions

router = APIRouter(prefix="/api/v1/extensions", tags=["extensions"])


def _links(ext: Extension):
    base = f"/api/v1/extensions/{ext.extension_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/extensions"}}


@router.get("", response_model=dict)
async def list_ext(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("extension"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_enabled: Optional[bool] = Query(None, alias="filter[enabled]"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)

    # support filter[extension]
    if filter_extension:
        q = filter_extension

    items, total = await list_extensions(session, domain_id, page, per_page, q, sort, order)
    # pagination headers per Global Constraints
    response.headers["X-Total-Count"] = str(total)
    # Link header
    base_url = str(request.url).split("?")[0]
    # simple next link if more pages
    if page * per_page < total:
        next_page = page + 1
        # preserve query params but update page
        from urllib.parse import urlencode

        qs = dict(request.query_params)
        qs["page"] = str(next_page)
        qs["per_page"] = str(per_page)
        response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="next"'
        # also prev
        if page > 1:
            qs["page"] = str(page - 1)
            response.headers["Link"] += f', <{base_url}?{urlencode(qs)}>; rel="prev"'
    else:
        if page > 1:
            from urllib.parse import urlencode

            qs = dict(request.query_params)
            qs["page"] = str(page - 1)
            response.headers["Link"] = f'<{base_url}?{urlencode(qs)}>; rel="prev"'

    # lightweight HATEOAS
    data = []
    for e in items:
        d = ExtensionRead.model_validate(e).model_dump()
        d["_links"] = _links(e)
        # filter enabled if requested
        if filter_enabled is not None and d["enabled"] != filter_enabled:
            continue
        data.append(d)
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", response_model=ExtensionRead, status_code=201)
async def create_ext(
    payload: ExtensionCreate,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("extension_add")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    # enforce tenant isolation: payload.domain_id must match JWT domain_id unless superadmin with X header
    if str(payload.domain_id) != str(domain_id):
        # allow if user has domain_add and payload domain exists (superadmin case) — for now allow if matched or superadmin
        # strict: must match
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    ext = await create_extension(session, payload)
    response.headers["Location"] = f"/api/v1/extensions/{ext.extension_uuid}"
    # add links
    result = ExtensionRead.model_validate(ext).model_dump()
    result["_links"] = _links(ext)
    # trick: return dict with links via response_model? FastAPI will strip _links if not in schema,
    # so we return Response directly
    # Instead we return extension read and rely on middleware? For now return ext
    return ext


@router.get("/{extension_uuid}", response_model=dict)
async def get_ext(extension_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Extension).where(Extension.extension_uuid == extension_uuid, Extension.domain_id == domain_id))
    e = r.scalar_one_or_none()
    if not e:
        raise HTTPException(status_code=404, detail="Extension not found")
    d = ExtensionRead.model_validate(e).model_dump(mode="json")
    d["_links"] = _links(e)
    return d


@router.patch("/{extension_uuid}", response_model=ExtensionRead)
async def patch_ext(extension_uuid: UUID, payload: ExtensionUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("extension_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Extension).where(Extension.extension_uuid == extension_uuid, Extension.domain_id == domain_id))
    e = r.scalar_one_or_none()
    if not e:
        raise HTTPException(status_code=404, detail="Extension not found")
    update = payload.model_dump(exclude_unset=True)
    for k, v in update.items():
        setattr(e, k, v)
    await session.commit()
    await session.refresh(e)
    try:
        import redis.asyncio as redis
        from app.config import settings
        rds = redis.from_url(settings.redis_url)
        await rds.delete(f"xml:{domain_id}:directory")
        await rds.aclose()
    except Exception:
        pass
    return e


@router.delete("/{extension_uuid}", status_code=204)
async def delete_ext(extension_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("extension_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Extension).where(Extension.extension_uuid == extension_uuid, Extension.domain_id == domain_id))
    e = r.scalar_one_or_none()
    if not e:
        raise HTTPException(status_code=404, detail="Extension not found")
    await session.delete(e)
    await session.commit()
    try:
        import redis.asyncio as redis
        from app.config import settings
        rds = redis.from_url(settings.redis_url)
        await rds.delete(f"xml:{domain_id}:directory")
        await rds.aclose()
    except Exception:
        pass
    return Response(status_code=204)


@router.post(":batch", status_code=201)
async def batch_create(payload: list[ExtensionCreate], session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("extension_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    created = []
    for data in payload:
        if str(data.domain_id) != str(domain_id):
            raise HTTPException(status_code=403, detail="Domain mismatch in batch")
        ext = Extension(**data.model_dump())
        session.add(ext)
        created.append(ext)
    await session.commit()
    for c in created:
        await session.refresh(c)
    return {"created": len(created)}
