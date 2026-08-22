from datetime import datetime
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DestinationBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    destination_type: str = Field(max_length=32, description="inbound, outbound, etc")
    destination_number: str = Field(max_length=32, description="DID number")
    destination_prefix: Optional[str] = Field(default=None, max_length=32)
    destination_enabled: bool = Field(default=True)
    destination_description: Optional[str] = Field(default=None, max_length=512)
    destination_context: Optional[str] = Field(default="public", max_length=128)
    destination_app: Optional[str] = Field(default=None, max_length=64)
    destination_data: Optional[str] = Field(default=None, max_length=512)
    destination_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    destination_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    destination_hold_music: Optional[str] = Field(default=None, max_length=128)
    destination_record: Optional[bool] = Field(default=False)


class Destination(DestinationBase, table=True):
    __tablename__ = "destinations"
    destination_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
