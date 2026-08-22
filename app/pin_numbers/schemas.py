from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator

class PinNumberCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    pin_number: Optional[str] = None
    number: Optional[str] = None
    accountcode: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "number" in data and "pin_number" not in data:
                data["pin_number"] = str(data.pop("number"))
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
        return data

class PinNumberRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    pin_number_uuid: UUID
    domain_id: UUID
    pin_number: str
    accountcode: Optional[str] = None
    enabled: bool
    description: Optional[str] = None
    created_at: datetime
    updated_at: datetime

class PinNumberUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    pin_number: Optional[str] = None
    accountcode: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None
