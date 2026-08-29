from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache_for_session
from app.middleware.app_gate import require_app_enabled
from app.fifo.models import Fifo, FifoMember
from app.fifo.schemas import FifoCreate, FifoRead, FifoUpdate, FifoMemberCreate, FifoMemberRead, FifoMemberUpdate
from app.fifo.service import create_fifo, list_fifo
router=APIRouter(prefix="/api/v1/fifo", tags=["fifo"], dependencies=[require_app_enabled("fifo")])
def _links(o): return {"self":{"href":f"/api/v1/fifo/{o.fifo_uuid}"}, "collection":{"href":"/api/v1/fifo"}}
def _ser(o):
    d=FifoRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"]=_links(o)
    d["fifo_uuid"]=str(o.fifo_uuid)
    return d
@router.get("", response_model=dict)
async def list_fifo_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_fifo(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data":[_ser(x) for x in items], "total":total, "page":page, "per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_fifo_ep(payload:FifoCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("fifo_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_fifo(session,payload,domain_id)
    response.headers["Location"]=f"/api/v1/fifo/{obj.fifo_uuid}"
    return _ser(obj)
@router.get("/{uid}", response_model=dict)
async def get_fifo(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(Fifo).where(Fifo.fifo_uuid==uid, Fifo.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="FIFO not found")
    return _ser(o)
@router.patch("/{uid}", response_model=dict)
async def patch_fifo(uid:UUID, payload:FifoUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("fifo_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(Fifo).where(Fifo.fifo_uuid==uid, Fifo.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="FIFO not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(o,k,v)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(o)
    return _ser(o)
@router.delete("/{uid}", status_code=204)
async def delete_fifo(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("fifo_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    r=await session.execute(select(Fifo).where(Fifo.fifo_uuid==uid, Fifo.domain_id==domain_id))
    o=r.scalar_one_or_none()
    if not o: raise HTTPException(status_code=404, detail="FIFO not found")
    await session.delete(o)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return Response(status_code=204)

def _member_links(fifo_uuid: UUID, member_uuid: UUID):
    return {"self": {"href": f"/api/v1/fifo/{fifo_uuid}/members/{member_uuid}"}, "collection": {"href": f"/api/v1/fifo/{fifo_uuid}/members"}, "fifo": {"href": f"/api/v1/fifo/{fifo_uuid}"}}

def _ser_member(o: FifoMember):
    d = FifoMemberRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    d["_links"] = _member_links(o.fifo_uuid, o.fifo_member_uuid)
    d["fifo_member_uuid"] = str(o.fifo_member_uuid)
    d["fifo_uuid"] = str(o.fifo_uuid)
    d["domain_id"] = str(o.domain_id)
    return d

@router.get("/{fifo_uuid}/members", response_model=dict)
async def list_fifo_members(fifo_uuid: UUID, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fifo_member_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Fifo).where(Fifo.fifo_uuid == fifo_uuid, Fifo.domain_id == domain_id))
    fifo = r.scalar_one_or_none()
    if not fifo:
        raise HTTPException(status_code=404, detail="FIFO not found")
    m = await session.execute(select(FifoMember).where(FifoMember.fifo_uuid == fifo_uuid, FifoMember.domain_id == domain_id).order_by(FifoMember.created_at))
    items = m.scalars().all()
    response.headers["X-Total-Count"] = str(len(items))
    return {"data": [_ser_member(x) for x in items], "total": len(items), "page": 1, "per_page": len(items) or 20}

@router.post("/{fifo_uuid}/members", response_model=dict, status_code=201)
async def create_fifo_member(fifo_uuid: UUID, payload: FifoMemberCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fifo_member_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Fifo).where(Fifo.fifo_uuid == fifo_uuid, Fifo.domain_id == domain_id))
    fifo = r.scalar_one_or_none()
    if not fifo:
        raise HTTPException(status_code=404, detail="FIFO not found")
    data = payload.model_dump(exclude_unset=True, exclude_none=True)
    # enforce domain/fifo consistency
    if data.get("domain_id") and str(data["domain_id"]) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    if data.get("fifo_uuid") and str(data["fifo_uuid"]) != str(fifo_uuid):
        raise HTTPException(status_code=403, detail="FIFO mismatch")
    obj = FifoMember(domain_id=domain_id, fifo_uuid=fifo_uuid, member_contact=data.get("member_contact"), member_call_timeout=data.get("member_call_timeout"), member_simultaneous=data.get("member_simultaneous"), member_wrap_up_time=data.get("member_wrap_up_time"), member_enabled=data.get("member_enabled", True))
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    response.headers["Location"] = f"/api/v1/fifo/{fifo_uuid}/members/{obj.fifo_member_uuid}"
    return _ser_member(obj)

@router.patch("/{fifo_uuid}/members/{member_uuid}", response_model=dict)
async def patch_fifo_member(fifo_uuid: UUID, member_uuid: UUID, payload: FifoMemberUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fifo_member_edit"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(FifoMember).where(FifoMember.fifo_member_uuid == member_uuid, FifoMember.fifo_uuid == fifo_uuid, FifoMember.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="FIFO member not found")
    for k, v in payload.model_dump(exclude_unset=True).items():
        setattr(o, k, v)
    await session.commit()
    await session.refresh(o)
    return _ser_member(o)

@router.delete("/{fifo_uuid}/members/{member_uuid}", status_code=204)
async def delete_fifo_member(fifo_uuid: UUID, member_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("fifo_member_delete"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(FifoMember).where(FifoMember.fifo_member_uuid == member_uuid, FifoMember.fifo_uuid == fifo_uuid, FifoMember.domain_id == domain_id))
    o = r.scalar_one_or_none()
    if not o:
        raise HTTPException(status_code=404, detail="FIFO member not found")
    await session.delete(o)
    await session.commit()
    return Response(status_code=204)
