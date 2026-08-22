from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlmodel import JSON, Column, Field, SQLModel

from app.db import uuid7


class WebRTCSettingsBase(SQLModel):
    enabled: bool = Field(default=False)
    auth_mode: str = Field(default="extension", max_length=32, description="extension|webrtc|both")
    wss_url: Optional[str] = Field(default=None, max_length=256)
    sip_domain: Optional[str] = Field(default=None, max_length=128)
    ice_servers: Optional[list] = Field(default=None, sa_column=Column(JSON))

    @field_validator("auth_mode")
    @classmethod
    def validate_auth_mode(cls, v: str) -> str:
        if v not in ("extension", "webrtc", "both"):
            raise ValueError("auth_mode must be extension|webrtc|both")
        return v

    @field_validator("wss_url")
    @classmethod
    def validate_wss_url(cls, v: Optional[str]) -> Optional[str]:
        if v is None:
            return v
        if not (v.startswith("wss://") or v.startswith("ws://")):
            raise ValueError("wss_url must start with wss:// or ws://")
        return v


class WebRTCSettings(WebRTCSettingsBase, table=True):
    __tablename__ = "webrtc_settings"
    domain_id: UUID = Field(foreign_key="domains.domain_id", primary_key=True, ondelete="CASCADE")
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
