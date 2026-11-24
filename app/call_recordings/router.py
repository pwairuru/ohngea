from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from pydantic import BaseModel, ConfigDict

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant, uuid7
from app.call_recordings.models import CallRecording

router = APIRouter(prefix="/api/v1/call_recordings", tags=["call_recordings"])


class CallRecordingCreate(BaseModel):
    model_config = ConfigDict(extra="forbid")
    domain_id: Optional[UUID] = None
    cdr_uuid: Optional[UUID] = None
    recording_path: Optional[str] = None
    recording_name: Optional[str] = None
    transcription: Optional[str] = None
    summary: Optional[str] = None


class CallRecordingUpdate(BaseModel):
    model_config = ConfigDict(extra="forbid")
    cdr_uuid: Optional[UUID] = None
    recording_path: Optional[str] = None
    recording_name: Optional[str] = None
    transcription: Optional[str] = None
    summary: Optional[str] = None


def _ser(o: CallRecording):
    return {
        "call_recording_uuid": str(o.call_recording_uuid),
        "domain_id": str(o.domain_id),
        "cdr_uuid": str(o.cdr_uuid) if o.cdr_uuid else None,
        "recording_path": o.recording_path,
        "recording_name": o.recording_name,
        "transcription": o.transcription,
        "summary": o.summary,
        "created_at": o.created_at.isoformat() if o.created_at else None,
        "updated_at": o.updated_at.isoformat() if getattr(o, "updated_at", None) else None,
        "_links": {
            "self": {"href": f"/api/v1/call_recordings/{o.call_recording_uuid}"},
            "collection": {"href": "/api/v1/call_recordings"},
        },
    }


@router.get("", response_model=dict)
async def list_call_recordings(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    q: Optional[str] = Query(None),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("call_recording_view")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    query = select(CallRecording).where(CallRecording.domain_id == domain_id)
    count_q = select(func.count()).select_from(CallRecording).where(CallRecording.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(CallRecording.recording_name.ilike(like), CallRecording.recording_path.ilike(like), CallRecording.transcription.ilike(like)))
        count_q = count_q.where(or_(CallRecording.recording_name.ilike(like), CallRecording.recording_path.ilike(like), CallRecording.transcription.ilike(like)))
    query = query.order_by(CallRecording.created_at.desc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    response.headers["X-Total-Count"] = str(total)
    return {"data": [_ser(x) for x in items], "total": total, "page": page, "per_page": per_page}


@router.post("", response_model=dict, status_code=201)
async def create_call_recording(
    payload: CallRecordingCreate,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("call_recording_add")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    obj = CallRecording(
        call_recording_uuid=uuid7(),
        domain_id=domain_id,
        cdr_uuid=payload.cdr_uuid,
        recording_path=payload.recording_path,
        recording_name=payload.recording_name,
        transcription=payload.transcription,
        summary=payload.summary,
    )
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/call_recordings/{obj.call_recording_uuid}"
    return _ser(obj)


@router.get("/{uid}", response_model=dict)
async def get_call_recording(
    uid: UUID,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("call_recording_view")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallRecording).where(CallRecording.call_recording_uuid == uid, CallRecording.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Call recording not found")
    return _ser(o)


@router.patch("/{uid}", response_model=dict)
async def patch_call_recording(
    uid: UUID,
    payload: CallRecordingUpdate,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("call_recording_edit")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallRecording).where(CallRecording.call_recording_uuid == uid, CallRecording.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Call recording not found")
    for k, v in payload.model_dump(exclude_unset=True).items():
        setattr(o, k, v)
    await session.commit()
    await session.refresh(o)
    return _ser(o)


@router.delete("/{uid}", status_code=204)
async def delete_call_recording(
    uid: UUID,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("call_recording_delete")),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallRecording).where(CallRecording.call_recording_uuid == uid, CallRecording.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Call recording not found")
    await session.delete(o)
    await session.commit()
    return Response(status_code=204)
