from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class CdrBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    caller_id_number: Optional[str] = Field(default=None, max_length=32)
    caller_id_name: Optional[str] = Field(default=None, max_length=128)
    destination_number: str = Field(max_length=32, index=True)
    start_stamp: datetime = Field(index=True)
    answer_stamp: Optional[datetime] = Field(default=None)
    end_stamp: Optional[datetime] = Field(default=None)
    duration: Optional[int] = Field(default=None)
    billsec: Optional[int] = Field(default=None)
    direction: Optional[str] = Field(default=None, max_length=16)
    sip_call_id: Optional[str] = Field(default=None, max_length=128)
    hangup_cause: Optional[str] = Field(default=None, max_length=64)
    # parity additions — at least 15 critical nullable columns
    accountcode: Optional[str] = Field(default=None, max_length=64)
    context: Optional[str] = Field(default=None, max_length=128)
    leg: Optional[str] = Field(default=None, max_length=16, description="a-leg or b-leg")
    status: Optional[str] = Field(default=None, max_length=32)
    billmsec: Optional[int] = Field(default=None, description="billing milliseconds")
    mduration: Optional[int] = Field(default=None, description="media duration")
    hold_accum_seconds: Optional[int] = Field(default=None)
    bridge_uuid: Optional[str] = Field(default=None, max_length=128)
    conference_id: Optional[str] = Field(default=None, max_length=128)
    record_path: Optional[str] = Field(default=None, max_length=512)
    record_name: Optional[str] = Field(default=None, max_length=255)
    json_text: Optional[str] = Field(default=None, description="json payload")
    xml_text: Optional[str] = Field(default=None, description="xml cdr payload")
    processed: Optional[bool] = Field(default=None)
    caller_extension: Optional[str] = Field(default=None, max_length=32)
    destination_extension: Optional[str] = Field(default=None, max_length=32)


class Cdr(CdrBase, table=True):
    __tablename__ = "cdr"
    cdr_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

    __table_args__ = {"extend_existing": True}
