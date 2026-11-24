from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.fax.models import Fax, FaxFile, FaxLog, FaxUser
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
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
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
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)


def _file_to_dict(f: FaxFile) -> dict:
    return {
        "fax_file_uuid": str(f.fax_file_uuid),
        "fax_uuid": str(f.fax_uuid),
        "domain_id": str(f.domain_id),
        "file_name": f.file_name,
        "file_path": f.file_path,
        "file_size": f.file_size,
        "fax_file_path": f.fax_file_path,
        "fax_file_type": f.fax_file_type,
        "created_at": f.created_at.isoformat() if f.created_at else None,
    }


def _log_to_dict(l: FaxLog) -> dict:
    return {
        "fax_log_uuid": str(l.fax_log_uuid),
        "fax_uuid": str(l.fax_uuid),
        "domain_id": str(l.domain_id),
        "fax_success": l.fax_success,
        "result_code": l.result_code,
        "fax_result_text": l.fax_result_text,
        "created_at": l.created_at.isoformat() if l.created_at else None,
    }


async def _get_fax_or_404(session, fax_uuid: UUID, domain_id):
    r = await session.execute(select(Fax).where(Fax.fax_uuid == fax_uuid, Fax.domain_id == domain_id))
    fax = r.scalar_one_or_none()
    if not fax:
        raise HTTPException(status_code=404, detail="Fax not found")
    return fax


@router.get("/{fax_uuid}/files", response_model=dict)
async def list_fax_files(fax_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    await _get_fax_or_404(session, fax_uuid, domain_id)
    r = await session.execute(select(FaxFile).where(FaxFile.fax_uuid == fax_uuid, FaxFile.domain_id == domain_id).order_by(FaxFile.created_at.desc()))
    items = r.scalars().all()
    return {"data": [_file_to_dict(f) for f in items], "total": len(items)}


@router.get("/{fax_uuid}/logs", response_model=dict)
async def list_fax_logs(fax_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    await _get_fax_or_404(session, fax_uuid, domain_id)
    r = await session.execute(select(FaxLog).where(FaxLog.fax_uuid == fax_uuid, FaxLog.domain_id == domain_id).order_by(FaxLog.created_at.desc()))
    items = r.scalars().all()
    return {"data": [_log_to_dict(x) for x in items], "total": len(items)}


@router.get("/{fax_uuid}/users", response_model=dict)
async def list_fax_users(fax_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    await _get_fax_or_404(session, fax_uuid, domain_id)
    r = await session.execute(select(FaxUser).where(FaxUser.fax_uuid == fax_uuid, FaxUser.domain_id == domain_id))
    items = r.scalars().all()
    return {"data": [{"fax_user_uuid": str(x.fax_user_uuid), "fax_uuid": str(x.fax_uuid), "user_uuid": str(x.user_uuid)} for x in items], "total": len(items)}


@router.post("/{fax_uuid}/files", response_model=dict, status_code=201)
async def upload_fax_file(fax_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fax_add"))):
    """Upload a fax document: base64 content goes to S3, metadata in fax_files."""
    import base64 as _b64

    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    await _get_fax_or_404(session, fax_uuid, domain_id)
    name = payload.get("file_name") or "document.tiff"
    file_path = payload.get("file_path")
    size = payload.get("file_size")
    raw_b64 = payload.get("file_base64")
    if raw_b64:
        try:
            data = _b64.b64decode(raw_b64)
        except Exception:
            raise HTTPException(status_code=422, detail="invalid file_base64")
        from app.storage.s3 import s3_key_for_fax, s3_upload_bytes

        file_path = s3_key_for_fax(domain_id, fax_uuid, name)
        s3_upload_bytes(file_path, data, content_type=payload.get("content_type") or "image/tiff")
        size = size or len(data)
    if not file_path:
        raise HTTPException(status_code=422, detail="file_path or file_base64 required")
    obj = FaxFile(domain_id=domain_id, fax_uuid=fax_uuid, file_name=name, file_path=file_path, file_size=size, fax_file_path=file_path, fax_file_type=payload.get("fax_file_type"))
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return _file_to_dict(obj)


@router.post("/{fax_uuid}/send", response_model=dict)
async def send_fax(fax_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fax_add"))):
    """Queue an outbound fax: creates a fax_queue entry and attempts ESL originate (best effort)."""
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    fax = await _get_fax_or_404(session, fax_uuid, domain_id)
    number = payload.get("fax_number") or payload.get("number")
    if not number:
        raise HTTPException(status_code=422, detail="fax_number required")
    from app.fax_queue.models import FaxQueue
    from app.freeswitch.sanitize import sanitize_dial

    try:
        number = sanitize_dial(str(number))
    except ValueError:
        raise HTTPException(status_code=422, detail="invalid fax_number")

    q = FaxQueue(
        fax_uuid=fax_uuid,
        domain_id=domain_id,
        fax_number=str(number),
        fax_recipient=payload.get("fax_recipient"),
        fax_caller_id_name=payload.get("fax_caller_id_name") or fax.fax_caller_id_name,
        fax_caller_id_number=payload.get("fax_caller_id_number") or fax.fax_caller_id_number,
        fax_prefix=payload.get("fax_prefix") or fax.fax_prefix,
        fax_status="queued",
    )
    session.add(q)
    await session.commit()
    result = "queued"
    try:
        from app.freeswitch.esl_client import esl_command

        ext = fax.fax_extension or ""
        try:
            ext = sanitize_dial(str(ext))
        except ValueError:
            ext = ""
        caller = q.fax_caller_id_number or ""
        try:
            caller = sanitize_dial(str(caller)) if caller else ""
        except ValueError:
            caller = ""
        raw = await esl_command(f"originate {{origination_caller_id_number={caller}}}user/{ext} &txfax({number})")
        result = str(raw)[:500] if raw else "queued"
    except Exception as e:
        result = "queued (ESL offline)"
    return {"fax_queue_uuid": str(q.fax_queue_uuid), "fax_uuid": str(fax_uuid), "fax_number": str(number), "status": "queued", "result": result}
