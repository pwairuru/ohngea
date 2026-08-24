from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.voicemails.models import Voicemail, VoicemailDestination
from app.voicemails.schemas import VoicemailCreate, VoicemailRead, VoicemailUpdate, VoicemailDestinationRead
from app.voicemails.service import create_voicemail, list_voicemails, get_voicemail, get_voicemail_with_destinations

router = APIRouter(prefix="/api/v1/voicemails", tags=["voicemails"])


def _links(vm: Voicemail):
    base = f"/api/v1/voicemails/{vm.voicemail_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/voicemails"}}


def _serialize(vm: Voicemail, destinations: list[VoicemailDestination] | None = None):
    d = VoicemailRead.model_validate(vm, from_attributes=True).model_dump(mode="json")
    # embed destinations if provided
    if destinations is not None:
        d["forward_destinations"] = [VoicemailDestinationRead.model_validate(x).model_dump(mode="json") for x in destinations]
        d["destinations"] = d["forward_destinations"]
    else:
        d.setdefault("forward_destinations", [])
        d.setdefault("destinations", d["forward_destinations"])
    d["_links"] = _links(vm)
    # legacy aliases for Ohngea PBX compatibility
    d["voicemail_uuid"] = str(vm.voicemail_uuid)
    d["domain_uuid"] = str(vm.domain_id)
    d["domain_id"] = str(vm.domain_id)
    d["voicemail_id"] = vm.voicemail_id
    d["voicemail_mailbox"] = vm.voicemail_id
    d["mailbox"] = vm.voicemail_id
    d["extension"] = vm.voicemail_id
    d["voicemail_password"] = vm.voicemail_password
    d["password"] = vm.voicemail_password
    d["voicemail_enabled"] = vm.enabled
    d["enabled"] = vm.enabled
    d["voicemail_description"] = vm.description
    d["description"] = vm.description
    d["greeting_id"] = vm.greeting_id
    d["greeting"] = vm.greeting_id
    d["toll_allow"] = vm.toll_allow
    d["voicemail_mail_to"] = vm.voicemail_mail_to
    d["voicemail_file"] = vm.voicemail_file
    d["alternate_greet_id"] = vm.alternate_greet_id
    d["voicemail_alternate_greet_id"] = vm.alternate_greet_id
    # provide plural aliases for api consistency: keep voicemail_id etc.
    return d


@router.get("", response_model=dict, dependencies=[Depends(require_permission("voicemail_view"))])
async def list_vm(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    sort: str = Query("voicemail_id"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    q: Optional[str] = Query(None),
    filter_voicemail_id: Optional[str] = Query(None, alias="filter[voicemail_id]"),
    filter_mailbox: Optional[str] = Query(None, alias="filter[mailbox]"),
    filter_extension: Optional[str] = Query(None, alias="filter[extension]"),
    filter_enabled: Optional[bool] = Query(None, alias="filter[enabled]"),
    filter_description: Optional[str] = Query(None, alias="filter[description]"),
    filter_voicemail_enabled: Optional[bool] = Query(None, alias="filter[voicemail_enabled]"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)

    # merge legacy filters
    if filter_mailbox and not filter_voicemail_id:
        filter_voicemail_id = filter_mailbox
    if filter_extension and not filter_voicemail_id:
        filter_voicemail_id = filter_extension
    if filter_voicemail_enabled is not None and filter_enabled is None:
        filter_enabled = filter_voicemail_enabled

    items, total = await list_voicemails(
        session,
        domain_id,
        page,
        per_page,
        q,
        sort,
        order,
        filter_voicemail_id=filter_voicemail_id,
        filter_enabled=filter_enabled,
        filter_description=filter_description,
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

    data = []
    for vm in items:
        _, dests = await get_voicemail_with_destinations(session, vm.voicemail_uuid, domain_id)
        data.append(_serialize(vm, dests))
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/health")
async def health():
    return {"plugin": "voicemails", "status": "ok", "mode": "core"}


@router.post("", response_model=dict, status_code=201)
async def create_vm(
    payload: VoicemailCreate,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("voicemail_add")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id is not None and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    vm = await create_voicemail(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/voicemails/{vm.voicemail_uuid}"
    _, dests = await get_voicemail_with_destinations(session, vm.voicemail_uuid, domain_id)
    return _serialize(vm, dests)


@router.get("/{voicemail_uuid}", response_model=dict, dependencies=[Depends(require_permission("voicemail_view"))])
async def get_vm(voicemail_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    vm, dests = await get_voicemail_with_destinations(session, voicemail_uuid, domain_id)
    if not vm:
        raise HTTPException(status_code=404, detail="Voicemail not found")
    return _serialize(vm, dests)


@router.patch("/{voicemail_uuid}", response_model=dict)
async def patch_vm(
    voicemail_uuid: UUID,
    payload: VoicemailUpdate,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    perms = getattr(user, "_perms", None)
    if perms is not None:
        if not any(p in perms for p in ("voicemail_add", "voicemail_edit", "voicemail_delete", "*")):
            from app.auth.service import get_user_permissions

            db_perms = await get_user_permissions(session, user)
            if not any(p in db_perms for p in ("voicemail_add", "voicemail_edit", "*")):
                raise HTTPException(status_code=403, detail="Missing permission: voicemail_add or voicemail_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    vm = await get_voicemail(session, voicemail_uuid, domain_id)
    if not vm:
        raise HTTPException(status_code=404, detail="Voicemail not found")
    update = payload.model_dump(exclude_unset=True)
    # handle forward destinations separately
    forward_payload = update.pop("forward_destinations", None)
    if "destinations" in update:
        alt = update.pop("destinations", None)
        if alt is not None and forward_payload is None:
            forward_payload = alt
    # check duplicate voicemail_id if changed
    if "voicemail_id" in update and update["voicemail_id"] != vm.voicemail_id:
        existing = await session.execute(
            select(Voicemail).where(Voicemail.domain_id == domain_id, Voicemail.voicemail_id == str(update["voicemail_id"]), Voicemail.voicemail_uuid != voicemail_uuid)
        )
        if existing.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Voicemail mailbox already exists")
    for k, v in update.items():
        if k in ("domain_id",):
            continue
        setattr(vm, k, v)
    # handle forward destinations update if provided
    if forward_payload is not None:
        # delete existing
        existing = await session.execute(select(VoicemailDestination).where(VoicemailDestination.voicemail_uuid == voicemail_uuid))
        for m in existing.scalars().all():
            await session.delete(m)
        await session.flush()
        for item in forward_payload:
            copy_uuid = None
            if isinstance(item, str):
                s = item.strip()
                if not s:
                    continue
                try:
                    copy_uuid = UUID(s)
                except Exception:
                    r = await session.execute(select(Voicemail).where(Voicemail.domain_id == domain_id, Voicemail.voicemail_id == s))
                    found = r.scalar_one_or_none()
                    if found:
                        copy_uuid = found.voicemail_uuid
                    else:
                        continue
            elif isinstance(item, dict):
                val = item.get("voicemail_uuid_copy") or item.get("voicemail_copy_uuid") or item.get("copy_uuid") or item.get("uuid") or item.get("voicemail_id")
                if not val:
                    continue
                try:
                    copy_uuid = UUID(str(val))
                except Exception:
                    r = await session.execute(select(Voicemail).where(Voicemail.domain_id == domain_id, Voicemail.voicemail_id == str(val)))
                    found = r.scalar_one_or_none()
                    if found:
                        copy_uuid = found.voicemail_uuid
                    else:
                        continue
            else:
                try:
                    copy_uuid = UUID(str(item))
                except Exception:
                    try:
                        d = item.model_dump() if hasattr(item, "model_dump") else dict(item)
                        val = d.get("voicemail_uuid_copy")
                        if val:
                            copy_uuid = UUID(str(val))
                    except Exception:
                        continue
            if copy_uuid and copy_uuid != vm.voicemail_uuid:
                dest = VoicemailDestination(domain_id=domain_id, voicemail_uuid=vm.voicemail_uuid, voicemail_uuid_copy=copy_uuid)
                session.add(dest)
    from datetime import datetime

    vm.updated_at = datetime.utcnow()
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(vm)
    _, dests = await get_voicemail_with_destinations(session, vm.voicemail_uuid, domain_id)
    return _serialize(vm, dests)


@router.delete("/{voicemail_uuid}", status_code=204)
async def delete_vm(voicemail_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is not None:
        if not any(p in perms for p in ("voicemail_add", "voicemail_delete", "voicemail_edit", "*")):
            from app.auth.service import get_user_permissions

            db_perms = await get_user_permissions(session, user)
            if not any(p in db_perms for p in ("voicemail_add", "voicemail_delete", "*")):
                raise HTTPException(status_code=403, detail="Missing permission: voicemail_add or voicemail_delete")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    vm = await get_voicemail(session, voicemail_uuid, domain_id)
    if not vm:
        raise HTTPException(status_code=404, detail="Voicemail not found")
    await session.delete(vm)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)
