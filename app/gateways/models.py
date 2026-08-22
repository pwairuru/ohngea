from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

import sqlalchemy as sa
from sqlmodel import Field, SQLModel, Column

from app.db import uuid7


class GatewayBase(SQLModel):
    gateway: str = Field(max_length=64, description="gateway name")
    username: Optional[str] = Field(default=None, max_length=64)
    password: Optional[str] = Field(default=None, max_length=64)
    proxy: Optional[str] = Field(default=None, max_length=128)
    # use sa_column to avoid shadowing SQLModel.register classmethod; DB column remains "register"
    gateway_register: bool = Field(default=True, sa_column=Column("register", sa.Boolean(), nullable=False))
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    # full parity — hidden advanced
    distinct_to: Optional[bool] = Field(default=None)
    auth_username: Optional[str] = Field(default=None, max_length=64)
    realm: Optional[str] = Field(default=None, max_length=128)
    from_user: Optional[str] = Field(default=None, max_length=64)
    from_domain: Optional[str] = Field(default=None, max_length=128)
    register_proxy: Optional[str] = Field(default=None, max_length=128)
    outbound_proxy: Optional[str] = Field(default=None, max_length=128)
    expire_seconds: Optional[int] = Field(default=800)
    register_transport: Optional[str] = Field(default=None, max_length=32, description="udp,tcp,tls")
    contact_params: Optional[str] = Field(default=None, max_length=256)
    retry_seconds: Optional[int] = Field(default=30)
    extension: Optional[str] = Field(default=None, max_length=64)
    ping: Optional[str] = Field(default=None, max_length=32)
    ping_min: Optional[int] = Field(default=None)
    ping_max: Optional[int] = Field(default=None)
    contact_in_ping: Optional[bool] = Field(default=None)
    caller_id_in_from: Optional[bool] = Field(default=None)
    supress_cng: Optional[bool] = Field(default=None)
    sip_cid_type: Optional[str] = Field(default=None, max_length=32)
    codec_prefs: Optional[str] = Field(default=None, max_length=256)
    channels: Optional[int] = Field(default=None)
    extension_in_contact: Optional[str] = Field(default=None, max_length=64)
    context: Optional[str] = Field(default="public", max_length=64)
    profile: Optional[str] = Field(default="external", max_length=64)
    hostname: Optional[str] = Field(default=None, max_length=255)
    description: Optional[str] = Field(default=None, max_length=512)

    # compatibility: expose "register" as property for API payloads
    @property
    def register(self) -> bool:
        return self.gateway_register

    @register.setter
    def register(self, value: bool):
        self.gateway_register = bool(value) if value is not None else True


class Gateway(GatewayBase, table=True):
    __tablename__ = "gateways"
    gateway_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
