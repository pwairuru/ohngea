from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class FeatureCodeCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    feature_code_name: Optional[str]=None
    name: Optional[str]=None
    feature_code_value: Optional[str]=None
    value: Optional[str]=None
    feature_code_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    feature_code_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "feature_code_name" not in data: data["feature_code_name"]=data.pop("name")
            if "value" in data and "feature_code_value" not in data: data["feature_code_value"]=str(data.pop("value"))
            if "enabled" in data and "feature_code_enabled" not in data: data["feature_code_enabled"]=data.pop("enabled")
            if "description" in data and "feature_code_description" not in data: data["feature_code_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class FeatureCodeRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    feature_code_uuid: UUID
    domain_id: UUID
    feature_code_name: str
    feature_code_value: str
    feature_code_enabled: bool
    feature_code_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class FeatureCodeUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    feature_code_name: Optional[str]=None
    feature_code_value: Optional[str]=None
    feature_code_enabled: Optional[bool]=None
    feature_code_description: Optional[str]=None
