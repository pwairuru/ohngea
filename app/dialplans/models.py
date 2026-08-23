from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DialplanBase(SQLModel):
    name: str = Field(max_length=128, index=True)
    context: str = Field(max_length=128, description="e.g. tenant.example.com")
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    description: Optional[str] = Field(default=None)
    # full parity
    dialplan_continue: bool = Field(default=False, description="continue flag")
    dialplan_order: Optional[int] = Field(default=100)
    hostname: Optional[str] = Field(default=None, max_length=128)
    app_uuid: Optional[str] = Field(default=None, max_length=64)


class Dialplan(DialplanBase, table=True):
    __tablename__ = "dialplans"
    dialplan_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class DialplanDetail(SQLModel, table=True):
    __tablename__ = "dialplan_details"
    detail_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    dialplan_uuid: UUID = Field(foreign_key="dialplans.dialplan_uuid", ondelete="CASCADE", index=True)
    tag: str = Field(max_length=32)  # condition, action, anti-action
    type: str = Field(max_length=64)  # destination_number, etc.
    data: str = Field(max_length=512)
    order: int = Field(default=0)
