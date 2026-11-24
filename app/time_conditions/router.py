from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.time_conditions.models import TimeCondition
from app.time_conditions.schemas import TimeConditionCreate, TimeConditionRead, TimeConditionUpdate
from app.time_conditions.service import create_time_condition, list_time_conditions, get_time_condition

router = APIRouter(prefix="/api/v1/time_conditions", tags=["time_conditions"])


def _links(tc: TimeCondition):
    base = f"/api/v1/time_conditions/{tc.time_condition_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/time_conditions"}}


def _serialize(tc: TimeCondition):
    d = TimeConditionRead.model_validate(tc, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(tc)
    # legacy compatibility keys
    d["time_condition_uuid"] = str(tc.time_condition_uuid)
    d["dialplan_uuid"] = str(tc.dialplan_uuid) if tc.dialplan_uuid else None
    d["time_condition_name"] = tc.name
    d["dialplan_name"] = tc.name
    d["time_condition_extension"] = tc.extension
    d["dialplan_number"] = tc.extension
    d["time_condition_description"] = tc.description
    d["dialplan_description"] = tc.description
    d["time_condition_enabled"] = tc.enabled
    d["dialplan_enabled"] = tc.enabled
    d["dialplan_context"] = tc.context
    # also map true/false dest aliases
    d["true_dest"] = tc.true_destination
    d["false_dest"] = tc.false_destination
    # provide extension alias
    d["extension"] = tc.extension
    d["name"] = tc.name
    return d


@router.get("", response_model=dict)
async def list_tc(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("name"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    filter_name: Optional[str] = Query(None, alias="filter[name]"),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_enabled: Optional[bool] = Query(None, alias="filter[enabled]"),
    filter_context: Optional[str] = Query(None, alias="filter[context]"),
    # legacy aliases
    filter_time_condition_name: Optional[str] = Query(None, alias="filter[time_condition_name]"),
    filter_dialplan_number: Optional[str] = Query(None, alias="filter[dialplan_number]"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)

    # merge legacy filter aliases
    if filter_time_condition_name and not filter_name:
        filter_name = filter_time_condition_name
    if filter_dialplan_number and not filter_extension:
        filter_extension = filter_dialplan_number

    items, total = await list_time_conditions(
        session, domain_id, page, per_page, q, sort, order,
        filter_name=filter_name,
        filter_extension=filter_extension,
        filter_enabled=filter_enabled,
        filter_context=filter_context,
    )

    response.headers["X-Total-Count"] = str(total)
    base_url = str(request.url).split("?")[0]
    if page * per_page < total:
        next_page = page + 1
        from urllib.parse import urlencode

        qs = dict(request.query_params)
        qs["page"] = str(next_page)
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

    data = [_serialize(tc) for tc in items]
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/health")
async def health():
    return {"plugin": "time_conditions", "status": "ok", "mode": "core"}


@router.post("", response_model=dict, status_code=201)
async def create_tc(
    payload: TimeConditionCreate,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("time_condition_add")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    # allow payload.domain_id to be missing — will override; if present must match
    if payload.domain_id is not None and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    tc = await create_time_condition(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/time_conditions/{tc.time_condition_uuid}"
    return _serialize(tc)


@router.get("/{time_condition_uuid}", response_model=dict)
async def get_tc(time_condition_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    tc = await get_time_condition(session, time_condition_uuid, domain_id)
    if not tc:
        raise HTTPException(status_code=404, detail="Time condition not found")
    return _serialize(tc)


@router.patch("/{time_condition_uuid}", response_model=dict)
async def patch_tc(
    time_condition_uuid: UUID,
    payload: TimeConditionUpdate,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    perms = getattr(user, "_perms", None)
    # allow add, edit, or wildcard
    if perms is not None:
        if not any(p in perms for p in ("time_condition_add", "time_condition_edit", "time_condition_delete", "*")):
            # fallback to DB perms check
            from app.auth.service import get_user_permissions
            db_perms = await get_user_permissions(session, user)
            if not any(p in db_perms for p in ("time_condition_add", "time_condition_edit", "*")):
                raise HTTPException(status_code=403, detail="Missing permission: time_condition_add or time_condition_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    tc = await get_time_condition(session, time_condition_uuid, domain_id)
    if not tc:
        raise HTTPException(status_code=404, detail="Time condition not found")
    update = payload.model_dump(exclude_unset=True)
    # handle extension duplicate check
    if "extension" in update and update["extension"] != tc.extension:
        existing = await session.execute(select(TimeCondition).where(TimeCondition.domain_id == domain_id, TimeCondition.extension == str(update["extension"]), TimeCondition.time_condition_uuid != time_condition_uuid))
        if existing.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Time condition extension already exists")
    for k, v in update.items():
        if k in ("domain_id",):
            continue
        setattr(tc, k, v)
    # update updated_at
    from datetime import datetime, timezone
    tc.updated_at = datetime.now(timezone.utc)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(tc)
    return _serialize(tc)


@router.delete("/{time_condition_uuid}", status_code=204)
async def delete_tc(time_condition_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is not None:
        if not any(p in perms for p in ("time_condition_add", "time_condition_delete", "time_condition_edit", "*")):
            from app.auth.service import get_user_permissions
            db_perms = await get_user_permissions(session, user)
            if not any(p in db_perms for p in ("time_condition_add", "time_condition_delete", "*")):
                raise HTTPException(status_code=403, detail="Missing permission: time_condition_add or time_condition_delete")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    tc = await get_time_condition(session, time_condition_uuid, domain_id)
    if not tc:
        raise HTTPException(status_code=404, detail="Time condition not found")
    await session.delete(tc)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
