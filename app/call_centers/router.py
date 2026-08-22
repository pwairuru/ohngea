from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.call_centers.models import CallCenter, CallCenterAgent
from app.call_centers.schemas import CallCenterCreate, CallCenterRead, CallCenterUpdate
from app.call_centers.service import create_cc, list_cc, get_cc_with_agents
router=APIRouter(prefix="/api/v1/call_centers", tags=["call_centers"])
def _links(o): return {"self":{"href":f"/api/v1/call_centers/{o.call_center_queue_uuid}"}, "collection":{"href":"/api/v1/call_centers"}}
def _ser(o, agents=None):
    d=CallCenterRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    if agents is not None:
        d["agents"]=[{"call_center_agent_uuid":str(a.call_center_agent_uuid), "agent_name":a.agent_name, "agent_contact":a.agent_contact, "agent_status":a.agent_status} for a in agents]
    d["_links"]=_links(o)
    d["call_center_queue_uuid"]=str(o.call_center_queue_uuid)
    return d
@router.get("", response_model=dict)
async def list_cc_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_cc(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    data=[]
    for cc in items:
        _, agents=await get_cc_with_agents(session, cc.call_center_queue_uuid, domain_id)
        data.append(_ser(cc, agents))
    return {"data":data,"total":total,"page":page,"per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_cc_ep(payload:CallCenterCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_cc(session,payload,domain_id)
    _, agents=await get_cc_with_agents(session,obj.call_center_queue_uuid,domain_id)
    response.headers["Location"]=f"/api/v1/call_centers/{obj.call_center_queue_uuid}"
    return _ser(obj, agents)
@router.get("/{uid}", response_model=dict)
async def get_cc(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    cc, agents=await get_cc_with_agents(session,uid,domain_id)
    if not cc: raise HTTPException(status_code=404, detail="Call center not found")
    return _ser(cc, agents)
@router.patch("/{uid}", response_model=dict)
async def patch_cc(uid:UUID, payload:CallCenterUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    cc,_=await get_cc_with_agents(session,uid,domain_id)
    if not cc: raise HTTPException(status_code=404, detail="Call center not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(cc,k,v)
    await session.commit()
    await session.refresh(cc)
    _, agents=await get_cc_with_agents(session,cc.call_center_queue_uuid,domain_id)
    return _ser(cc, agents)
@router.delete("/{uid}", status_code=204)
async def delete_cc(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    cc,_=await get_cc_with_agents(session,uid,domain_id)
    if not cc: raise HTTPException(status_code=404, detail="Call center not found")
    await session.delete(cc)
    await session.commit()
    return Response(status_code=204)
