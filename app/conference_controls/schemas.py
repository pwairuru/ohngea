from datetime import datetime
from typing import Optional, List, Any
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class ConferenceControlCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    control_name: Optional[str]=None
    name: Optional[str]=None
    control_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    control_description: Optional[str]=None
    description: Optional[str]=None
    details: Optional[List[Any]]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "control_name" not in data: data["control_name"]=data.pop("name")
            if "enabled" in data and "control_enabled" not in data: data["control_enabled"]=data.pop("enabled")
            if "description" in data and "control_description" not in data: data["control_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class ConferenceControlRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    conference_control_uuid: UUID
    domain_id: UUID
    control_name: str
    control_enabled: bool
    control_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
    details: List[Any]=[]
class ConferenceControlUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    control_name: Optional[str]=None
    control_enabled: Optional[bool]=None
    control_description: Optional[str]=None
