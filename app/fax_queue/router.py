from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
from app.fax_queue.models import FaxQueue
from app.fax_queue.schemas import FaxQueueCreate, FaxQueueRead, FaxQueueUpdate
from app.fax_queue.service import create_fq, list_fq
router=APIRouter(prefix="/api/v1/fax_queue", tags=["fax_queue"])
def _links(o): return {"self":{"href":f"/api/v1/fax_queue/{o.fax_queue_uuid}"}, "collection":{"href":"/api/v1/fax_queue"}}
def _ser(o):
    d=FaxQueueRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["fax_queue_uuid"]=str(o.fax_queue_uuid)
    return d
@router.get("", response_model=dict)
async def list_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_fq(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    # Link pagination
    base_url=str(request.url).split("?")[0]
    from urllib.parse import urlencode
    links=[]
    if page*per_page<total:
        qs=dict(request.query_params); qs["page"]=str(page+1); qs["per_page"]=str(per_page)
        links.append(f'<{base_url}?{urlencode(qs)}>; rel="next"')
    if page>1:
        qs=dict(request.query_params); qs["page"]=str(page-1); qs["per_page"]=str(per_page)
        links.append(f'<{base_url}?{urlencode(qs)}>; rel="prev"')
    if links:
        response.headers["Link"]= ", ".join(links)
        response.headers["Access-Control-Expose-Headers"]="X-Total-Count, Link"
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_ep(payload:FaxQueueCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("fax_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_fq(session,payload,domain_id)
    response.headers["Location"]=f"/api/v1/fax_queue/{obj.fax_queue_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(FaxQueue).where(FaxQueue.fax_queue_uuid==uid, FaxQueue.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Fax queue not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_ep(uid:UUID, payload:FaxQueueUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("fax_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(FaxQueue).where(FaxQueue.fax_queue_uuid==uid, FaxQueue.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Fax queue not found")
    for k,v in payload.model_dump(exclude_unset=True).items(): setattr(o,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(o)
    return _ser(o)

@router.get("/{uid}/status", response_model=dict)
async def status_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    """mod_spandsp fax status via ESL (mock fallback)."""
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(FaxQueue).where(FaxQueue.fax_queue_uuid==uid, FaxQueue.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Fax queue not found")
    # ESL query best-effort
    esl_detail = "mock: no ESL"
    try:
        from app.freeswitch.esl_client import esl_client
        resp = await esl_client.api(f"fax status {uid}")
        esl_detail = resp[:500] if resp else "no response"
    except Exception as e:
        esl_detail = f"mock error: {e}"[:500]
    return {"fax_queue_uuid": str(o.fax_queue_uuid), "fax_status": o.fax_status, "fax_number": o.fax_number, "fax_file": o.fax_file, "esl": esl_detail, "_links": _links(o)}

@router.post("/{uid}/retry", response_model=dict)
async def retry_fax_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("fax_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(FaxQueue).where(FaxQueue.fax_queue_uuid==uid, FaxQueue.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Fax queue not found")
    if o.fax_status == "failed":
        o.fax_status = "waiting"
        if hasattr(o, "fax_retry_count"):
            o.fax_retry_count = (o.fax_retry_count or 0) + 1
        await session.commit()
        await session.refresh(o)
    try:
        from app.workers.celery_app import fax_send_task
        fax_send_task.delay(str(o.fax_queue_uuid))
    except Exception:
        pass
    return _ser(o)

@router.delete("/{uid}", status_code=204)
async def delete_ep(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("fax_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(FaxQueue).where(FaxQueue.fax_queue_uuid==uid, FaxQueue.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="Fax queue not found")
    await session.delete(o); await session.commit(); return Response(status_code=204)
