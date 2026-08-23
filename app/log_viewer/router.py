from typing import Optional
from pathlib import Path
from fastapi import APIRouter, Depends, Query
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.freeswitch.esl_client import esl_client, parse_sofia_status

router = APIRouter(prefix="/api/v1/logs", tags=["logs"])
ALLOWED = ["/var/log/freeswitch/freeswitch.log", "/var/log/freeswitch.log", "/tmp/freeswitch.log", "freeswitch.log", "/app/freeswitch_conf/../freeswitch.log"]


@router.get("", response_model=dict)
async def view_logs(lines: int = Query(100, ge=1, le=1000), level: Optional[str] = Query(None), q: Optional[str] = Query(None), user: User = Depends(require_permission("log_view"))):
    # Try real FS via ESL first, then file, then mock
    content = []
    found = False
    # Try ESL: sofia status + show registrations + status
    try:
        # Get FS status and sofia
        status_raw = await esl_client.api("status")
        sofia_raw = await esl_client.api("sofia status")
        # Try to get registrations via sofia
        # FS has no direct show registrations, but sofia status shows count
        # Also try to get channels
        channels_raw = await esl_client.api("show channels")
        # Build log lines from ESL
        if status_raw and "UP" in status_raw:
            content.append(f"[FS status] {status_raw.strip().splitlines()[0][:200]}")
            found = True
        if sofia_raw:
            for line in sofia_raw.splitlines()[:20]:
                if line.strip():
                    content.append(f"[sofia] {line.strip()}")
                    found = True
        if channels_raw and "total" not in channels_raw.lower():
            # show channels returns CSV or text
            for line in channels_raw.splitlines()[:20]:
                if line.strip():
                    content.append(f"[channels] {line.strip()}")
    except Exception:
        pass

    # Try reading FS log file (if mounted)
    if not found:
        for p in ALLOWED:
            pp = Path(p)
            if pp.exists():
                try:
                    txt = pp.read_text()[-20000:]
                    content = txt.splitlines()[-lines:]
                    found = True
                    break
                except:
                    pass
    # Try reading via ESL system command if file not directly accessible
    if not found:
        try:
            # Try to cat log via FS system api (if log file exists inside FS container)
            for log_path in ["/var/log/freeswitch/freeswitch.log", "/usr/local/freeswitch/log/freeswitch.log"]:
                raw = await esl_client.api(f"system cat {log_path} | tail -n {lines}")
                if raw and "No such file" not in raw and "error" not in raw.lower():
                    # raw contains Content-Length header, extract body
                    if "\n\n" in raw:
                        body = raw.split("\n\n", 1)[1]
                        content = body.splitlines()[-lines:]
                        found = True
                        break
        except Exception:
            pass

    if not found:
        # Fallback: generate live mock based on current FS state
        try:
            # Get real registrations via ESL
            sofia_map = await esl_client.sofia_status()
            if sofia_map:
                for name, state in sofia_map.items():
                    content.append(f"[sofia] {name} {state}")
                found = True
            # Get FS status
            probe = await esl_client.check_status()
            if probe.get("connected"):
                content.append(f"[FS] connected {probe.get('host')}:{probe.get('port')} latency {probe.get('latency_ms')}ms")
                found = True
            else:
                content.append(f"[FS] disconnected {probe.get('error')}")
        except Exception:
            pass

    if not found or not content:
        # Final mock for dev without FS
        mock = [
            f"[NOTICE] sofia_reg.c:1234 SIP auth success for 1010@192.168.1.108 (via xml_curl)",
            f"[INFO] switch_core.c: dialplan found for 192.168.1.108 context 192.168.1.108",
            f"[INFO] sofia.c: registration 1010@192.168.1.108 expires 3600",
        ]
        while len(content) < lines and len(content) < 100:
            content.extend(mock)
        content = content[:lines]

    # Apply filters
    if level:
        content = [l for l in content if level.upper() in l.upper()]
    if q:
        content = [l for l in content if q.lower() in l.lower()]

    # Add hint about where logs come from
    return {"lines": content[:lines], "total": len(content), "truncated": not found, "source": "esl" if found else "mock", "_links": {"self": {"href": "/api/v1/logs"}}}


@router.get("/freeswitch", response_model=dict)
async def freeswitch_logs(lines: int = Query(50, ge=1, le=500), user: User = Depends(require_permission("log_view"))):
    """Dedicated FS logs endpoint: sofia status + registrations + recent SIP trace."""
    result = {}
    try:
        # sofia status
        sofia_raw = await esl_client.api("sofia status")
        result["sofia_status"] = sofia_raw[:4000] if sofia_raw else ""
        result["sofia_parsed"] = parse_sofia_status(sofia_raw) if sofia_raw else {}
    except Exception as e:
        result["sofia_status"] = f"error: {e}"
        result["sofia_parsed"] = {}
    try:
        probe = await esl_client.check_status()
        result["fs_status"] = probe
    except Exception as e:
        result["fs_status"] = {"connected": False, "error": str(e)}
    try:
        # Try to get registrations via sofia XML
        # FS doesn't have direct show registrations, but we can try system
        regs = await esl_client.api("show registrations")
        result["registrations_raw"] = regs[:4000] if regs else ""
    except Exception as e:
        result["registrations_raw"] = f"error: {e}"
    try:
        # Recent channels
        chans = await esl_client.api("show channels")
        result["channels"] = chans[:4000] if chans else ""
    except Exception as e:
        result["channels"] = f"error: {e}"
    return result
