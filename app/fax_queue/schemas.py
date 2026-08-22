from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class FaxQueueCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    fax_uuid: Optional[UUID]=None
    fax_number: Optional[str]=None
    fax_file: Optional[str]=None
    fax_status: Optional[str]=None
    fax_queue_name: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict) and "domain_uuid" in data and "domain_id" not in data:
            data["domain_id"]=data.pop("domain_uuid")
        return data
class FaxQueueRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    fax_queue_uuid: UUID
    domain_id: UUID
    fax_uuid: Optional[UUID]=None
    fax_number: Optional[str]=None
    fax_file: Optional[str]=None
    fax_status: str
    created_at: datetime
    updated_at: datetime
class FaxQueueUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    fax_status: Optional[str]=None
    fax_number: Optional[str]=None
    fax_file: Optional[str]=None
