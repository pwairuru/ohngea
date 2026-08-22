import re
from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlmodel import Field, SQLModel

from app.db import uuid7


class FaxBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fax_extension: str = Field(max_length=32, description="Fax extension 2-7 digits")
    fax_name: str = Field(max_length=128, description="Fax name")
    fax_email: Optional[str] = Field(default=None, max_length=255)
    fax_pin_number: Optional[str] = Field(default=None, max_length=32)
    fax_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    fax_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    fax_forward_number: Optional[str] = Field(default=None, max_length=32)
    fax_toll_allow: Optional[str] = Field(default=None, max_length=32)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)

    @field_validator("fax_extension")
    @classmethod
    def validate_ext(cls, v: str) -> str:
        if not re.match(r"^\d{2,7}$", v):
            raise ValueError("fax_extension must be 2-7 digits")
        return v


class Fax(FaxBase, table=True):
    __tablename__ = "fax"
    fax_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
