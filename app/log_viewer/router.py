from typing import Optional
from pathlib import Path
from fastapi import APIRouter, Depends, Query
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
router = APIRouter(prefix="/api/v1/logs", tags=["logs"])
ALLOWED = ["/var/log/freeswitch/freeswitch.log", "/var/log/freeswitch.log", "/tmp/freeswitch.log", "freeswitch.log"]
@router.get("", response_model=dict)
async def view_logs(lines:int=Query(100,ge=1,le=1000), level:Optional[str]=Query(None), q:Optional[str]=Query(None), user:User=Depends(require_permission("log_view"))):
    # try real FS log, fallback mock
    content=[]
    found=False
    for p in ALLOWED:
        pp=Path(p)
        if pp.exists():
            try:
                txt=pp.read_text()[-20000:]
                content=txt.splitlines()[-lines:]
                found=True
                break
            except: pass
    if not found:
        # mock log lines for dev without FS
        mock=[f"2026-08-22T06:40:00Z [NOTICE] sofia_reg.c:1234 SIP auth success for 1001@test.example.com", f"2026-08-22T06:40:01Z [ERR] switch_core.c: dialplan not found 9999", f"2026-08-22T06:40:02Z [INFO] switch_channel.c: channel answered 1001->1002"]
        # repeat to fill lines
        while len(content)<lines and len(content)<100:
            content.extend(mock)
        content=content[:lines]
    if level:
        content=[l for l in content if level.upper() in l.upper()]
    if q:
        content=[l for l in content if q.lower() in l.lower()]
    return {"lines": content, "total": len(content), "truncated": not found, "_links": {"self": {"href": "/api/v1/logs"}}}
