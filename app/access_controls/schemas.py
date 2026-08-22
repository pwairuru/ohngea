from datetime import datetime
from typing import Optional, List, Any
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class AccessControlCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    access_control_name: Optional[str]=None
    name: Optional[str]=None
    access_control_default: Optional[str]=None
    default: Optional[str]=None
    access_control_description: Optional[str]=None
    description: Optional[str]=None
    nodes: Optional[List[Any]]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "access_control_name" not in data: data["access_control_name"]=data.pop("name")
            if "default" in data and "access_control_default" not in data: data["access_control_default"]=data.pop("default")
            if "description" in data and "access_control_description" not in data: data["access_control_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class AccessControlRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    access_control_uuid: UUID
    domain_id: UUID
    access_control_name: str
    access_control_default: str
    access_control_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
    nodes: List[Any]=[]
class AccessControlUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    access_control_name: Optional[str]=None
    access_control_default: Optional[str]=None
    access_control_description: Optional[str]=None
