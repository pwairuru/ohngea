from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, Query, Request, Response, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.emergency_logs.models import EmergencyLog
from app.emergency_logs.schemas import EmergencyLogCreate, EmergencyLogRead
from app.emergency_logs.service import create_log, list_logs
router=APIRouter(prefix="/api/v1/emergency_logs", tags=["emergency_logs"])
def _links(o): return {"self":{"href":f"/api/v1/emergency_logs/{o.emergency_log_uuid}"}, "collection":{"href":"/api/v1/emergency_logs"}}
def _ser(o):
    d=EmergencyLogRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["emergency_log_uuid"]=str(o.emergency_log_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_logs(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:EmergencyLogCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("emergency_logs_view"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_log(session,payload,domain_id)
    response.headers["Location"]=f"/api/v1/emergency_logs/{obj.emergency_log_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(EmergencyLog).where(EmergencyLog.emergency_log_uuid==uid, EmergencyLog.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Emergency log not found")
    return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("emergency_logs_view"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(EmergencyLog).where(EmergencyLog.emergency_log_uuid==uid, EmergencyLog.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Emergency log not found")
    await session.delete(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
