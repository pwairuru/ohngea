from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator

class FaxQueueCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    fax_uuid: Optional[UUID]=None
    origination_uuid: Optional[UUID]=None
    fax_log_uuid: Optional[UUID]=None
    fax_date: Optional[datetime]=None
    hostname: Optional[str]=None
    fax_caller_id_name: Optional[str]=None
    fax_caller_id_number: Optional[str]=None
    fax_recipient: Optional[str]=None
    fax_number: Optional[str]=None
    fax_prefix: Optional[str]=None
    fax_email_address: Optional[str]=None
    fax_file: Optional[str]=None
    fax_status: Optional[str]=None
    fax_queue_name: Optional[str]=None
    fax_retry_date: Optional[datetime]=None
    fax_notify_sent: Optional[bool]=None
    fax_notify_date: Optional[datetime]=None
    fax_accountcode: Optional[str]=None
    fax_command: Optional[str]=None
    fax_response: Optional[str]=None
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
    origination_uuid: Optional[UUID]=None
    fax_log_uuid: Optional[UUID]=None
    fax_date: Optional[datetime]=None
    hostname: Optional[str]=None
    fax_caller_id_name: Optional[str]=None
    fax_caller_id_number: Optional[str]=None
    fax_recipient: Optional[str]=None
    fax_number: Optional[str]=None
    fax_prefix: Optional[str]=None
    fax_email_address: Optional[str]=None
    fax_file: Optional[str]=None
    fax_status: str
    fax_retry_date: Optional[datetime]=None
    fax_notify_sent: Optional[bool]=None
    fax_notify_date: Optional[datetime]=None
    fax_accountcode: Optional[str]=None
    fax_command: Optional[str]=None
    fax_response: Optional[str]=None
    fax_retry_count: int=0
    created_at: datetime
    updated_at: datetime

class FaxQueueUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    fax_status: Optional[str]=None
    fax_number: Optional[str]=None
    fax_file: Optional[str]=None
    fax_recipient: Optional[str]=None
    fax_caller_id_name: Optional[str]=None
    fax_caller_id_number: Optional[str]=None
    fax_prefix: Optional[str]=None
    fax_email_address: Optional[str]=None
    fax_retry_date: Optional[datetime]=None
    fax_notify_sent: Optional[bool]=None
    fax_accountcode: Optional[str]=None
    fax_command: Optional[str]=None
    fax_response: Optional[str]=None
