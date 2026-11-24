from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class EventGuardLogBase(SQLModel):
    hostname: Optional[str]=Field(default=None, max_length=128)
    log_date: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    filter: Optional[str]=Field(default=None, max_length=128)
    ip_address: Optional[str]=Field(default=None, max_length=64)
    extension: Optional[str]=Field(default=None, max_length=32)
    user_agent: Optional[str]=Field(default=None, max_length=512)
    log_status: Optional[str]=Field(default=None, max_length=32)
    insert_user: Optional[UUID]=Field(default=None)
class EventGuardLog(EventGuardLogBase, table=True):
    __tablename__="event_guard_logs"
    event_guard_log_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
