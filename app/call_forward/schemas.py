from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, ConfigDict, model_validator


class CallForwardCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    domain_uuid: Optional[UUID] = None
    extension_uuid: Optional[UUID] = None
    extension_id: Optional[UUID] = None
    forward_all_destination: Optional[str] = None
    forward_all_enabled: Optional[bool] = False
    forward_busy_destination: Optional[str] = None
    forward_busy_enabled: Optional[bool] = False
    forward_no_answer_destination: Optional[str] = None
    forward_no_answer_enabled: Optional[bool] = False
    forward_user_not_registered_destination: Optional[str] = None
    forward_user_not_registered_enabled: Optional[bool] = False
    enabled: Optional[bool] = True
    description: Optional[str] = None
    # legacy aliases
    forward_destination: Optional[str] = None
    forward_enabled: Optional[bool] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
            if "extension_id" in data and "extension_uuid" not in data:
                data["extension_uuid"] = data.pop("extension_id")
            # map generic forward_destination/enabled to forward_all
            if "forward_destination" in data and "forward_all_destination" not in data:
                data["forward_all_destination"] = data.pop("forward_destination")
            if "forward_enabled" in data and "forward_all_enabled" not in data:
                data["forward_all_enabled"] = data.pop("forward_enabled")
        return data


class CallForwardRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    call_forward_uuid: UUID
    domain_id: UUID
    extension_uuid: Optional[UUID] = None
    forward_all_destination: Optional[str] = None
    forward_all_enabled: bool = False
    forward_busy_destination: Optional[str] = None
    forward_busy_enabled: bool = False
    forward_no_answer_destination: Optional[str] = None
    forward_no_answer_enabled: bool = False
    forward_user_not_registered_destination: Optional[str] = None
    forward_user_not_registered_enabled: bool = False
    enabled: bool = True
    description: Optional[str] = None
    created_at: datetime
    updated_at: datetime


class CallForwardUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    extension_uuid: Optional[UUID] = None
    forward_all_destination: Optional[str] = None
    forward_all_enabled: Optional[bool] = None
    forward_busy_destination: Optional[str] = None
    forward_busy_enabled: Optional[bool] = None
    forward_no_answer_destination: Optional[str] = None
    forward_no_answer_enabled: Optional[bool] = None
    forward_user_not_registered_destination: Optional[str] = None
    forward_user_not_registered_enabled: Optional[bool] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None
    forward_destination: Optional[str] = None
    forward_enabled: Optional[bool] = None
