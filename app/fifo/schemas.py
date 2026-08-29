from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator

class FifoCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    dialplan_uuid: Optional[UUID]=None
    fifo_name: Optional[str]=None
    name: Optional[str]=None
    fifo_extension: Optional[str]=None
    extension: Optional[str]=None
    fifo_agent_queue: Optional[str]=None
    fifo_agent_status: Optional[str]=None
    fifo_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    fifo_description: Optional[str]=None
    description: Optional[str]=None
    fifo_strategy: Optional[str]=None
    fifo_members: Optional[str]=None
    fifo_timeout_seconds: Optional[str]=None
    fifo_exit_key: Optional[str]=None
    fifo_exit_action: Optional[str]=None
    fifo_music: Optional[str]=None
    music: Optional[str]=None
    fifo_order: Optional[int]=None
    order: Optional[int]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "fifo_name" not in data: data["fifo_name"]=data.pop("name")
            if "extension" in data and "fifo_extension" not in data: data["fifo_extension"]=str(data.pop("extension"))
            if "enabled" in data and "fifo_enabled" not in data: data["fifo_enabled"]=data.pop("enabled")
            if "description" in data and "fifo_description" not in data: data["fifo_description"]=data.pop("description")
            if "music" in data and "fifo_music" not in data: data["fifo_music"]=data.pop("music")
            if "order" in data and "fifo_order" not in data: data["fifo_order"]=data.pop("order")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data

class FifoRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    fifo_uuid: UUID
    domain_id: UUID
    dialplan_uuid: Optional[UUID]=None
    fifo_name: str
    fifo_extension: Optional[str]=None
    fifo_agent_queue: Optional[str]=None
    fifo_agent_status: Optional[str]=None
    fifo_enabled: bool
    fifo_description: Optional[str]=None
    fifo_strategy: Optional[str]=None
    fifo_members: Optional[str]=None
    fifo_timeout_seconds: Optional[str]=None
    fifo_exit_key: Optional[str]=None
    fifo_exit_action: Optional[str]=None
    fifo_music: Optional[str]=None
    fifo_order: Optional[int]=None
    created_at: datetime
    updated_at: datetime

class FifoUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    dialplan_uuid: Optional[UUID]=None
    fifo_name: Optional[str]=None
    fifo_extension: Optional[str]=None
    fifo_agent_queue: Optional[str]=None
    fifo_agent_status: Optional[str]=None
    fifo_enabled: Optional[bool]=None
    fifo_description: Optional[str]=None
    fifo_strategy: Optional[str]=None
    fifo_members: Optional[str]=None
    fifo_timeout_seconds: Optional[str]=None
    fifo_exit_key: Optional[str]=None
    fifo_exit_action: Optional[str]=None
    fifo_music: Optional[str]=None
    fifo_order: Optional[int]=None

class FifoMemberCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    fifo_uuid: Optional[UUID]=None
    member_contact: Optional[str]=None
    member_call_timeout: Optional[int]=None
    member_simultaneous: Optional[int]=None
    member_wrap_up_time: Optional[int]=None
    member_enabled: Optional[bool]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data,dict) and "domain_uuid" in data and "domain_id" not in data:
            data["domain_id"]=data.pop("domain_uuid")
        return data

class FifoMemberRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    fifo_member_uuid: UUID
    domain_id: UUID
    fifo_uuid: UUID
    member_contact: Optional[str]=None
    member_call_timeout: Optional[int]=None
    member_simultaneous: Optional[int]=None
    member_wrap_up_time: Optional[int]=None
    member_enabled: bool
    created_at: datetime
    updated_at: datetime

class FifoMemberUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    member_contact: Optional[str]=None
    member_call_timeout: Optional[int]=None
    member_simultaneous: Optional[int]=None
    member_wrap_up_time: Optional[int]=None
    member_enabled: Optional[bool]=None
