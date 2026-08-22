from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class NumberTranslationCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    number_translation_name: Optional[str]=None
    name: Optional[str]=None
    number_translation_regex: Optional[str]=None
    regex: Optional[str]=None
    number_translation_replace: Optional[str]=None
    replace: Optional[str]=None
    number_translation_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    number_translation_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "number_translation_name" not in data: data["number_translation_name"]=data.pop("name")
            if "regex" in data and "number_translation_regex" not in data: data["number_translation_regex"]=data.pop("regex")
            if "replace" in data and "number_translation_replace" not in data: data["number_translation_replace"]=data.pop("replace")
            if "enabled" in data and "number_translation_enabled" not in data: data["number_translation_enabled"]=data.pop("enabled")
            if "description" in data and "number_translation_description" not in data: data["number_translation_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class NumberTranslationRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    number_translation_uuid: UUID
    domain_id: UUID
    number_translation_name: str
    number_translation_regex: str
    number_translation_replace: str
    number_translation_enabled: bool
    number_translation_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class NumberTranslationUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    number_translation_name: Optional[str]=None
    number_translation_regex: Optional[str]=None
    number_translation_replace: Optional[str]=None
    number_translation_enabled: Optional[bool]=None
    number_translation_description: Optional[str]=None
