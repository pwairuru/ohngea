"""Deprecated mock — time_conditions now core DB-backed at app/time_conditions.
This plugin router is no longer mounted (see app/main.py skip for time_conditions).
Kept for backwards compatibility if imported directly; it now proxies to core
or just exposes health. The real API is at app.time_conditions.router.
"""
from fastapi import APIRouter

router = APIRouter(prefix="/api/v1/time_conditions", tags=["time_conditions"])


@router.get("/health")
async def health():
    return {"plugin": "time_conditions", "status": "ok", "mode": "core-delegated"}
