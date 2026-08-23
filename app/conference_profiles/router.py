from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.conference_profiles.models import ConferenceProfile
from app.conference_profiles.schemas import ConferenceProfileCreate, ConferenceProfileRead, ConferenceProfileUpdate
from app.conference_profiles.service import create_profile, list_profiles, get_with_params
router=APIRouter(prefix="/api/v1/conference_profiles", tags=["conference_profiles"])
def _links(o): return {"self":{"href":f"/api/v1/conference_profiles/{o.conference_profile_uuid}"}, "collection":{"href":"/api/v1/conference_profiles"}}
def _ser(o, params=None):
    d=ConferenceProfileRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    if params is not None:
        d["params"]=[{"conference_profile_param_uuid":str(p.conference_profile_param_uuid), "profile_param_name":p.profile_param_name, "profile_param_value":p.profile_param_value} for p in params]
    d["_links"]=_links(o)
    d["conference_profile_uuid"]=str(o.conference_profile_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_profiles(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    data=[]
    for prof in items:
        _, params=await get_with_params(session, prof.conference_profile_uuid, domain_id)
        data.append(_ser(prof, params))
    return {"data":data,"total":total,"page":page,"per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:ConferenceProfileCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_profile_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_profile(session,payload,domain_id)
    _, params=await get_with_params(session,obj.conference_profile_uuid,domain_id)
    response.headers["Location"]=f"/api/v1/conference_profiles/{obj.conference_profile_uuid}"
    return _ser(obj, params)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    prof, params=await get_with_params(session,uid,domain_id)
    if not prof: raise HTTPException(status_code=404, detail="Conference profile not found")
    return _ser(prof, params)
@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid:UUID, payload:ConferenceProfileUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_profile_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    prof,_=await get_with_params(session,uid,domain_id)
    if not prof: raise HTTPException(status_code=404, detail="Conference profile not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(prof,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(prof)
    _, params=await get_with_params(session,prof.conference_profile_uuid,domain_id)
    return _ser(prof, params)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_profile_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    prof,_=await get_with_params(session,uid,domain_id)
    if not prof: raise HTTPException(status_code=404, detail="Conference profile not found")
    await session.delete(prof)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)
