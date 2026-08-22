from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class CallBroadcastCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    broadcast_name: Optional[str]=None
    name: Optional[str]=None
    broadcast_description: Optional[str]=None
    description: Optional[str]=None
    broadcast_timeout: Optional[int]=None
    broadcast_concurrent_limit: Optional[int]=None
    broadcast_caller_id_name: Optional[str]=None
    broadcast_caller_id_number: Optional[str]=None
    broadcast_destination_type: Optional[str]=None
    broadcast_phone_numbers: Optional[str]=None
    broadcast_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "broadcast_name" not in data: data["broadcast_name"]=data.pop("name")
            if "description" in data and "broadcast_description" not in data: data["broadcast_description"]=data.pop("description")
            if "enabled" in data and "broadcast_enabled" not in data: data["broadcast_enabled"]=data.pop("enabled")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class CallBroadcastRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    call_broadcast_uuid: UUID
    domain_id: UUID
    broadcast_name: str
    broadcast_description: Optional[str]=None
    broadcast_timeout: Optional[int]=None
    broadcast_concurrent_limit: Optional[int]=None
    broadcast_caller_id_name: Optional[str]=None
    broadcast_caller_id_number: Optional[str]=None
    broadcast_destination_type: Optional[str]=None
    broadcast_phone_numbers: Optional[str]=None
    broadcast_enabled: bool
    created_at: datetime
    updated_at: datetime
class CallBroadcastUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    broadcast_name: Optional[str]=None
    broadcast_description: Optional[str]=None
    broadcast_timeout: Optional[int]=None
    broadcast_concurrent_limit: Optional[int]=None
    broadcast_caller_id_name: Optional[str]=None
    broadcast_caller_id_number: Optional[str]=None
    broadcast_destination_type: Optional[str]=None
    broadcast_phone_numbers: Optional[str]=None
    broadcast_enabled: Optional[bool]=None
