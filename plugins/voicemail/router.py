"""Deprecated mock — voicemails now core DB-backed at app/voicemails.
This plugin router is no longer mounted (see app/main.py skip for voicemail).
Kept for backwards compatibility if imported directly; it now proxies to core
or just exposes health. The real API is at app.voicemails.router (/api/v1/voicemails).
"""
from fastapi import APIRouter

router = APIRouter(prefix="/api/v1/voicemail", tags=["voicemail"])


@router.get("/health")
async def health():
    return {"plugin": "voicemail", "status": "ok", "mode": "core-delegated"}
