from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DeviceBase(SQLModel):
    mac: str = Field(max_length=17, regex=r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$", description="MAC 00:11:22:33:44:55")
    vendor: str = Field(max_length=32, description="poly, yealink, etc.")
    model: Optional[str] = Field(default=None, max_length=64)
    label: Optional[str] = Field(default=None, max_length=128)
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    extension_uuid: Optional[UUID] = Field(default=None, foreign_key="extensions.extension_uuid")


class Device(DeviceBase, table=True):
    __tablename__ = "devices"
    device_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
