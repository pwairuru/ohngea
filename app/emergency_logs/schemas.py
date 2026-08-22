from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class EmergencyLogCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    extension: Optional[str]=None
    event: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict) and "domain_uuid" in data and "domain_id" not in data:
            data["domain_id"]=data.pop("domain_uuid")
        return data
class EmergencyLogRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    emergency_log_uuid: UUID
    domain_id: UUID
    extension: Optional[str]=None
    event: Optional[str]=None
    created_at: datetime
