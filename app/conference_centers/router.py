from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.conference_centers.models import ConferenceCenter, ConferenceRoom, ConferenceRoomUser, ConferenceSession, ConferenceSessionDetail
from app.conference_centers.schemas import ConferenceCenterCreate, ConferenceCenterRead, ConferenceCenterUpdate
from app.conference_centers.service import create_center, list_centers
router=APIRouter(prefix="/api/v1/conference_centers", tags=["conference_centers"])
def _links(o): return {"self":{"href":f"/api/v1/conference_centers/{o.conference_center_uuid}"}, "collection":{"href":"/api/v1/conference_centers"}}
def _ser(o):
    d=ConferenceCenterRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["conference_center_uuid"]=str(o.conference_center_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_centers(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:ConferenceCenterCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_center(session,payload,domain_id)
    response.headers["Location"]=f"/api/v1/conference_centers/{obj.conference_center_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(ConferenceCenter).where(ConferenceCenter.conference_center_uuid==uid, ConferenceCenter.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Conference center not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid:UUID, payload:ConferenceCenterUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(ConferenceCenter).where(ConferenceCenter.conference_center_uuid==uid, ConferenceCenter.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Conference center not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(o,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(o)
    return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(ConferenceCenter).where(ConferenceCenter.conference_center_uuid==uid, ConferenceCenter.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Conference center not found")
    await session.delete(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)

# --- conference rooms child resources ---
def _room_ser(o: ConferenceRoom):
    return {
        "conference_room_uuid": str(o.conference_room_uuid),
        "domain_id": str(o.domain_id),
        "conference_center_uuid": str(o.conference_center_uuid),
        "conference_room_name": o.conference_room_name,
        "room_name": o.room_name or o.conference_room_name,
        "room_pin": o.room_pin or o.participant_pin,
        "participant_pin": o.participant_pin,
        "moderator_pin": o.moderator_pin,
        "max_members": o.max_members,
        "record": o.record,
        "profile": o.profile,
        "enabled": o.enabled,
        "description": o.description,
        "created_at": o.created_at.isoformat() if o.created_at else None,
        "updated_at": o.updated_at.isoformat() if getattr(o, "updated_at", None) else None,
        "_links": {
            "self": {"href": f"/api/v1/conference_centers/{o.conference_center_uuid}/rooms/{o.conference_room_uuid}"},
            "collection": {"href": f"/api/v1/conference_centers/{o.conference_center_uuid}/rooms"},
        },
    }

@router.get("/{center_uuid}/rooms", response_model=dict)
async def list_rooms(center_uuid: UUID, request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), q: Optional[str] = Query(None), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # verify center exists
    r = await session.execute(select(ConferenceCenter).where(ConferenceCenter.conference_center_uuid == center_uuid, ConferenceCenter.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Conference center not found")
    query = select(ConferenceRoom).where(ConferenceRoom.domain_id == domain_id, ConferenceRoom.conference_center_uuid == center_uuid)
    if q:
        like = f"%{q}%"
        from sqlalchemy import or_
        query = query.where(or_(ConferenceRoom.conference_room_name.ilike(like), ConferenceRoom.room_name.ilike(like)))
    from sqlalchemy import func
    count_q = select(func.count()).select_from(query.subquery())
    total = (await session.execute(count_q)).scalar() or 0
    query = query.order_by(ConferenceRoom.conference_room_name.asc()).offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    response.headers["X-Total-Count"] = str(total)
    return {"data": [_room_ser(x) for x in items], "total": total, "page": page, "per_page": per_page}

@router.post("/{center_uuid}/rooms", response_model=dict, status_code=201)
async def create_room(center_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("conference_room_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(ConferenceCenter).where(ConferenceCenter.conference_center_uuid == center_uuid, ConferenceCenter.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Conference center not found")
    name = payload.get("conference_room_name") or payload.get("room_name") or payload.get("name")
    if not name:
        raise HTTPException(status_code=422, detail="conference_room_name required")
    obj = ConferenceRoom(
        domain_id=domain_id,
        conference_center_uuid=center_uuid,
        conference_room_name=name,
        room_name=payload.get("room_name") or name,
        room_pin=payload.get("room_pin") or payload.get("participant_pin"),
        participant_pin=payload.get("participant_pin") or payload.get("room_pin"),
        moderator_pin=payload.get("moderator_pin"),
        max_members=payload.get("max_members"),
        record=payload.get("record") if payload.get("record") is not None else False,
        profile=payload.get("profile"),
        enabled=payload.get("enabled") if payload.get("enabled") is not None else True,
        description=payload.get("description"),
        email_address=payload.get("email_address"),
        account_code=payload.get("account_code"),
    )
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    response.headers["Location"] = f"/api/v1/conference_centers/{center_uuid}/rooms/{obj.conference_room_uuid}"
    return _room_ser(obj)

@router.get("/{center_uuid}/rooms/{room_uuid}", response_model=dict)
async def get_room(center_uuid: UUID, room_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(ConferenceRoom).where(ConferenceRoom.conference_room_uuid == room_uuid, ConferenceRoom.conference_center_uuid == center_uuid, ConferenceRoom.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Conference room not found")
    return _room_ser(o)

@router.patch("/{center_uuid}/rooms/{room_uuid}", response_model=dict)
async def patch_room(center_uuid: UUID, room_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("conference_room_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(ConferenceRoom).where(ConferenceRoom.conference_room_uuid == room_uuid, ConferenceRoom.conference_center_uuid == center_uuid, ConferenceRoom.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Conference room not found")
    for k, v in payload.items():
        if hasattr(o, k):
            setattr(o, k, v)
    # alias handling
    if "room_name" in payload and payload["room_name"]:
        o.conference_room_name = payload["room_name"]
    if "room_pin" in payload and payload["room_pin"]:
        o.participant_pin = payload["room_pin"]
        o.room_pin = payload["room_pin"]
    await session.commit()
    await session.refresh(o)
    return _room_ser(o)

@router.delete("/{center_uuid}/rooms/{room_uuid}", status_code=204)
async def delete_room(center_uuid: UUID, room_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("conference_room_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(ConferenceRoom).where(ConferenceRoom.conference_room_uuid == room_uuid, ConferenceRoom.conference_center_uuid == center_uuid, ConferenceRoom.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Conference room not found")
    await session.delete(o)
    await session.commit()
    return Response(status_code=204)

# --- conference sessions child resources ---
def _session_ser(o: ConferenceSession):
    return {
        "conference_session_uuid": str(o.conference_session_uuid),
        "domain_id": str(o.domain_id),
        "conference_room_uuid": str(o.conference_room_uuid) if o.conference_room_uuid else None,
        "conference_center_uuid": str(o.conference_center_uuid) if o.conference_center_uuid else None,
        "profile": o.profile,
        "recording": o.recording,
        "start_epoch": o.start_epoch,
        "end_epoch": o.end_epoch,
        "created_at": o.created_at.isoformat() if o.created_at else None,
        "_links": {"self": {"href": f"/api/v1/conference_centers/{o.conference_center_uuid}/sessions/{o.conference_session_uuid}" if o.conference_center_uuid else f"/api/v1/conference_centers/sessions/{o.conference_session_uuid}"}, "collection": {"href": f"/api/v1/conference_centers/{o.conference_center_uuid}/sessions" if o.conference_center_uuid else "/api/v1/conference_centers/sessions"}},
    }

@router.get("/{center_uuid}/sessions", response_model=dict)
async def list_sessions(center_uuid: UUID, request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    query = select(ConferenceSession).where(ConferenceSession.domain_id == domain_id, ConferenceSession.conference_center_uuid == center_uuid)
    from sqlalchemy import func
    count_q = select(func.count()).select_from(query.subquery())
    total = (await session.execute(count_q)).scalar() or 0
    query = query.order_by(ConferenceSession.created_at.desc()).offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    response.headers["X-Total-Count"] = str(total)
    return {"data": [_session_ser(x) for x in items], "total": total, "page": page, "per_page": per_page}

@router.post("/{center_uuid}/sessions", response_model=dict, status_code=201)
async def create_session(center_uuid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("conference_session_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    obj = ConferenceSession(
        domain_id=domain_id,
        conference_center_uuid=center_uuid,
        conference_room_uuid=payload.get("conference_room_uuid"),
        profile=payload.get("profile"),
        recording=payload.get("recording"),
        start_epoch=payload.get("start_epoch"),
        end_epoch=payload.get("end_epoch"),
    )
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/conference_centers/{center_uuid}/sessions/{obj.conference_session_uuid}"
    return _session_ser(obj)

@router.get("/{center_uuid}/sessions/{session_uuid}", response_model=dict)
async def get_session_ep(center_uuid: UUID, session_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(ConferenceSession).where(ConferenceSession.conference_session_uuid == session_uuid, ConferenceSession.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Conference session not found")
    return _session_ser(o)

# --- conference session details (nested under sessions) ---
@router.get("/{center_uuid}/sessions/{session_uuid}/details", response_model=dict)
async def list_session_details(center_uuid: UUID, session_uuid: UUID, request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    query = select(ConferenceSessionDetail).where(ConferenceSessionDetail.domain_id == domain_id, ConferenceSessionDetail.conference_session_uuid == session_uuid)
    from sqlalchemy import func
    count_q = select(func.count()).select_from(query.subquery())
    total = (await session.execute(count_q)).scalar() or 0
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    response.headers["X-Total-Count"] = str(total)
    return {"data": [{"conference_session_detail_uuid": str(x.conference_session_detail_uuid), "conference_session_uuid": str(x.conference_session_uuid), "username": x.username, "caller_id_name": x.caller_id_name, "caller_id_number": x.caller_id_number} for x in items], "total": total, "page": page, "per_page": per_page}
