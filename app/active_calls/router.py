from typing import Optional
from fastapi import APIRouter, Depends, Query, Request, Response
from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session
from sqlalchemy.ext.asyncio import AsyncSession
router=APIRouter(prefix="/api/v1/active_calls", tags=["active_calls"])
@router.get("", response_model=dict)
async def list_active_calls(request: Request, response: Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    # Live FS view — try ESL, fallback empty (no DB)
    data=[]
    total=0
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command("show channels as json")
        import json
        if raw:
            j=json.loads(raw) if isinstance(raw, str) else raw
            rows=j.get("rows") or j.get("data") or []
            # filter by q if present
            if q:
                rows=[r for r in rows if q.lower() in str(r).lower()]
            total=len(rows)
            # paginate
            start=(page-1)*per_page
            data=rows[start:start+per_page]
            # normalize to our schema
            data=[{"call_uuid": r.get("uuid") or r.get("call_uuid") or str(i), "direction": r.get("direction") or "inbound", "cid_num": r.get("cid_num") or r.get("caller_id_number"), "dest": r.get("dest") or r.get("destination_number"), "state": r.get("state") or r.get("callstate") } for i,r in enumerate(data)]
    except Exception:
        pass
    response.headers["X-Total-Count"]=str(total)
    return {"data": data, "total": total, "page": page, "per_page": per_page}
@router.get("/{uid}", response_model=dict)
async def get_call(uid: str, session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    # try ESL uuid_exists
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command(f"uuid_exists {uid}")
        if raw and "true" in str(raw).lower():
            return {"call_uuid": uid, "exists": True, "_links": {"self": {"href": f"/api/v1/active_calls/{uid}"}, "collection": {"href": "/api/v1/active_calls"}}}
    except Exception:
        pass
    from fastapi import HTTPException
    raise HTTPException(status_code=404, detail="Active call not found")
