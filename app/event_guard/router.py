from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.event_guard.models import EventGuardLog
from app.event_guard.schemas import EventGuardLogCreate, EventGuardLogRead
from app.event_guard.service import create_eg, list_eg
router=APIRouter(prefix="/api/v1/event_guard", tags=["event_guard"])
def _links(o): return {"self":{"href":f"/api/v1/event_guard/{o.event_guard_log_uuid}"}, "collection":{"href":"/api/v1/event_guard"}}
def _ser(o):
    d=EventGuardLogRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["event_guard_log_uuid"]=str(o.event_guard_log_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    items,total=await list_eg(session,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:EventGuardLogCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("event_guard_add"))):
    obj=await create_eg(session,payload)
    response.headers["Location"]=f"/api/v1/event_guard/{obj.event_guard_log_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    r=await session.execute(select(EventGuardLog).where(EventGuardLog.event_guard_log_uuid==uid))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Not found")
    return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("event_guard_add"))):
    r=await session.execute(select(EventGuardLog).where(EventGuardLog.event_guard_log_uuid==uid))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Not found")
    await session.delete(o); await session.commit(); return Response(status_code=204)
