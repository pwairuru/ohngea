from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class CallBroadcastBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    broadcast_name: str = Field(max_length=128)
    broadcast_description: Optional[str]=Field(default=None, max_length=512)
    broadcast_timeout: Optional[int]=Field(default=30)
    broadcast_concurrent_limit: Optional[int]=Field(default=1)
    broadcast_caller_id_name: Optional[str]=Field(default=None, max_length=128)
    broadcast_caller_id_number: Optional[str]=Field(default=None, max_length=32)
    broadcast_destination_type: Optional[str]=Field(default=None, max_length=32)
    broadcast_phone_numbers: Optional[str]=Field(default=None, description="newline separated")
    broadcast_enabled: bool = Field(default=True)
class CallBroadcast(CallBroadcastBase, table=True):
    __tablename__="call_broadcasts"
    call_broadcast_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
