from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.vars.models import Var
from app.vars.schemas import VarCreate, VarRead, VarUpdate
from app.vars.service import create_var, list_vars
router=APIRouter(prefix="/api/v1/vars", tags=["vars"])
def _links(o): return {"self":{"href":f"/api/v1/vars/{o.var_uuid}"}, "collection":{"href":"/api/v1/vars"}}
def _ser(o):
    d=VarRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["var_uuid"]=str(o.var_uuid)
    return d
@router.get("", response_model=dict)
async def list_vars_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_vars(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_vars(payload:VarCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("var_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_var(session,payload,domain_id)
    response.headers["Location"]=f"/api/v1/vars/{obj.var_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_var(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(Var).where(Var.var_uuid==uid, Var.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Var not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_var(uid:UUID, payload:VarUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("var_edit"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(Var).where(Var.var_uuid==uid, Var.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Var not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(o,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(o)
    return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_var(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("var_delete"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(Var).where(Var.var_uuid==uid, Var.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Var not found")
    await session.delete(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
