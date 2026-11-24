from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.number_translations.models import NumberTranslation, NumberTranslationDetail
from app.number_translations.schemas import NumberTranslationCreate, NumberTranslationRead, NumberTranslationUpdate, NumberTranslationDetailCreate, NumberTranslationDetailRead, NumberTranslationDetailUpdate
from app.number_translations.service import create_nt, list_nt
router=APIRouter(prefix="/api/v1/number_translations", tags=["number_translations"])
def _links(o): return {"self":{"href":f"/api/v1/number_translations/{o.number_translation_uuid}"}, "collection":{"href":"/api/v1/number_translations"}}
def _ser(o):
    d=NumberTranslationRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["number_translation_uuid"]=str(o.number_translation_uuid)
    return d
def _detail_links(parent_uuid: UUID, detail_uuid: UUID):
    base=f"/api/v1/number_translations/{parent_uuid}/details/{detail_uuid}"
    return {"self":{"href":base}, "collection":{"href":f"/api/v1/number_translations/{parent_uuid}/details"}, "parent":{"href":f"/api/v1/number_translations/{parent_uuid}"}}
def _detail_ser(o: NumberTranslationDetail):
    d=NumberTranslationDetailRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_detail_links(o.number_translation_uuid, o.number_translation_detail_uuid)
    d["number_translation_detail_uuid"]=str(o.number_translation_detail_uuid)
    d["number_translation_uuid"]=str(o.number_translation_uuid)
    d["domain_id"]=str(o.domain_id)
    # also provide alias fields for compat
    d["regex"]=d.get("number_translation_detail_regex")
    d["replace"]=d.get("number_translation_detail_replace")
    d["order"]=d.get("number_translation_detail_order")
    return d
async def _verify_parent(session: AsyncSession, domain_id: UUID, number_translation_uuid: UUID):
    r=await session.execute(select(NumberTranslation).where(NumberTranslation.number_translation_uuid==number_translation_uuid, NumberTranslation.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Number translation not found")
    return o
@router.get("", response_model=dict)
async def list_nt_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_nt(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_nt_ep(payload:NumberTranslationCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("number_translation_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_nt(session,payload,domain_id)
    response.headers["Location"]=f"/api/v1/number_translations/{obj.number_translation_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_nt(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(NumberTranslation).where(NumberTranslation.number_translation_uuid==uid, NumberTranslation.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Number translation not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_nt(uid:UUID, payload:NumberTranslationUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("number_translation_edit"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(NumberTranslation).where(NumberTranslation.number_translation_uuid==uid, NumberTranslation.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Number translation not found")
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
async def delete_nt(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("number_translation_delete"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(NumberTranslation).where(NumberTranslation.number_translation_uuid==uid, NumberTranslation.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Number translation not found")
    await session.delete(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)

# --- child: number_translation_details ---
@router.get("/{number_translation_uuid}/details", response_model=dict)
async def list_details(number_translation_uuid: UUID, session: AsyncSession=Depends(get_session), user: User=Depends(require_permission("number_translation_detail_view"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _verify_parent(session, domain_id, number_translation_uuid)
    r=await session.execute(select(NumberTranslationDetail).where(NumberTranslationDetail.number_translation_uuid==number_translation_uuid, NumberTranslationDetail.domain_id==domain_id).order_by(NumberTranslationDetail.number_translation_detail_order.asc(), NumberTranslationDetail.created_at.asc()))
    items=r.scalars().all()
    return {"data":[_detail_ser(x) for x in items], "total":len(items)}

@router.post("/{number_translation_uuid}/details", response_model=dict, status_code=201)
async def create_detail(number_translation_uuid: UUID, payload: NumberTranslationDetailCreate, response: Response, session: AsyncSession=Depends(get_session), user: User=Depends(require_permission("number_translation_detail_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _verify_parent(session, domain_id, number_translation_uuid)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    regex=payload.number_translation_detail_regex or payload.regex
    replace=payload.number_translation_detail_replace or payload.replace
    if not regex or replace is None:
        raise HTTPException(status_code=400, detail="regex and replace required")
    order_val=payload.number_translation_detail_order if payload.number_translation_detail_order is not None else (payload.order if payload.order is not None else 0)
    obj=NumberTranslationDetail(domain_id=domain_id, number_translation_uuid=number_translation_uuid, number_translation_detail_regex=str(regex), number_translation_detail_replace=str(replace), number_translation_detail_order=int(order_val))
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(obj)
    response.headers["Location"]=f"/api/v1/number_translations/{number_translation_uuid}/details/{obj.number_translation_detail_uuid}"
    return _detail_ser(obj)

@router.get("/{number_translation_uuid}/details/{detail_uuid}", response_model=dict)
async def get_detail(number_translation_uuid: UUID, detail_uuid: UUID, session: AsyncSession=Depends(get_session), user: User=Depends(require_permission("number_translation_detail_view"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _verify_parent(session, domain_id, number_translation_uuid)
    r=await session.execute(select(NumberTranslationDetail).where(NumberTranslationDetail.number_translation_detail_uuid==detail_uuid, NumberTranslationDetail.number_translation_uuid==number_translation_uuid, NumberTranslationDetail.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Number translation detail not found")
    return _detail_ser(o)

@router.patch("/{number_translation_uuid}/details/{detail_uuid}", response_model=dict)
async def patch_detail(number_translation_uuid: UUID, detail_uuid: UUID, payload: NumberTranslationDetailUpdate, session: AsyncSession=Depends(get_session), user: User=Depends(require_permission("number_translation_detail_edit"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _verify_parent(session, domain_id, number_translation_uuid)
    r=await session.execute(select(NumberTranslationDetail).where(NumberTranslationDetail.number_translation_detail_uuid==detail_uuid, NumberTranslationDetail.number_translation_uuid==number_translation_uuid, NumberTranslationDetail.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Number translation detail not found")
    data=payload.model_dump(exclude_unset=True)
    # map alias already handled by validator, but also handle if payload used raw dict keys via extra allow?
    for k,v in data.items():
        if k in ("regex",): continue
        if k=="number_translation_detail_regex" and v is not None:
            o.number_translation_detail_regex=str(v)
        elif k=="number_translation_detail_replace" and v is not None:
            o.number_translation_detail_replace=str(v)
        elif k=="number_translation_detail_order" and v is not None:
            o.number_translation_detail_order=int(v)
        elif k=="order" and v is not None:
            o.number_translation_detail_order=int(v)
        elif hasattr(o,k):
            setattr(o,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    await session.refresh(o)
    return _detail_ser(o)

@router.delete("/{number_translation_uuid}/details/{detail_uuid}", status_code=204)
async def delete_detail(number_translation_uuid: UUID, detail_uuid: UUID, session: AsyncSession=Depends(get_session), user: User=Depends(require_permission("number_translation_detail_delete"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _verify_parent(session, domain_id, number_translation_uuid)
    r=await session.execute(select(NumberTranslationDetail).where(NumberTranslationDetail.number_translation_detail_uuid==detail_uuid, NumberTranslationDetail.number_translation_uuid==number_translation_uuid, NumberTranslationDetail.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="Number translation detail not found")
    await session.delete(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception as _e:
        import logging as _lg
        _lg.getLogger("app.cache").warning("invalidate failed: %s", _e, exc_info=True)
    return Response(status_code=204)
