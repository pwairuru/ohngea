import re
from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlmodel import Field, SQLModel

from app.db import uuid7


ALLOWED_STRATEGIES = {"simultaneous", "enterprise", "sequence", "rollover", "random", "hunt"}


class RingGroupBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    name: str = Field(max_length=128, description="Ring group name")
    extension: str = Field(max_length=32, description="Ring group extension 2-7 digits")
    strategy: str = Field(default="simultaneous", max_length=32, description="simultaneous, enterprise, sequence, rollover, random, hunt")
    timeout: int = Field(default=30, ge=1, le=999, description="Call timeout seconds")
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)

    @field_validator("extension")
    @classmethod
    def validate_extension(cls, v: str) -> str:
        if not re.match(r"^\d{2,7}$", v):
            raise ValueError("extension must be 2-7 digits")
        return v

    @field_validator("strategy")
    @classmethod
    def validate_strategy(cls, v: str) -> str:
        # normalize hunt -> sequence for compatibility, but allow hunt as alias
        if v not in ALLOWED_STRATEGIES:
            raise ValueError(f"strategy must be one of {', '.join(sorted(ALLOWED_STRATEGIES))}")
        return v


class RingGroup(RingGroupBase, table=True):
    __tablename__ = "ring_groups"
    ring_group_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class RingGroupMemberBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    ring_group_uuid: UUID = Field(foreign_key="ring_groups.ring_group_uuid", index=True, ondelete="CASCADE")
    destination_number: str = Field(max_length=32, description="Member destination extension or number")
    destination_delay: int = Field(default=0, ge=0, le=999)
    destination_timeout: int = Field(default=30, ge=1, le=999)
    destination_enabled: bool = Field(default=True)
    destination_description: Optional[str] = Field(default=None, max_length=255)


class RingGroupMember(RingGroupMemberBase, table=True):
    __tablename__ = "ring_group_members"
    ring_group_member_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())


# Compatibility alias — legacy Ohngea uses v_ring_group_destinations.
# We use ring_group_members as primary; a VIEW ring_group_destinations
# is created in migration for drop-in queries without needing a second model.
