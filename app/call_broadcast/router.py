from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.call_broadcast.models import CallBroadcast
from app.call_broadcast.schemas import CallBroadcastCreate, CallBroadcastRead, CallBroadcastUpdate
from app.call_broadcast.service import create_cb, list_cb
router=APIRouter(prefix="/api/v1/call_broadcasts", tags=["call_broadcasts"])
def _links(o): return {"self":{"href":f"/api/v1/call_broadcasts/{o.call_broadcast_uuid}"}, "collection":{"href":"/api/v1/call_broadcasts"}}
def _ser(o):
    d=CallBroadcastRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["call_broadcast_uuid"]=str(o.call_broadcast_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_cb(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:CallBroadcastCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_broadcast_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_cb(session,payload,domain_id)
    response.headers["Location"]=f"/api/v1/call_broadcasts/{obj.call_broadcast_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(CallBroadcast).where(CallBroadcast.call_broadcast_uuid==uid, CallBroadcast.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Call broadcast not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid:UUID, payload:CallBroadcastUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_broadcast_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(CallBroadcast).where(CallBroadcast.call_broadcast_uuid==uid, CallBroadcast.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Call broadcast not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(o,k,v)
    await session.commit()
    await session.refresh(o)
    return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_broadcast_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(CallBroadcast).where(CallBroadcast.call_broadcast_uuid==uid, CallBroadcast.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Call broadcast not found")
    await session.delete(o)
    await session.commit()
    return Response(status_code=204)
