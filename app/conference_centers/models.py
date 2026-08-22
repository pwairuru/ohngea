from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class ConferenceCenterBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_center_name: str = Field(max_length=128)
    conference_center_extension: Optional[str]=Field(default=None, max_length=32)
    conference_center_pin_length: Optional[int]=Field(default=6)
    conference_center_greeting: Optional[str]=Field(default=None, max_length=512)
    conference_center_description: Optional[str]=Field(default=None, max_length=512)
    conference_center_enabled: bool = Field(default=True)
    dialplan_uuid: Optional[UUID]=Field(default=None)
class ConferenceCenter(ConferenceCenterBase, table=True):
    __tablename__="conference_centers"
    conference_center_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
