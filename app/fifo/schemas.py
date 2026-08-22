from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class FifoCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    fifo_name: Optional[str]=None
    name: Optional[str]=None
    fifo_extension: Optional[str]=None
    extension: Optional[str]=None
    fifo_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    fifo_description: Optional[str]=None
    description: Optional[str]=None
    fifo_strategy: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "fifo_name" not in data: data["fifo_name"]=data.pop("name")
            if "extension" in data and "fifo_extension" not in data: data["fifo_extension"]=str(data.pop("extension"))
            if "enabled" in data and "fifo_enabled" not in data: data["fifo_enabled"]=data.pop("enabled")
            if "description" in data and "fifo_description" not in data: data["fifo_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class FifoRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    fifo_uuid: UUID
    domain_id: UUID
    fifo_name: str
    fifo_extension: Optional[str]=None
    fifo_enabled: bool
    fifo_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class FifoUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    fifo_name: Optional[str]=None
    fifo_extension: Optional[str]=None
    fifo_enabled: Optional[bool]=None
    fifo_description: Optional[str]=None
    fifo_strategy: Optional[str]=None
