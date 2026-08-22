from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.access_controls.schemas import AccessControlCreate, AccessControlRead, AccessControlUpdate
from app.access_controls.service import create_acl, list_acls, get_with_nodes
router=APIRouter(prefix="/api/v1/access_controls", tags=["access_controls"])
def _links(o): return {"self":{"href":f"/api/v1/access_controls/{o.access_control_uuid}"}, "collection":{"href":"/api/v1/access_controls"}}
def _ser(o, nodes=None):
    d=AccessControlRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    if nodes is not None:
        d["nodes"]=[{"access_control_node_uuid":str(n.access_control_node_uuid), "node_type":n.node_type, "node_cidr":n.node_cidr} for n in nodes]
    d["_links"]=_links(o)
    d["access_control_uuid"]=str(o.access_control_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_acls(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    data=[]
    for acl in items:
        _, nodes=await get_with_nodes(session, acl.access_control_uuid, domain_id)
        data.append(_ser(acl, nodes))
    return {"data":data,"total":total,"page":page,"per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:AccessControlCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("access_control_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_acl(session,payload,domain_id)
    _, nodes=await get_with_nodes(session,obj.access_control_uuid,domain_id)
    response.headers["Location"]=f"/api/v1/access_controls/{obj.access_control_uuid}"
    return _ser(obj, nodes)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    acl, nodes=await get_with_nodes(session,uid,domain_id)
    if not acl: raise HTTPException(status_code=404, detail="Access control not found")
    return _ser(acl, nodes)
@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid:UUID, payload:AccessControlUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("access_control_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    acl,_=await get_with_nodes(session,uid,domain_id)
    if not acl: raise HTTPException(status_code=404, detail="Access control not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(acl,k,v)
    await session.commit()
    await session.refresh(acl)
    _, nodes=await get_with_nodes(session,acl.access_control_uuid,domain_id)
    return _ser(acl, nodes)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("access_control_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    acl,_=await get_with_nodes(session,uid,domain_id)
    if not acl: raise HTTPException(status_code=404, detail="Access control not found")
    await session.delete(acl)
    await session.commit()
    return Response(status_code=204)
