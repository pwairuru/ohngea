"""Deprecated mock — ring_groups now core DB-backed at app/ring_groups.
This plugin router is no longer mounted (see app/main.py skip for ring_groups).
Kept for backwards compatibility if imported directly; it now proxies to core
or just exposes health. The real API is at app.ring_groups.router.
"""
from fastapi import APIRouter

router = APIRouter(prefix="/api/v1/ring_groups", tags=["ring_groups"])


@router.get("/health")
async def health():
    return {"plugin": "ring_groups", "status": "ok", "mode": "core-delegated"}
