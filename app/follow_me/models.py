from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class FollowMeBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    follow_me_enabled: bool = Field(default=True)
    cid_name_prefix: Optional[str] = Field(default=None, max_length=128)
    cid_number_prefix: Optional[str] = Field(default=None, max_length=32)
    call_prompt: Optional[str] = Field(default=None, max_length=32)
    dial_string: Optional[str] = Field(default=None, max_length=512)
    description: Optional[str] = Field(default=None, max_length=512)
    follow_me_ignore_busy: bool = Field(default=False)


class FollowMe(FollowMeBase, table=True):
    __tablename__ = "follow_me"
    follow_me_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class FollowMeDestinationBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    follow_me_uuid: UUID = Field(foreign_key="follow_me.follow_me_uuid", index=True, ondelete="CASCADE")
    follow_me_destination: str = Field(max_length=128, description="Destination number")
    follow_me_delay: int = Field(default=0)
    follow_me_timeout: int = Field(default=30)
    follow_me_prompt: Optional[str] = Field(default=None, max_length=32)
    follow_me_order: int = Field(default=0)


class FollowMeDestination(FollowMeDestinationBase, table=True):
    __tablename__ = "follow_me_destinations"
    follow_me_destination_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
