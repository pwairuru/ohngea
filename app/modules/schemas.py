from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class ModuleCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    module_label: Optional[str]=None
    label: Optional[str]=None
    module_name: Optional[str]=None
    name: Optional[str]=None
    module_category: Optional[str]=None
    module_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    module_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "label" in data and "module_label" not in data: data["module_label"]=data.pop("label")
            if "name" in data and "module_name" not in data: data["module_name"]=data.pop("name")
            if "enabled" in data and "module_enabled" not in data: data["module_enabled"]=data.pop("enabled")
            if "description" in data and "module_description" not in data: data["module_description"]=data.pop("description")
        return data
class ModuleRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    module_uuid: UUID
    module_label: str
    module_name: str
    module_category: Optional[str]=None
    module_enabled: bool
    module_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class ModuleUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    module_label: Optional[str]=None
    module_name: Optional[str]=None
    module_category: Optional[str]=None
    module_enabled: Optional[bool]=None
    module_description: Optional[str]=None
