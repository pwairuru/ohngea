from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.modules.models import Module
from app.modules.schemas import ModuleCreate, ModuleRead, ModuleUpdate
from app.modules.service import create_mod, list_mods
router=APIRouter(prefix="/api/v1/modules", tags=["modules"])
def _links(o): return {"self":{"href":f"/api/v1/modules/{o.module_uuid}"}, "collection":{"href":"/api/v1/modules"}}
def _ser(o):
    d=ModuleRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["module_uuid"]=str(o.module_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    items,total=await list_mods(session,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:ModuleCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("module_add"))):
    obj=await create_mod(session,payload)
    response.headers["Location"]=f"/api/v1/modules/{obj.module_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    r=await session.execute(select(Module).where(Module.module_uuid==uid))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Module not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid:UUID, payload:ModuleUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("module_edit"))):
    r=await session.execute(select(Module).where(Module.module_uuid==uid))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Module not found")
    for k,v in payload.model_dump(exclude_unset=True).items(): setattr(o,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache()
    except Exception:
        pass; await session.refresh(o); return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("module_delete"))):
    r=await session.execute(select(Module).where(Module.module_uuid==uid))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Module not found")
    await session.delete(o); await session.commit(); return Response(status_code=204)
