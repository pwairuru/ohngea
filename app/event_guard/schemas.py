from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict
class EventGuardLogCreate(BaseModel):
    model_config=ConfigDict(extra="allow")
    hostname: Optional[str]=None
    ip_address: Optional[str]=None
    extension: Optional[str]=None
    filter: Optional[str]=None
    log_status: Optional[str]=None
class EventGuardLogRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    event_guard_log_uuid: UUID
    hostname: Optional[str]=None
    ip_address: Optional[str]=None
    extension: Optional[str]=None
    log_status: Optional[str]=None
    created_at: datetime
