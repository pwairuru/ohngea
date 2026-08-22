from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class ExtensionSettingCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    extension_uuid: Optional[UUID]=None
    extension_setting_type: Optional[str]=None
    extension_setting_name: Optional[str]=None
    name: Optional[str]=None
    extension_setting_value: Optional[str]=None
    value: Optional[str]=None
    extension_setting_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    extension_setting_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "extension_setting_name" not in data: data["extension_setting_name"]=data.pop("name")
            if "value" in data and "extension_setting_value" not in data: data["extension_setting_value"]=str(data.pop("value"))
            if "enabled" in data and "extension_setting_enabled" not in data: data["extension_setting_enabled"]=data.pop("enabled")
            if "description" in data and "extension_setting_description" not in data: data["extension_setting_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class ExtensionSettingRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    extension_setting_uuid: UUID
    domain_id: UUID
    extension_uuid: UUID
    extension_setting_name: str
    extension_setting_value: str
    extension_setting_enabled: bool
    created_at: datetime
    updated_at: datetime
class ExtensionSettingUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    extension_setting_name: Optional[str]=None
    extension_setting_value: Optional[str]=None
    extension_setting_enabled: Optional[bool]=None
    extension_setting_description: Optional[str]=None
