from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class StreamBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    stream_name: str = Field(max_length=128)
    stream_location: Optional[str] = Field(default=None, max_length=512)
    stream_enabled: bool = Field(default=True)
    stream_description: Optional[str] = Field(default=None, max_length=512)
class Stream(StreamBase, table=True):
    __tablename__ = "streams"
    stream_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
