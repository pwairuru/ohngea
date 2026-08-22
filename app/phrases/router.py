from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.phrases.models import Phrase, PhraseDetail
from app.phrases.schemas import PhraseCreate, PhraseRead, PhraseUpdate
from app.phrases.service import create_phrase, list_phrases, get_phrase_with_details
router=APIRouter(prefix="/api/v1/phrases", tags=["phrases"])
def _links(o: Phrase):
    return {"self":{"href":f"/api/v1/phrases/{o.phrase_uuid}"}, "collection":{"href":"/api/v1/phrases"}}
def _ser(o: Phrase, details=None):
    d=PhraseRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    if details is not None:
        d["details"]=[{"phrase_detail_uuid":str(x.phrase_detail_uuid), "phrase_detail_function":x.phrase_detail_function, "phrase_detail_data":x.phrase_detail_data, "phrase_detail_order":x.phrase_detail_order} for x in details]
    d["_links"]=_links(o)
    d["phrase_uuid"]=str(o.phrase_uuid)
    return d
@router.get("", response_model=dict)
async def list_ph(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_phrases(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    data=[]
    for ph in items:
        _, dets=await get_phrase_with_details(session, ph.phrase_uuid, domain_id)
        data.append(_ser(ph, dets))
    return {"data":data,"total":total,"page":page,"per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ph(payload:PhraseCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("phrase_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_phrase(session,payload,domain_id)
    _, dets=await get_phrase_with_details(session,obj.phrase_uuid,domain_id)
    response.headers["Location"]=f"/api/v1/phrases/{obj.phrase_uuid}"
    return _ser(obj,dets)
@router.get("/{uid}", response_model=dict)
async def get_ph(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    ph, dets=await get_phrase_with_details(session,uid,domain_id)
    if not ph:
        raise HTTPException(status_code=404, detail="Phrase not found")
    return _ser(ph,dets)
@router.patch("/{uid}", response_model=dict)
async def patch_ph(uid:UUID, payload:PhraseUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("phrase_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    ph, _=await get_phrase_with_details(session,uid,domain_id)
    if not ph:
        raise HTTPException(status_code=404, detail="Phrase not found")
    for k,v in payload.model_dump(exclude_unset=True, exclude={"details"}).items():
        setattr(ph, k if hasattr(ph,k) else k, v)
    if payload.details is not None:
        existing=await session.execute(select(PhraseDetail).where(PhraseDetail.phrase_uuid==uid))
        for e in existing.scalars().all():
            await session.delete(e)
        await session.flush()
        for d in payload.details:
            if isinstance(d, dict):
                det=PhraseDetail(domain_id=domain_id, phrase_uuid=ph.phrase_uuid, phrase_detail_function=d.get("function"), phrase_detail_data=d.get("data"), phrase_detail_order=int(d.get("order",0) or 0))
                session.add(det)
    await session.commit()
    await session.refresh(ph)
    _, dets=await get_phrase_with_details(session,ph.phrase_uuid,domain_id)
    return _ser(ph,dets)
@router.delete("/{uid}", status_code=204)
async def delete_ph(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("phrase_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    ph, _=await get_phrase_with_details(session,uid,domain_id)
    if not ph:
        raise HTTPException(status_code=404, detail="Phrase not found")
    await session.delete(ph)
    await session.commit()
    return Response(status_code=204)
