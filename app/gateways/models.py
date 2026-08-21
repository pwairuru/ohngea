from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class GatewayBase(SQLModel):
    gateway: str = Field(max_length=64, description="gateway name")
    username: Optional[str] = Field(default=None, max_length=64)
    password: Optional[str] = Field(default=None, max_length=64)
    proxy: Optional[str] = Field(default=None, max_length=128)
    register: bool = Field(default=True)
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")


class Gateway(GatewayBase, table=True):
    __tablename__ = "gateways"
    gateway_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
