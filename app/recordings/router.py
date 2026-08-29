from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from fastapi.responses import RedirectResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.recordings.models import Recording
from app.recordings.schemas import RecordingCreate, RecordingRead, RecordingUpdate
from app.recordings.service import create_recording, list_recordings, presigned_for

router = APIRouter(prefix="/api/v1/recordings", tags=["recordings"])

def _links(obj: Recording):
    return {
        "self": {"href": f"/api/v1/recordings/{obj.recording_uuid}"},
        "collection": {"href": "/api/v1/recordings"},
        "playback": {"href": f"/api/v1/recordings/{obj.recording_uuid}/playback"},
    }

def _ser(obj: Recording, include_presigned: bool = False):
    d = RecordingRead.model_validate(obj, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(obj)
    if include_presigned:
        try:
            d["presigned_url"] = presigned_for(obj)
        except Exception:
            d["presigned_url"] = None
    return d

@router.get("", response_model=dict)
async def list_ep(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    q: Optional[str] = Query(None),
    sort: str = Query("created_at"),
    order: str = Query("desc"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("recording_view")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_recordings(session, domain_id, page, per_page, q, sort, order)
    response.headers["X-Total-Count"] = str(total)
    # RFC 5988 Link pagination
    base_url = str(request.url).split("?")[0]
    from urllib.parse import urlencode

    links = []
    if page * per_page < total:
        qs = dict(request.query_params)
        qs["page"] = str(page + 1)
        qs["per_page"] = str(per_page)
        links.append(f'<{base_url}?{urlencode(qs)}>; rel="next"')
    if page > 1:
        qs = dict(request.query_params)
        qs["page"] = str(page - 1)
        qs["per_page"] = str(per_page)
        links.append(f'<{base_url}?{urlencode(qs)}>; rel="prev"')
        qs_first = dict(request.query_params)
        qs_first["page"] = "1"
        qs_first["per_page"] = str(per_page)
        links.append(f'<{base_url}?{urlencode(qs_first)}>; rel="first"')
        qs_last_page = (total + per_page - 1) // per_page
        qs_last = dict(request.query_params)
        qs_last["page"] = str(qs_last_page)
        qs_last["per_page"] = str(per_page)
        links.append(f'<{base_url}?{urlencode(qs_last)}>; rel="last"')
    if links:
        response.headers["Link"] = ", ".join(links)
    response.headers["Access-Control-Expose-Headers"] = "X-Total-Count, Link"
    return {"data": [_ser(x) for x in items], "total": total, "page": page, "per_page": per_page, "_links": {"self": {"href": "/api/v1/recordings"}}}

@router.post("", response_model=dict, status_code=201)
async def create_ep(payload: RecordingCreate, response: Response, request: Request, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("recording_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    obj = await create_recording(session, payload, domain_id, user_id=getattr(user, "user_uuid", None))
    loc = f"/api/v1/recordings/{obj.recording_uuid}"
    response.headers["Location"] = loc
    return _ser(obj, include_presigned=True)

@router.get("/{uid}", response_model=dict)
async def get_ep(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("recording_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Recording).where(Recording.recording_uuid == uid, Recording.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Recording not found")
    return _ser(obj, include_presigned=True)

@router.get("/{uid}/playback")
async def playback_ep(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("recording_view")), expires: int = Query(3600, ge=60, le=86400)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Recording).where(Recording.recording_uuid == uid, Recording.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Recording not found")
    url = presigned_for(obj, expires=expires)
    # 302 redirect to S3 presigned URL (object store playback)
    return RedirectResponse(url=url, status_code=302)

@router.get("/{uid}/presign", response_model=dict)
async def presign_ep(uid: UUID, expires: int = Query(3600, ge=60, le=86400), session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("recording_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Recording).where(Recording.recording_uuid == uid, Recording.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Recording not found")
    url = presigned_for(obj, expires=expires)
    return {"recording_uuid": str(uid), "file_path": obj.file_path, "presigned_url": url, "expires_in": expires, "_links": _links(obj)}

@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid: UUID, payload: RecordingUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("recording_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Recording).where(Recording.recording_uuid == uid, Recording.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Recording not found")
    for k, v in payload.model_dump(exclude_unset=True).items():
        setattr(obj, k, v)
    await session.commit()
    await session.refresh(obj)
    return _ser(obj, include_presigned=True)

@router.delete("/{uid}", status_code=204)
async def delete_ep(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("recording_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Recording).where(Recording.recording_uuid == uid, Recording.domain_id == domain_id))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Recording not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)

@router.get("/health", include_in_schema=False)
async def health():
    return {"plugin": "recordings", "status": "ok", "backend": "s3"}
