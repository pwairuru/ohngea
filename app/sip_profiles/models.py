from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class SipProfileBase(SQLModel):
    name: str = Field(max_length=64)
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    sip_port: int = Field(default=5060)
    sip_ip: Optional[str] = Field(default=None)
    wss_port: Optional[int] = Field(default=None)
    wss_binding: Optional[str] = Field(default=None, max_length=128)
    tls_cert_path: Optional[str] = Field(default=None, max_length=256)
    tls_key_path: Optional[str] = Field(default=None, max_length=256)


class SipProfile(SipProfileBase, table=True):
    __tablename__ = "sip_profiles"
    sip_profile_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
