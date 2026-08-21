from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel

from app.extensions.models import ExtensionBase


class ExtensionCreate(ExtensionBase):
    pass


class ExtensionUpdate(BaseModel):
    extension: Optional[str] = None
    number_alias: Optional[str] = None
    effective_caller_id_name: Optional[str] = None
    effective_caller_id_number: Optional[str] = None
    call_group: Optional[str] = None
    enabled: Optional[bool] = None


class ExtensionRead(ExtensionBase):
    extension_uuid: UUID
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True
