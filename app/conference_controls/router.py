from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.conference_controls.schemas import ConferenceControlCreate, ConferenceControlRead, ConferenceControlUpdate
from app.conference_controls.service import create_control, list_controls, get_with_details
router=APIRouter(prefix="/api/v1/conference_controls", tags=["conference_controls"])
def _links(o): return {"self":{"href":f"/api/v1/conference_controls/{o.conference_control_uuid}"}, "collection":{"href":"/api/v1/conference_controls"}}
def _ser(o, details=None):
    d=ConferenceControlRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    if details is not None:
        d["details"]=[{"conference_control_detail_uuid":str(x.conference_control_detail_uuid), "control_digits":x.control_digits, "control_action":x.control_action} for x in details]
    d["_links"]=_links(o)
    d["conference_control_uuid"]=str(o.conference_control_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_controls(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    data=[]
    for ctrl in items:
        _, details=await get_with_details(session, ctrl.conference_control_uuid, domain_id)
        data.append(_ser(ctrl, details))
    return {"data":data,"total":total,"page":page,"per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:ConferenceControlCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_control_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_control(session,payload,domain_id)
    _, details=await get_with_details(session,obj.conference_control_uuid,domain_id)
    response.headers["Location"]=f"/api/v1/conference_controls/{obj.conference_control_uuid}"
    return _ser(obj, details)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    ctrl, details=await get_with_details(session,uid,domain_id)
    if not ctrl: raise HTTPException(status_code=404, detail="Conference control not found")
    return _ser(ctrl, details)
@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid:UUID, payload:ConferenceControlUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_control_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    ctrl,_=await get_with_details(session,uid,domain_id)
    if not ctrl: raise HTTPException(status_code=404, detail="Conference control not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(ctrl,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(ctrl)
    _, details=await get_with_details(session,ctrl.conference_control_uuid,domain_id)
    return _ser(ctrl, details)
@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("conference_control_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    ctrl,_=await get_with_details(session,uid,domain_id)
    if not ctrl: raise HTTPException(status_code=404, detail="Conference control not found")
    await session.delete(ctrl)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
