from datetime import datetime
from typing import Optional, List, Union, Any
from uuid import UUID

from pydantic import BaseModel, field_validator

from app.ring_groups.models import ALLOWED_STRATEGIES, RingGroupBase


class RingGroupDestinationCreate(BaseModel):
    destination_number: str
    destination_delay: int = 0
    destination_timeout: int = 30
    destination_enabled: bool = True
    destination_description: Optional[str] = None


class RingGroupDestinationRead(BaseModel):
    ring_group_member_uuid: UUID
    ring_group_uuid: UUID
    domain_id: UUID
    destination_number: str
    destination_delay: int
    destination_timeout: int
    destination_enabled: bool
    destination_description: Optional[str] = None

    class Config:
        from_attributes = True


# Alias for compatibility
RingGroupMemberCreate = RingGroupDestinationCreate
RingGroupMemberRead = RingGroupDestinationRead


class RingGroupCreate(RingGroupBase):
    domain_id: Optional[UUID] = None  # type: ignore[assignment]
    members: Optional[List[Any]] = None
    destinations: Optional[List[RingGroupDestinationCreate]] = None

    @field_validator("members", mode="before")
    @classmethod
    def normalize_members(cls, v):
        # allow None, list of strings, list of dicts, or mixed
        if v is None:
            return v
        if not isinstance(v, list):
            raise ValueError("members must be a list")
        return v


class RingGroupRead(RingGroupBase):
    ring_group_uuid: UUID
    created_at: datetime
    updated_at: datetime
    members: List[RingGroupDestinationRead] = []
    destinations: List[RingGroupDestinationRead] = []

    class Config:
        from_attributes = True


class RingGroupUpdate(BaseModel):
    name: Optional[str] = None
    extension: Optional[str] = None
    strategy: Optional[str] = None
    timeout: Optional[int] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None
    members: Optional[List[Any]] = None
    destinations: Optional[List[RingGroupDestinationCreate]] = None

    @field_validator("strategy")
    @classmethod
    def validate_strategy(cls, v):
        if v is None:
            return v
        if v not in ALLOWED_STRATEGIES:
            raise ValueError(f"strategy must be one of {', '.join(sorted(ALLOWED_STRATEGIES))}")
        return v

    @field_validator("extension")
    @classmethod
    def validate_extension(cls, v):
        if v is None:
            return v
        import re

        if not re.match(r"^\d{2,7}$", v):
            raise ValueError("extension must be 2-7 digits")
        return v
