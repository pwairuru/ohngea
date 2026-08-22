from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, ConfigDict, model_validator


class DestinationCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    destination_type: Optional[str] = "inbound"
    destination_number: Optional[str] = None
    number: Optional[str] = None
    destination_enabled: Optional[bool] = None
    enabled: Optional[bool] = None
    destination_description: Optional[str] = None
    description: Optional[str] = None
    destination_context: Optional[str] = None
    destination_app: Optional[str] = None
    destination_data: Optional[str] = None
    destination_caller_id_name: Optional[str] = None
    destination_caller_id_number: Optional[str] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "number" in data and "destination_number" not in data:
                data["destination_number"] = str(data.pop("number"))
            if "enabled" in data and "destination_enabled" not in data:
                data["destination_enabled"] = data.pop("enabled")
            if "description" in data and "destination_description" not in data:
                data["destination_description"] = data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
            if "did" in data and "destination_number" not in data:
                data["destination_number"] = str(data.pop("did"))
        return data


class DestinationRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    destination_uuid: UUID
    domain_id: UUID
    destination_type: str
    destination_number: str
    destination_enabled: bool
    destination_description: Optional[str] = None
    destination_context: Optional[str] = None
    destination_app: Optional[str] = None
    destination_data: Optional[str] = None
    created_at: datetime
    updated_at: datetime


class DestinationUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    destination_type: Optional[str] = None
    destination_number: Optional[str] = None
    destination_enabled: Optional[bool] = None
    destination_description: Optional[str] = None
    destination_context: Optional[str] = None
    destination_app: Optional[str] = None
    destination_data: Optional[str] = None
