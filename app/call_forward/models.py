from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class CallForwardBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    extension_uuid: Optional[UUID] = Field(default=None, foreign_key="extensions.extension_uuid", index=True, ondelete="CASCADE")
    forward_all_destination: Optional[str] = Field(default=None, max_length=128, description="Unconditional forward destination")
    forward_all_enabled: bool = Field(default=False)
    forward_busy_destination: Optional[str] = Field(default=None, max_length=128)
    forward_busy_enabled: bool = Field(default=False)
    forward_no_answer_destination: Optional[str] = Field(default=None, max_length=128)
    forward_no_answer_enabled: bool = Field(default=False)
    forward_user_not_registered_destination: Optional[str] = Field(default=None, max_length=128)
    forward_user_not_registered_enabled: bool = Field(default=False)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)


class CallForward(CallForwardBase, table=True):
    __tablename__ = "call_forwards"
    call_forward_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
