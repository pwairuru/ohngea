from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class ConferenceControlBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    control_name: str = Field(max_length=128)
    control_enabled: bool = Field(default=True)
    control_description: Optional[str]=Field(default=None, max_length=512)
class ConferenceControl(ConferenceControlBase, table=True):
    __tablename__="conference_controls"
    conference_control_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
class ConferenceControlDetailBase(SQLModel):
    conference_control_uuid: UUID = Field(foreign_key="conference_controls.conference_control_uuid", ondelete="CASCADE")
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    control_digits: str = Field(max_length=16)
    control_action: str = Field(max_length=64)
    control_data: Optional[str]=Field(default=None, max_length=512)
    control_enabled: bool = Field(default=True)
class ConferenceControlDetail(ConferenceControlDetailBase, table=True):
    __tablename__="conference_control_details"
    conference_control_detail_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
