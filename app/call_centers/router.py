from typing import Optional
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.middleware.app_gate import require_app_enabled
from app.call_centers.models import CallCenter, CallCenterAgent, CallCenterTier
from app.call_centers.schemas import CallCenterCreate, CallCenterRead, CallCenterUpdate
from app.call_centers.service import create_cc, list_cc, get_cc_with_agents
router=APIRouter(prefix="/api/v1/call_centers", tags=["call_centers"], dependencies=[require_app_enabled("call_centers")])
def _links(o): return {"self":{"href":f"/api/v1/call_centers/{o.call_center_queue_uuid}"}, "collection":{"href":"/api/v1/call_centers"}}
def _ser(o, agents=None):
    d=CallCenterRead.model_validate(o, from_attributes=True).model_dump(mode="json")
    if agents is not None:
        d["agents"]=[{"call_center_agent_uuid":str(a.call_center_agent_uuid), "agent_name":a.agent_name, "agent_contact":a.agent_contact, "agent_status":a.agent_status} for a in agents]
    d["_links"]=_links(o)
    d["call_center_queue_uuid"]=str(o.call_center_queue_uuid)
    return d
@router.get("", response_model=dict)
async def list_cc_ep(request:Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    items,total=await list_cc(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    data=[]
    for cc in items:
        _, agents=await get_cc_with_agents(session, cc.call_center_queue_uuid, domain_id)
        data.append(_ser(cc, agents))
    return {"data":data,"total":total,"page":page,"per_page":per_page}
@router.post("", response_model=dict, status_code=201)
async def create_cc_ep(payload:CallCenterCreate, response:Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id)!=str(domain_id): raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session,domain_id)
    obj=await create_cc(session,payload,domain_id)
    _, agents=await get_cc_with_agents(session,obj.call_center_queue_uuid,domain_id)
    response.headers["Location"]=f"/api/v1/call_centers/{obj.call_center_queue_uuid}"
    return _ser(obj, agents)
@router.get("/{uid}", response_model=dict)
async def get_cc(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    cc, agents=await get_cc_with_agents(session,uid,domain_id)
    if not cc: raise HTTPException(status_code=404, detail="Call center not found")
    return _ser(cc, agents)
@router.patch("/{uid}", response_model=dict)
async def patch_cc(uid:UUID, payload:CallCenterUpdate, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    cc,_=await get_cc_with_agents(session,uid,domain_id)
    if not cc: raise HTTPException(status_code=404, detail="Call center not found")
    for k,v in payload.model_dump(exclude_unset=True).items():
        setattr(cc,k,v)
    await session.commit()
    await session.refresh(cc)
    _, agents=await get_cc_with_agents(session,cc.call_center_queue_uuid,domain_id)
    return _ser(cc, agents)
@router.delete("/{uid}", status_code=204)
async def delete_cc(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    cc,_=await get_cc_with_agents(session,uid,domain_id)
    if not cc: raise HTTPException(status_code=404, detail="Call center not found")
    await session.delete(cc)
    await session.commit()
    return Response(status_code=204)


def _agent_to_dict(a) -> dict:
    return {
        "call_center_agent_uuid": str(a.call_center_agent_uuid),
        "call_center_queue_uuid": str(a.call_center_queue_uuid),
        "domain_id": str(a.domain_id),
        "user_uuid": str(a.user_uuid) if a.user_uuid else None,
        "agent_name": a.agent_name,
        "agent_type": a.agent_type,
        "agent_contact": a.agent_contact,
        "agent_status": a.agent_status,
        "agent_call_timeout": a.agent_call_timeout,
        "agent_id": a.agent_id,
        "agent_max_no_answer": a.agent_max_no_answer,
        "agent_wrap_up_time": a.agent_wrap_up_time,
        "agent_reject_delay_time": a.agent_reject_delay_time,
        "agent_busy_delay_time": a.agent_busy_delay_time,
        "agent_no_answer_delay_time": a.agent_no_answer_delay_time,
        "agent_record": a.agent_record,
    }


def _tier_to_dict(t) -> dict:
    return {
        "call_center_tier_uuid": str(t.call_center_tier_uuid),
        "call_center_queue_uuid": str(t.call_center_queue_uuid),
        "call_center_agent_uuid": str(t.call_center_agent_uuid),
        "domain_id": str(t.domain_id),
        "agent_name": t.agent_name,
        "queue_name": t.queue_name,
        "tier_level": t.tier_level,
        "tier_position": t.tier_position,
    }


async def _get_cc_or_404(session, uid: UUID, domain_id):
    cc, _ = await get_cc_with_agents(session, uid, domain_id)
    if not cc:
        from fastapi import HTTPException as _HE
        raise _HE(status_code=404, detail="Call center not found")
    return cc


@router.get("/{uid}/agents", response_model=dict)
async def list_agents(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    r = await session.execute(select(CallCenterAgent).where(CallCenterAgent.call_center_queue_uuid==uid, CallCenterAgent.domain_id==domain_id).order_by(CallCenterAgent.agent_name))
    items = r.scalars().all()
    return {"data": [_agent_to_dict(a) for a in items], "total": len(items)}


@router.post("/{uid}/agents", response_model=dict, status_code=201)
async def create_agent(uid:UUID, payload: dict, response: Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    if not payload.get("agent_name"):
        raise HTTPException(status_code=422, detail="agent_name required")
    allowed={c.name for c in CallCenterAgent.__table__.columns} - {"call_center_agent_uuid","created_at","updated_at","domain_id","call_center_queue_uuid"}
    data={k:v for k,v in payload.items() if k in allowed}
    data["domain_id"]=domain_id
    data["call_center_queue_uuid"]=uid
    obj = CallCenterAgent(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return _agent_to_dict(obj)


@router.patch("/{uid}/agents/{agent_uuid}", response_model=dict)
async def patch_agent(uid:UUID, agent_uuid:UUID, payload: dict, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    r = await session.execute(select(CallCenterAgent).where(CallCenterAgent.call_center_agent_uuid==agent_uuid, CallCenterAgent.call_center_queue_uuid==uid))
    obj = r.scalar_one_or_none()
    if not obj: raise HTTPException(status_code=404, detail="Agent not found")
    allowed={c.name for c in CallCenterAgent.__table__.columns} - {"call_center_agent_uuid","created_at","updated_at","domain_id","call_center_queue_uuid"}
    _ints = {"agent_call_timeout","agent_max_no_answer","agent_wrap_up_time","agent_reject_delay_time","agent_busy_delay_time"}
    for k in allowed:
        if k in payload:
            v = payload[k]
            if k in _ints and v not in (None, ""):
                try: v = int(v)
                except (ValueError, TypeError): raise HTTPException(status_code=422, detail=f"{k} must be an integer")
            setattr(obj, k, v)
    await session.commit()
    await session.refresh(obj)
    return _agent_to_dict(obj)


@router.delete("/{uid}/agents/{agent_uuid}", status_code=204)
async def delete_agent(uid:UUID, agent_uuid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    r = await session.execute(select(CallCenterAgent).where(CallCenterAgent.call_center_agent_uuid==agent_uuid, CallCenterAgent.call_center_queue_uuid==uid))
    obj = r.scalar_one_or_none()
    if not obj: raise HTTPException(status_code=404, detail="Agent not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


@router.get("/{uid}/tiers", response_model=dict)
async def list_tiers(uid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    r = await session.execute(select(CallCenterTier).where(CallCenterTier.call_center_queue_uuid==uid, CallCenterTier.domain_id==domain_id).order_by(CallCenterTier.tier_level, CallCenterTier.tier_position))
    items = r.scalars().all()
    return {"data": [_tier_to_dict(t) for t in items], "total": len(items)}


@router.post("/{uid}/tiers", response_model=dict, status_code=201)
async def create_tier(uid:UUID, payload: dict, response: Response, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    if not payload.get("call_center_agent_uuid"):
        raise HTTPException(status_code=422, detail="call_center_agent_uuid required")
    allowed={c.name for c in CallCenterTier.__table__.columns} - {"call_center_tier_uuid","created_at","updated_at","domain_id","call_center_queue_uuid"}
    data={k:v for k,v in payload.items() if k in allowed}
    data["domain_id"]=domain_id
    data["call_center_queue_uuid"]=uid
    obj = CallCenterTier(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return _tier_to_dict(obj)


@router.patch("/{uid}/tiers/{tier_uuid}", response_model=dict)
async def patch_tier(uid:UUID, tier_uuid:UUID, payload: dict, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    r = await session.execute(select(CallCenterTier).where(CallCenterTier.call_center_tier_uuid==tier_uuid, CallCenterTier.call_center_queue_uuid==uid))
    obj = r.scalar_one_or_none()
    if not obj: raise HTTPException(status_code=404, detail="Tier not found")
    allowed={c.name for c in CallCenterTier.__table__.columns} - {"call_center_tier_uuid","created_at","updated_at","domain_id","call_center_queue_uuid"}
    for k in allowed:
        if k in payload:
            v = payload[k]
            if k in ("tier_level","tier_position") and v not in (None, ""):
                try: v = int(v)
                except (ValueError, TypeError): raise HTTPException(status_code=422, detail=f"{k} must be an integer")
            setattr(obj, k, v)
    await session.commit()
    await session.refresh(obj)
    return _tier_to_dict(obj)


@router.delete("/{uid}/tiers/{tier_uuid}", status_code=204)
async def delete_tier(uid:UUID, tier_uuid:UUID, session:AsyncSession=Depends(get_session), user:User=Depends(require_permission("call_center_add"))):
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session,domain_id)
    await _get_cc_or_404(session, uid, domain_id)
    r = await session.execute(select(CallCenterTier).where(CallCenterTier.call_center_tier_uuid==tier_uuid, CallCenterTier.call_center_queue_uuid==uid))
    obj = r.scalar_one_or_none()
    if not obj: raise HTTPException(status_code=404, detail="Tier not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)
