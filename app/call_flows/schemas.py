from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, ConfigDict, field_validator, model_validator
import re


class CallFlowCreate(BaseModel):
    model_config = ConfigDict(extra="allow")
    domain_id: Optional[UUID] = None
    call_flow_name: Optional[str] = None
    name: Optional[str] = None
    call_flow_extension: Optional[str] = None
    extension: Optional[str] = None
    call_flow_feature_code: Optional[str] = None
    feature_code: Optional[str] = None
    call_flow_context: Optional[str] = None
    context: Optional[str] = None
    call_flow_enabled: Optional[bool] = None
    enabled: Optional[bool] = None
    call_flow_description: Optional[str] = None
    description: Optional[str] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "name" in data and "call_flow_name" not in data:
                data["call_flow_name"] = data.pop("name")
            if "extension" in data and "call_flow_extension" not in data:
                data["call_flow_extension"] = data.pop("extension")
            if "feature_code" in data and "call_flow_feature_code" not in data:
                data["call_flow_feature_code"] = data.pop("feature_code")
            if "context" in data and "call_flow_context" not in data:
                data["call_flow_context"] = data.pop("context")
            if "enabled" in data and "call_flow_enabled" not in data:
                data["call_flow_enabled"] = data.pop("enabled")
            if "description" in data and "call_flow_description" not in data:
                data["call_flow_description"] = data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
        return data

    @field_validator("call_flow_extension")
    @classmethod
    def validate_ext(cls, v):
        if v is None:
            return v
        if not re.match(r"^[*#+]?\d{2,7}$", str(v)):
            raise ValueError("call_flow_extension must be 2-7 digits")
        return str(v)


class CallFlowRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    call_flow_uuid: UUID
    domain_id: UUID
    call_flow_name: str
    call_flow_extension: str
    call_flow_feature_code: Optional[str] = None
    call_flow_context: str
    call_flow_enabled: bool
    call_flow_description: Optional[str] = None
    created_at: datetime
    updated_at: datetime


class CallFlowUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    call_flow_name: Optional[str] = None
    call_flow_extension: Optional[str] = None
    call_flow_feature_code: Optional[str] = None
    call_flow_context: Optional[str] = None
    call_flow_enabled: Optional[bool] = None
    call_flow_description: Optional[str] = None

    @field_validator("call_flow_extension")
    @classmethod
    def validate_ext(cls, v):
        if v is None:
            return v
        if not re.match(r"^[*#+]?\d{2,7}$", str(v)):
            raise ValueError("call_flow_extension must be 2-7 digits")
        return str(v)
