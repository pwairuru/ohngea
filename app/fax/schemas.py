from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, ConfigDict, field_validator, model_validator
import re


class FaxCreate(BaseModel):
    model_config = ConfigDict(extra="allow")
    domain_id: Optional[UUID] = None
    fax_extension: Optional[str] = None
    extension: Optional[str] = None
    fax_name: Optional[str] = None
    name: Optional[str] = None
    fax_email: Optional[str] = None
    email: Optional[str] = None
    fax_pin_number: Optional[str] = None
    fax_caller_id_name: Optional[str] = None
    fax_caller_id_number: Optional[str] = None
    fax_forward_number: Optional[str] = None
    fax_toll_allow: Optional[str] = None
    enabled: Optional[bool] = True
    description: Optional[str] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "extension" in data and "fax_extension" not in data:
                data["fax_extension"] = data.pop("extension")
            if "name" in data and "fax_name" not in data:
                data["fax_name"] = data.pop("name")
            if "email" in data and "fax_email" not in data:
                data["fax_email"] = data.pop("email")
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
            if "fax_description" in data and "description" not in data:
                data["description"] = data.pop("fax_description")
        return data

    @field_validator("fax_extension")
    @classmethod
    def validate_ext(cls, v):
        if v is None:
            return v
        if not re.match(r"^\d{2,7}$", str(v)):
            raise ValueError("fax_extension must be 2-7 digits")
        return str(v)


class FaxRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    fax_uuid: UUID
    domain_id: UUID
    fax_extension: str
    fax_name: str
    fax_email: Optional[str] = None
    fax_pin_number: Optional[str] = None
    fax_caller_id_name: Optional[str] = None
    fax_caller_id_number: Optional[str] = None
    fax_forward_number: Optional[str] = None
    fax_toll_allow: Optional[str] = None
    enabled: bool
    description: Optional[str] = None
    created_at: datetime
    updated_at: datetime


class FaxUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    fax_extension: Optional[str] = None
    fax_name: Optional[str] = None
    fax_email: Optional[str] = None
    fax_pin_number: Optional[str] = None
    fax_caller_id_name: Optional[str] = None
    fax_caller_id_number: Optional[str] = None
    fax_forward_number: Optional[str] = None
    fax_toll_allow: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None

    @field_validator("fax_extension")
    @classmethod
    def validate_ext(cls, v):
        if v is None:
            return v
        if not re.match(r"^\d{2,7}$", str(v)):
            raise ValueError("fax_extension must be 2-7 digits")
        return str(v)
