from datetime import datetime
from typing import Optional, List, Any
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class PhraseCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    phrase_name: Optional[str] = None
    name: Optional[str] = None
    phrase_language: Optional[str] = "en-us"
    phrase_enabled: Optional[bool] = None
    enabled: Optional[bool] = None
    phrase_description: Optional[str] = None
    description: Optional[str] = None
    details: Optional[List[Any]] = None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "name" in data and "phrase_name" not in data: data["phrase_name"]=data.pop("name")
            if "enabled" in data and "phrase_enabled" not in data: data["phrase_enabled"]=data.pop("enabled")
            if "description" in data and "phrase_description" not in data: data["phrase_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class PhraseRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    phrase_uuid: UUID
    domain_id: UUID
    phrase_name: str
    phrase_language: Optional[str]=None
    phrase_enabled: bool
    phrase_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
    details: List[Any]=[]
class PhraseUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    phrase_name: Optional[str]=None
    phrase_language: Optional[str]=None
    phrase_enabled: Optional[bool]=None
    phrase_description: Optional[str]=None
    details: Optional[List[Any]]=None
