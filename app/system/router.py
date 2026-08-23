from fastapi import APIRouter, Depends
from app.auth.deps import get_current_user
from app.auth.models import User
import asyncio
import platform
from app.system.service import get_instance

router = APIRouter(prefix="/api/v1/system", tags=["system"])

# Simple in-memory cache for metrics (5s) to avoid hammering /proc/stat
_cache: dict[str, tuple[float, dict]] = {}
CACHE_TTL = 5.0

async def _get_metrics_cached() -> dict:
    import time
    now = time.monotonic()
    if "metrics" in _cache:
        ts, data = _cache["metrics"]
        if now - ts < CACHE_TTL:
            return data
    inst = get_instance()
    # Run blocking service in thread pool
    data = await asyncio.to_thread(inst.metrics)
    # Also add load and hostname already in metrics
    _cache["metrics"] = (now, data)
    return data

async def _get_info_cached() -> dict:
    import time
    now = time.monotonic()
    if "info" in _cache:
        ts, data = _cache["info"]
        if now - ts < CACHE_TTL:
            return data
    inst = get_instance()
    data = await asyncio.to_thread(inst.as_dict)
    _cache["info"] = (now, data)
    return data

@router.get("/status", response_model=dict)
async def system_status(user: User = Depends(get_current_user)):
    """Legacy compat + extended. Returns hostname, load_avg plus full metrics if available."""
    from app.freeswitch.status_router import _get_fs_status_cached
    fs = await _get_fs_status_cached()
    inst = get_instance()
    # Quick path: get load and hostname without heavy cpu calc?
    # But we want to provide metrics as well.
    try:
        metrics = await _get_metrics_cached()
        system = {
            "hostname": metrics.get("hostname") or platform.node(),
            "load_avg": metrics.get("load_avg"),
            "cpu": {"total_percent": metrics.get("cpu_total"), "per_core": metrics.get("cpu_per_core"), "cores": metrics.get("cpu_cores")},
            "memory": metrics.get("memory"),
            "disk": metrics.get("disks"),
            "disk_percent": metrics.get("disk_percent"),
            "uptime": metrics.get("uptime"),
        }
    except Exception:
        import os
        try:
            load = os.getloadavg()[0] if hasattr(os, "getloadavg") else None
        except Exception:
            load = None
        system = {"hostname": platform.node(), "load_avg": load}
    return {"freeswitch": fs, "system": system, "_links": {"self": {"href": "/api/v1/system/status"}, "info": {"href": "/api/v1/system/info"}, "metrics": {"href": "/api/v1/system/metrics"}}}

@router.get("/info", response_model=dict)
async def system_info(user: User = Depends(get_current_user)):
    """Full system information (OS, CPU, memory, disk, uptime, load)."""
    data = await _get_info_cached()
    data["_links"] = {"self": {"href": "/api/v1/system/info"}, "metrics": {"href": "/api/v1/system/metrics"}, "status": {"href": "/api/v1/system/status"}}
    return data

@router.get("/metrics", response_model=dict)
async def system_metrics(user: User = Depends(get_current_user)):
    """Lightweight live metrics for polling/dashboard (cpu, mem, disk, load)."""
    data = await _get_metrics_cached()
    data["_links"] = {"self": {"href": "/api/v1/system/metrics"}, "info": {"href": "/api/v1/system/info"}}
    return data
