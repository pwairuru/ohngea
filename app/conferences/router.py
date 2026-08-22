from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.conferences.models import Conference, ConferenceUser
from app.conferences.schemas import ConferenceCreate, ConferenceRead, ConferenceUpdate
from app.conferences.service import create_conference, list_conferences, get_conference

router = APIRouter(prefix="/api/v1/conferences", tags=["conferences"])


def _links(conf: Conference):
    base = f"/api/v1/conferences/{conf.conference_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/conferences"}}


def _serialize(conf: Conference):
    d = ConferenceRead.model_validate(conf, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(conf)
    # legacy / drop-in aliases
    d["conference_uuid"] = str(conf.conference_uuid)
    d["domain_uuid"] = str(conf.domain_id)
    d["domain_id"] = str(conf.domain_id)
    d["conference_name"] = conf.conference_name
    d["conference_extension"] = conf.conference_extension
    d["conference_pin_number"] = conf.conference_pin_number
    d["conference_profile"] = conf.conference_profile
    d["conference_enabled"] = conf.conference_enabled
    d["conference_description"] = conf.conference_description
    d["conference_order"] = conf.conference_order
    d["conference_flags"] = conf.conference_flags
    d["conference_context"] = conf.conference_context
    d["conference_email_address"] = conf.conference_email_address
    d["conference_account_code"] = conf.conference_account_code
    d["dialplan_uuid"] = str(conf.dialplan_uuid) if conf.dialplan_uuid else None
    # modern simple aliases
    d["name"] = conf.conference_name
    d["extension"] = conf.conference_extension
    d["pin_number"] = conf.conference_pin_number
    d["pin"] = conf.conference_pin_number
    d["profile"] = conf.conference_profile
    d["enabled"] = conf.conference_enabled
    d["description"] = conf.conference_description
    d["order"] = conf.conference_order
    d["flags"] = conf.conference_flags
    d["context"] = conf.conference_context
    d["email_address"] = conf.conference_email_address
    d["account_code"] = conf.conference_account_code
    # compatibility also provide conference_ wideband style?
    return d


@router.get("", response_model=dict)
async def list_conf(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("conference_name"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    filter_name: Optional[str] = Query(None, alias="filter[name]"),
    filter_conference_name: Optional[str] = Query(None, alias="filter[conference_name]"),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_conference_extension: Optional[str] = Query(None, alias="filter[conference_extension]"),
    filter_enabled: Optional[bool] = Query(None, alias="filter[enabled]"),
    filter_conference_enabled: Optional[bool] = Query(None, alias="filter[conference_enabled]"),
    filter_profile: Optional[str] = Query(None, alias="filter[profile]"),
    filter_conference_profile: Optional[str] = Query(None, alias="filter[conference_profile]"),
    filter_description: Optional[str] = Query(None, alias="filter[description]"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)

    # merge legacy filter aliases
    if filter_conference_name and not filter_name:
        filter_name = filter_conference_name
    if filter_conference_extension and not filter_extension:
        filter_extension = filter_conference_extension
    if filter_conference_enabled is not None and filter_enabled is None:
        filter_enabled = filter_conference_enabled
    if filter_conference_profile and not filter_profile:
        filter_profile = filter_conference_profile
    # description filter maps to name? use q instead; but handle
    if filter_description and not filter_name:
        # treat as name filter for simplicity; actually map to description via q inside service? For now add to filter_name to capture
        # We'll pass filter_name as combined? Just log but still handle via list param not exists; so use q for description?
        # We'll create additional filter logic here by not passing but handling manually after? Simpler: if filter_description set, use it as filter_name and search via q?
        # We'll pass filter_name as description search if filter_description present
        filter_name = filter_description

    items, total = await list_conferences(
        session, domain_id, page, per_page, q, sort, order,
        filter_name=filter_name,
        filter_extension=filter_extension,
        filter_enabled=filter_enabled,
        filter_profile=filter_profile,
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

    data = [_serialize(c) for c in items]
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/health")
async def health():
    return {"plugin": "conferences", "status": "ok", "mode": "core"}


@router.post("", response_model=dict, status_code=201)
async def create_conf(
    payload: ConferenceCreate,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("conference_add")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id is not None and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    conf = await create_conference(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/conferences/{conf.conference_uuid}"
    return _serialize(conf)


@router.get("/{conference_uuid}", response_model=dict)
async def get_conf(conference_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    conf = await get_conference(session, conference_uuid, domain_id)
    if not conf:
        raise HTTPException(status_code=404, detail="Conference not found")
    return _serialize(conf)


@router.patch("/{conference_uuid}", response_model=dict)
async def patch_conf(
    conference_uuid: UUID,
    payload: ConferenceUpdate,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    perms = getattr(user, "_perms", None)
    if perms is not None:
        if not any(p in perms for p in ("conference_add", "conference_edit", "conference_delete", "*")):
            from app.auth.service import get_user_permissions

            db_perms = await get_user_permissions(session, user)
            if not any(p in db_perms for p in ("conference_add", "conference_edit", "*")):
                raise HTTPException(status_code=403, detail="Missing permission: conference_add or conference_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    conf = await get_conference(session, conference_uuid, domain_id)
    if not conf:
        raise HTTPException(status_code=404, detail="Conference not found")
    update = payload.model_dump(exclude_unset=True)
    # duplicate extension check
    if "conference_extension" in update and update["conference_extension"] != conf.conference_extension:
        existing = await session.execute(select(Conference).where(Conference.domain_id == domain_id, Conference.conference_extension == str(update["conference_extension"]), Conference.conference_uuid != conference_uuid))
        if existing.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Conference extension already exists")
    for k, v in update.items():
        if k in ("domain_id",):
            continue
        setattr(conf, k, v)
    from datetime import datetime

    conf.updated_at = datetime.utcnow()
    await session.commit()
    await session.refresh(conf)
    return _serialize(conf)


@router.delete("/{conference_uuid}", status_code=204)
async def delete_conf(conference_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is not None:
        if not any(p in perms for p in ("conference_add", "conference_delete", "conference_edit", "*")):
            from app.auth.service import get_user_permissions

            db_perms = await get_user_permissions(session, user)
            if not any(p in db_perms for p in ("conference_add", "conference_delete", "*")):
                raise HTTPException(status_code=403, detail="Missing permission: conference_add or conference_delete")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    conf = await get_conference(session, conference_uuid, domain_id)
    if not conf:
        raise HTTPException(status_code=404, detail="Conference not found")
    await session.delete(conf)
    await session.commit()
    return Response(status_code=204)


# Conference users sub-resource (optional)
@router.get("/{conference_uuid}/users", response_model=dict)
async def list_conf_users(conference_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    conf = await get_conference(session, conference_uuid, domain_id)
    if not conf:
        raise HTTPException(status_code=404, detail="Conference not found")
    r = await session.execute(select(ConferenceUser).where(ConferenceUser.conference_uuid == conference_uuid, ConferenceUser.domain_id == domain_id))
    users = r.scalars().all()
    data = [{"conference_user_uuid": str(u.conference_user_uuid), "conference_uuid": str(u.conference_uuid), "domain_id": str(u.domain_id), "user_uuid": str(u.user_uuid)} for u in users]
    return {"data": data, "total": len(data)}


@router.post("/{conference_uuid}/users", response_model=dict, status_code=201)
async def add_conf_user(conference_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("conference_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    conf = await get_conference(session, conference_uuid, domain_id)
    if not conf:
        raise HTTPException(status_code=404, detail="Conference not found")
    user_uuid = payload.get("user_uuid") or payload.get("userUuid") or payload.get("uuid")
    if not user_uuid:
        raise HTTPException(status_code=422, detail="user_uuid required")
    try:
        user_uuid = UUID(str(user_uuid))
    except Exception:
        raise HTTPException(status_code=422, detail="Invalid user_uuid")
    # check duplicate
    existing = await session.execute(select(ConferenceUser).where(ConferenceUser.conference_uuid == conference_uuid, ConferenceUser.user_uuid == user_uuid))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="User already in conference")
    cu = ConferenceUser(domain_id=domain_id, conference_uuid=conference_uuid, user_uuid=user_uuid)
    session.add(cu)
    await session.commit()
    await session.refresh(cu)
    return {"conference_user_uuid": str(cu.conference_user_uuid), "conference_uuid": str(cu.conference_uuid), "domain_id": str(cu.domain_id), "user_uuid": str(cu.user_uuid)}
