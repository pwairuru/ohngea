import re
from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlmodel import Field, SQLModel

from app.db import uuid7


class CallFlowBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    call_flow_name: str = Field(max_length=128, description="Call flow name")
    call_flow_extension: str = Field(max_length=32, description="Feature code extension 2-7 digits or *code")
    call_flow_feature_code: Optional[str] = Field(default=None, max_length=32, description="Feature code like *123")
    call_flow_context: str = Field(default="public", max_length=128, description="Dialplan context")
    call_flow_enabled: bool = Field(default=True)
    call_flow_description: Optional[str] = Field(default=None, max_length=512)
    call_flow_alternate_label: Optional[str] = Field(default=None, max_length=128)
    call_flow_alternate_destination: Optional[str] = Field(default=None, max_length=128)
    call_flow_sound: Optional[str] = Field(default=None, max_length=128)

    @field_validator("call_flow_extension")
    @classmethod
    def validate_ext(cls, v: str) -> str:
        # allow * and # prefix for feature codes
        if not re.match(r"^[*#+]?\d{2,7}$", v):
            raise ValueError("call_flow_extension must be 2-7 digits, optionally prefixed with *#+")
        return v


class CallFlow(CallFlowBase, table=True):
    __tablename__ = "call_flows"
    call_flow_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
