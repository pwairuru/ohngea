from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class StreamCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    stream_name: Optional[str]=None
    name: Optional[str]=None
    stream_location: Optional[str]=None
    location: Optional[str]=None
    stream_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    stream_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "stream_name" not in data: data["stream_name"]=data.pop("name")
            if "location" in data and "stream_location" not in data: data["stream_location"]=data.pop("location")
            if "enabled" in data and "stream_enabled" not in data: data["stream_enabled"]=data.pop("enabled")
            if "description" in data and "stream_description" not in data: data["stream_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class StreamRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    stream_uuid: UUID
    domain_id: UUID
    stream_name: str
    stream_location: Optional[str]=None
    stream_enabled: bool
    stream_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class StreamUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    stream_name: Optional[str]=None
    stream_location: Optional[str]=None
    stream_enabled: Optional[bool]=None
    stream_description: Optional[str]=None
