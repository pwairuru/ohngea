from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class EmailQueueCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    email_from: Optional[str]=None
    email_to: Optional[str]=None
    email_subject: Optional[str]=None
    email_body: Optional[str]=None
    email_status: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict) and "domain_uuid" in data and "domain_id" not in data:
            data["domain_id"]=data.pop("domain_uuid")
        return data
class EmailQueueRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    email_queue_uuid: UUID
    domain_id: UUID
    email_from: str
    email_to: str
    email_subject: Optional[str]=None
    email_status: str
    created_at: datetime
class EmailQueueUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    email_status: Optional[str]=None
    email_subject: Optional[str]=None
