from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class SofiaGlobalSettingCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    global_setting_name: Optional[str]=None
    name: Optional[str]=None
    global_setting_value: Optional[str]=None
    value: Optional[str]=None
    global_setting_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    global_setting_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "global_setting_name" not in data: data["global_setting_name"]=data.pop("name")
            if "value" in data and "global_setting_value" not in data: data["global_setting_value"]=str(data.pop("value"))
            if "enabled" in data and "global_setting_enabled" not in data: data["global_setting_enabled"]=data.pop("enabled")
            if "description" in data and "global_setting_description" not in data: data["global_setting_description"]=data.pop("description")
        return data
class SofiaGlobalSettingRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    sofia_global_setting_uuid: UUID
    global_setting_name: str
    global_setting_value: str
    global_setting_enabled: bool
    global_setting_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class SofiaGlobalSettingUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    global_setting_name: Optional[str]=None
    global_setting_value: Optional[str]=None
    global_setting_enabled: Optional[bool]=None
    global_setting_description: Optional[str]=None
