from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.access_controls.models import AccessControlNode
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
async def patch_ep(uid:UUID, payload:AccessControlUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("access_control_edit"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    acl,_=await get_with_nodes(session,uid,domain_id)
    if not acl: raise HTTPException(status_code=404, detail="Access control not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(acl,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(acl)
    _, nodes=await get_with_nodes(session,acl.access_control_uuid,domain_id)
    return _ser(acl, nodes)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("access_control_delete"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    acl,_=await get_with_nodes(session,uid,domain_id)
    if not acl: raise HTTPException(status_code=404, detail="Access control not found")
    await session.delete(acl)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)

# --- nodes sub-resource ---
@router.get("/{uid}/nodes", response_model=dict)
async def list_nodes(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    acl, nodes = await get_with_nodes(session, uid, domain_id)
    if not acl:
        raise HTTPException(status_code=404, detail="Access control not found")
    return {"data": [{"access_control_node_uuid": str(n.access_control_node_uuid), "access_control_uuid": str(n.access_control_uuid), "node_type": n.node_type, "node_cidr": n.node_cidr, "node_description": n.node_description, "created_at": n.created_at.isoformat() if n.created_at else None, "updated_at": n.updated_at.isoformat() if getattr(n, "updated_at", None) else None} for n in nodes], "total": len(nodes)}

@router.post("/{uid}/nodes", response_model=dict, status_code=201)
async def create_node(uid: UUID, payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("access_control_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    acl, _ = await get_with_nodes(session, uid, domain_id)
    if not acl:
        raise HTTPException(status_code=404, detail="Access control not found")
    node_type = payload.get("node_type") or payload.get("type") or "allow"
    node_cidr = payload.get("node_cidr") or payload.get("cidr")
    if not node_cidr:
        raise HTTPException(status_code=422, detail="node_cidr required")
    node_description = payload.get("node_description") or payload.get("description")
    node = AccessControlNode(access_control_uuid=uid, domain_id=domain_id, node_type=node_type, node_cidr=node_cidr, node_description=node_description)
    session.add(node)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(node)
    response.headers["Location"] = f"/api/v1/access_controls/{uid}/nodes/{node.access_control_node_uuid}"
    return {"access_control_node_uuid": str(node.access_control_node_uuid), "access_control_uuid": str(node.access_control_uuid), "node_type": node.node_type, "node_cidr": node.node_cidr, "node_description": node.node_description, "created_at": node.created_at.isoformat() if node.created_at else None, "updated_at": node.updated_at.isoformat() if getattr(node, "updated_at", None) else None, "_links": {"self": {"href": f"/api/v1/access_controls/{uid}/nodes/{node.access_control_node_uuid}"}, "collection": {"href": f"/api/v1/access_controls/{uid}/nodes"}}}
