from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class VarCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    var_category: Optional[str]="Provision"
    var_name: Optional[str]=None
    name: Optional[str]=None
    var_value: Optional[str]=None
    value: Optional[str]=None
    var_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    var_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "var_name" not in data: data["var_name"]=data.pop("name")
            if "value" in data and "var_value" not in data: data["var_value"]=str(data.pop("value"))
            if "enabled" in data and "var_enabled" not in data: data["var_enabled"]=data.pop("enabled")
            if "description" in data and "var_description" not in data: data["var_description"]=data.pop("description")
            if "category" in data and "var_category" not in data: data["var_category"]=data.pop("category")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class VarRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    var_uuid: UUID
    domain_id: UUID
    var_category: str
    var_name: str
    var_value: str
    var_enabled: bool
    var_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class VarUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    var_category: Optional[str]=None
    var_name: Optional[str]=None
    var_value: Optional[str]=None
    var_enabled: Optional[bool]=None
    var_description: Optional[str]=None
