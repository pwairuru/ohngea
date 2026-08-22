from datetime import datetime
from typing import Optional, List, Any
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class ConferenceProfileCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    profile_name: Optional[str]=None
    name: Optional[str]=None
    profile_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    profile_description: Optional[str]=None
    description: Optional[str]=None
    params: Optional[List[Any]]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "profile_name" not in data: data["profile_name"]=data.pop("name")
            if "enabled" in data and "profile_enabled" not in data: data["profile_enabled"]=data.pop("enabled")
            if "description" in data and "profile_description" not in data: data["profile_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class ConferenceProfileRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    conference_profile_uuid: UUID
    domain_id: UUID
    profile_name: str
    profile_enabled: bool
    profile_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
    params: List[Any]=[]
class ConferenceProfileUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    profile_name: Optional[str]=None
    profile_enabled: Optional[bool]=None
    profile_description: Optional[str]=None
