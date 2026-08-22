from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class BridgeCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    bridge_name: Optional[str]=None
    name: Optional[str]=None
    bridge_destination: Optional[str]=None
    destination: Optional[str]=None
    bridge_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    bridge_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "bridge_name" not in data: data["bridge_name"]=data.pop("name")
            if "destination" in data and "bridge_destination" not in data: data["bridge_destination"]=str(data.pop("destination"))
            if "enabled" in data and "bridge_enabled" not in data: data["bridge_enabled"]=data.pop("enabled")
            if "description" in data and "bridge_description" not in data: data["bridge_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class BridgeRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    bridge_uuid: UUID
    domain_id: UUID
    bridge_name: str
    bridge_destination: Optional[str]=None
    bridge_enabled: bool
    bridge_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class BridgeUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    bridge_name: Optional[str]=None
    bridge_destination: Optional[str]=None
    bridge_enabled: Optional[bool]=None
    bridge_description: Optional[str]=None
