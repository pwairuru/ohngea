from typing import Optional
from fastapi import APIRouter, Depends, Query, Request, Response
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant

router = APIRouter(tags=["aliases"])

# alias /api/v1/sip_status -> same as /api/v1/sip_profiles/status but shorter
@router.get("/api/v1/sip_status", response_model=dict)
async def sip_status_alias(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    from app.freeswitch.status_router import sip_profiles_status
    return await sip_profiles_status(session=session, user=user)

# alias /api/v1/calls_active -> same as active_calls
@router.get("/api/v1/calls_active", response_model=dict)
async def calls_active_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from app.active_calls.router import list_active_calls
    return await list_active_calls(request=request, response=response, page=page, per_page=per_page, q=q, session=session, user=user)

# alias /api/v1/active_conferences + /api/v1/conferences_active -> same as conferences
@router.get("/api/v1/active_conferences", response_model=dict)
@router.get("/api/v1/conferences_active", response_model=dict)
async def active_conferences_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    # Try live FS conference list, fallback to DB conferences
    data=[]
    total=0
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command("conference list as json")
        import json
        if raw:
            j=json.loads(raw) if isinstance(raw, str) else raw
            rows=j.get("rows") or j.get("conferences") or j.get("data") or []
            if isinstance(rows, dict):
                rows=list(rows.values())
            if q:
                rows=[r for r in rows if q.lower() in str(r).lower()]
            total=len(rows)
            start=(page-1)*per_page
            data=rows[start:start+per_page]
            # normalise
            norm=[]
            for r in data:
                if isinstance(r, dict):
                    norm.append({"conference_name": r.get("conference_name") or r.get("name") or str(r), "member_count": r.get("member_count") or r.get("members") or 0, "profile": r.get("profile")})
                else:
                    norm.append({"conference_name": str(r)})
            data=norm
        else:
            # fallback to DB conferences
            from app.conferences.service import list_conferences
            from app.db import get_session, set_tenant as _gs
            # need domain_id
            domain_id=getattr(user,"_domain_id", user.domain_id)
            from app.db import set_tenant
            await set_tenant(session, domain_id)
            items,total=await list_conferences(session,domain_id,page,per_page,q)
            data=[{"conference_name": c.conference_name, "conference_extension": c.conference_extension, "conference_uuid": str(c.conference_uuid)} for c in items]
    except Exception:
        # fallback empty
        pass
    response.headers["X-Total-Count"]=str(total)
    return {"data": data, "total": total, "page": page, "per_page": per_page}

# switch + system status (infra) — system/status now handled by app.system.router for full metrics
@router.get("/api/v1/switch/status", response_model=dict)
async def switch_system_status(user: User = Depends(get_current_user)):
    from app.freeswitch.status_router import _get_fs_status_cached
    fs=await _get_fs_status_cached()
    import platform
    try:
        import os
        load=os.getloadavg()[0] if hasattr(os, 'getloadavg') else None
    except: load=None
    return {"freeswitch": fs, "system": {"hostname": platform.node(), "load_avg": load}, "_links": {"self": {"href": "/api/v1/switch/status"}, "system": {"href": "/api/v1/system/status"}}}

# dialplan inbound/outbound aliases filtered wrappers
@router.get("/api/v1/dialplan_inbound", response_model=dict)
async def dialplan_inbound_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from app.dialplans.router import list_dialplans as _list
    from sqlalchemy import select
    from app.dialplans.models import Dialplan
    from app.db import set_tenant
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # simple list via direct query (dialplan router already has pagination)
    from sqlalchemy import func, or_
    query=select(Dialplan).where(Dialplan.domain_id==domain_id)
    total=(await session.execute(select(func.count()).select_from(Dialplan).where(Dialplan.domain_id==domain_id))).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    inbound=[d for d in items if "inbound" in (d.name or "").lower() or "inbound" in (getattr(d,"context", "") or "").lower()]
    data=inbound if inbound else items
    total_inb=len(inbound) if inbound else total
    response.headers["X-Total-Count"]=str(total_inb)
    return {"data": [{"dialplan_uuid": str(d.dialplan_uuid), "name": d.name, "context": getattr(d,"context",None)} for d in data], "total": total_inb, "page": page, "per_page": per_page}

@router.get("/api/v1/dialplan_outbound", response_model=dict)
async def dialplan_outbound_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from sqlalchemy import select, func
    from app.dialplans.models import Dialplan
    from app.db import set_tenant
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    query=select(Dialplan).where(Dialplan.domain_id==domain_id)
    total=(await session.execute(select(func.count()).select_from(Dialplan).where(Dialplan.domain_id==domain_id))).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    outbound=[d for d in items if "outbound" in (d.name or "").lower() or "outbound" in (getattr(d,"context","") or "").lower()]
    data=outbound if outbound else items
    total_out=len(outbound) if outbound else total
    response.headers["X-Total-Count"]=str(total_out)
    return {"data": [{"dialplan_uuid": str(d.dialplan_uuid), "name": d.name, "context": getattr(d,"context",None)} for d in data], "total": total_out, "page": page, "per_page": per_page}

# call_center_active + fifo_list live aliases (legacy active queues)
@router.get("/api/v1/call_center_active", response_model=dict)
@router.get("/api/v1/call_center_active/{queue}", response_model=dict)
async def call_center_active_alias(request: Request, response: Response, queue: Optional[str]=None, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    # Try ESL callcenter_config list agents, fallback to DB call_centers
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command("callcenter_config list agents" if not queue else f"callcenter_config list agents {queue}")
        import json
        if raw and "agents" in raw.lower():
            # parse ESL json if available
            data=[{"agent": raw[:200]}]
            total=1
            response.headers["X-Total-Count"]=str(total)
            return {"data": data, "total": total, "page": page, "per_page": per_page, "_links": {"self": {"href": "/api/v1/call_center_active"}}}
    except Exception:
        pass
    from app.call_centers.service import list_cc
    from app.db import set_tenant
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items,total=await list_cc(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data": [{"queue_name": c.queue_name, "queue_extension": c.queue_extension, "queue_strategy": getattr(c,"queue_strategy",None)} for c in items], "total": total, "page": page, "per_page": per_page}

@router.get("/api/v1/fifo_list", response_model=dict)
async def fifo_list_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from app.fifo.service import list_fifo
    from app.db import set_tenant
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items,total=await list_fifo(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data": [{"fifo_name": c.fifo_name, "fifo_extension": getattr(c,"fifo_extension",None)} for c in items], "total": total, "page": page, "per_page": per_page}

# call_forward -> follow_me (call_forward.php uses follow_me table)
@router.get("/api/v1/call_forward", response_model=dict)
async def call_forward_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from app.follow_me.service import list_follow_me
    from app.db import set_tenant
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items,total=await list_follow_me(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data": [{"follow_me_uuid": str(c.follow_me_uuid), "extension": getattr(c,"extension",None), "follow_me_enabled": getattr(c,"follow_me_enabled",None)} for c in items], "total": total, "page": page, "per_page": per_page}

# call_recordings -> recordings (call_recordings uses recordings + cdr)
@router.get("/api/v1/call_recordings", response_model=dict)
async def call_recordings_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from app.recordings.service import list_recordings
    from app.db import set_tenant
    domain_id=getattr(user,"_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items,total=await list_recordings(session,domain_id,page,per_page,q)
    response.headers["X-Total-Count"]=str(total)
    return {"data": [{"recording_uuid": str(c.recording_uuid), "name": c.name, "file_path": c.file_path} for c in items], "total": total, "page": page, "per_page": per_page}

# click_to_call -> operator_panel originate helper (legacy click_to_call.php dials via ESL originate)
@router.get("/api/v1/click_to_call", response_model=dict)
async def click_to_call_get(session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    return {"message": "POST /api/v1/click_to_call with {extension, number} to originate", "_links": {"originate": {"href": "/api/v1/operator_panel/originate"}, "self": {"href": "/api/v1/click_to_call"}}}

@router.post("/api/v1/click_to_call", response_model=dict)
async def click_to_call_post(payload: dict, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    # delegate to operator_panel originate
    from app.operator_panel.router import originate as _orig
    ext=payload.get("extension") or payload.get("src") or "1001"
    num=payload.get("number") or payload.get("dest") or payload.get("destination") or "1002"
    return await _orig(payload={"source": ext, "destination": num}, session=session, user=user)

# xml_cdr legacy alias -> cdr (legacy uses v_xml_cdr view, modern uses cdr HASH32)
@router.get("/api/v1/xml_cdr", response_model=dict)
async def xml_cdr_list_alias(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), filter_extension: Optional[str]=Query(None, alias="filter[extension]"), filter_caller: Optional[str]=Query(None, alias="filter[caller_id_number]"), sort: str=Query("start_stamp"), order: str=Query("desc"), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from app.cdr.router import list_cdr
    return await list_cdr(request=request, response=response, page=page, per_page=per_page, filter_extension=filter_extension, filter_caller=filter_caller, sort=sort, order=order, q=q, session=session, user=user)

@router.get("/api/v1/xml_cdr/{cdr_uuid}", response_model=dict)
async def xml_cdr_get_alias(cdr_uuid: str, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    from app.cdr.router import get_cdr
    from uuid import UUID
    return await get_cdr(cdr_uuid=UUID(cdr_uuid), session=session, user=user)
