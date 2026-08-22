from typing import Optional
from fastapi import APIRouter, Depends, Query, Request, Response
from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session
from sqlalchemy.ext.asyncio import AsyncSession
router=APIRouter(prefix="/api/v1/registrations", tags=["registrations"])
@router.get("", response_model=dict)
async def list_regs(request: Request, response:Response, page:int=Query(1,ge=1), per_page:int=Query(20,ge=1,le=100), q:Optional[str]=Query(None), session:AsyncSession=Depends(get_session), user:User=Depends(get_current_user)):
    data=[]
    total=0
    try:
        from app.freeswitch.esl_client import esl_command
        raw=await esl_command("show registrations as json")
        import json
        if raw:
            j=json.loads(raw) if isinstance(raw, str) else raw
            rows=j.get("rows") or j.get("data") or []
            if q:
                rows=[r for r in rows if q.lower() in str(r).lower()]
            total=len(rows)
            start=(page-1)*per_page
            slice_rows=rows[start:start+per_page]
            data=[{"reg_user": r.get("reg_user") or r.get("user") or r.get("sip_user"), "realm": r.get("realm") or r.get("domain"), "network_ip": r.get("network_ip") or r.get("contact"), "status": r.get("status") or "Registered", "expires": r.get("expires")} for r in slice_rows]
    except Exception:
        pass
    response.headers["X-Total-Count"]=str(total)
    return {"data": data, "total": total, "page": page, "per_page": per_page}
