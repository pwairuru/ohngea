from datetime import datetime
from typing import Optional, List, Any
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class CallCenterCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    queue_name: Optional[str]=None
    name: Optional[str]=None
    queue_extension: Optional[str]=None
    extension: Optional[str]=None
    queue_strategy: Optional[str]=None
    queue_enabled: Optional[bool]=None
    enabled: Optional[bool]=None
    queue_description: Optional[str]=None
    description: Optional[str]=None
    agents: Optional[List[Any]]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "name" in data and "queue_name" not in data: data["queue_name"]=data.pop("name")
            if "extension" in data and "queue_extension" not in data: data["queue_extension"]=str(data.pop("extension"))
            if "enabled" in data and "queue_enabled" not in data: data["queue_enabled"]=data.pop("enabled")
            if "description" in data and "queue_description" not in data: data["queue_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class CallCenterRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    call_center_queue_uuid: UUID
    domain_id: UUID
    queue_name: str
    queue_extension: Optional[str]=None
    queue_strategy: Optional[str]=None
    queue_enabled: bool
    queue_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
    agents: List[Any]=[]
class CallCenterUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    queue_name: Optional[str]=None
    queue_extension: Optional[str]=None
    queue_strategy: Optional[str]=None
    queue_enabled: Optional[bool]=None
    queue_description: Optional[str]=None

class AgentCreate(BaseModel):
    agent_name: str
    agent_type: Optional[str]="callback"
    agent_contact: Optional[str]=None
    agent_status: Optional[str]="Logged Out"
