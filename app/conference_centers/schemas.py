from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class ConferenceCenterCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    conference_center_name: Optional[str]=None
    name: Optional[str]=None
    conference_center_extension: Optional[str]=None
    extension: Optional[str]=None
    conference_center_pin_length: Optional[int]=None
    conference_center_greeting: Optional[str]=None
    conference_center_description: Optional[str]=None
    description: Optional[str]=None
    conference_center_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "conference_center_name" not in data: data["conference_center_name"]=data.pop("name")
            if "extension" in data and "conference_center_extension" not in data: data["conference_center_extension"]=str(data.pop("extension"))
            if "enabled" in data and "conference_center_enabled" not in data: data["conference_center_enabled"]=data.pop("enabled")
            if "description" in data and "conference_center_description" not in data: data["conference_center_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class ConferenceCenterRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    conference_center_uuid: UUID
    domain_id: UUID
    conference_center_name: str
    conference_center_extension: Optional[str]=None
    conference_center_pin_length: Optional[int]=None
    conference_center_greeting: Optional[str]=None
    conference_center_description: Optional[str]=None
    conference_center_enabled: bool
    created_at: datetime
    updated_at: datetime
class ConferenceCenterUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    conference_center_name: Optional[str]=None
    conference_center_extension: Optional[str]=None
    conference_center_pin_length: Optional[int]=None
    conference_center_greeting: Optional[str]=None
    conference_center_description: Optional[str]=None
    conference_center_enabled: Optional[bool]=None
