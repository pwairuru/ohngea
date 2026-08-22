from datetime import datetime
from typing import Optional, Any, List
from uuid import UUID

from pydantic import BaseModel, ConfigDict, model_validator


class FollowMeDestinationsCreate(BaseModel):
    follow_me_destination: str
    follow_me_delay: int = 0
    follow_me_timeout: int = 30
    follow_me_order: int = 0


class FollowMeCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    follow_me_enabled: Optional[bool] = True
    enabled: Optional[bool] = None
    cid_name_prefix: Optional[str] = None
    cid_number_prefix: Optional[str] = None
    call_prompt: Optional[str] = None
    dial_string: Optional[str] = None
    description: Optional[str] = None
    destinations: Optional[List[Any]] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "enabled" in data and "follow_me_enabled" not in data:
                data["follow_me_enabled"] = data.pop("enabled")
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
        return data


class FollowMeRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    follow_me_uuid: UUID
    domain_id: UUID
    follow_me_enabled: bool
    cid_name_prefix: Optional[str] = None
    cid_number_prefix: Optional[str] = None
    call_prompt: Optional[str] = None
    dial_string: Optional[str] = None
    description: Optional[str] = None
    created_at: datetime
    updated_at: datetime
    destinations: List[Any] = []


class FollowMeUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    follow_me_enabled: Optional[bool] = None
    cid_name_prefix: Optional[str] = None
    cid_number_prefix: Optional[str] = None
    call_prompt: Optional[str] = None
    dial_string: Optional[str] = None
    description: Optional[str] = None
    destinations: Optional[List[Any]] = None
