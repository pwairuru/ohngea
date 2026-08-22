from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, ConfigDict, model_validator


class CallBlockCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    call_block_name: Optional[str] = None
    name: Optional[str] = None
    call_block_number: Optional[str] = None
    number: Optional[str] = None
    call_block_enabled: Optional[bool] = None
    enabled: Optional[bool] = None
    call_block_description: Optional[str] = None
    description: Optional[str] = None
    call_block_action: Optional[str] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "name" in data and "call_block_name" not in data:
                data["call_block_name"] = data.pop("name")
            if "number" in data and "call_block_number" not in data:
                data["call_block_number"] = data.pop("number")
            if "enabled" in data and "call_block_enabled" not in data:
                data["call_block_enabled"] = data.pop("enabled")
            if "description" in data and "call_block_description" not in data:
                data["call_block_description"] = data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
        return data


class CallBlockRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    call_block_uuid: UUID
    domain_id: UUID
    call_block_name: str
    call_block_number: str
    call_block_enabled: bool
    call_block_description: Optional[str] = None
    call_block_action: Optional[str] = None
    created_at: datetime
    updated_at: datetime


class CallBlockUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    call_block_name: Optional[str] = None
    call_block_number: Optional[str] = None
    call_block_enabled: Optional[bool] = None
    call_block_description: Optional[str] = None
    call_block_action: Optional[str] = None
