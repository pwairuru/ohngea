from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7

class FifoBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    dialplan_uuid: Optional[UUID] = Field(default=None, foreign_key="dialplans.dialplan_uuid", index=True, ondelete="SET NULL")
    fifo_name: str = Field(max_length=128)
    fifo_extension: Optional[str] = Field(default=None, max_length=32)
    fifo_agent_queue: Optional[str] = Field(default=None, max_length=256)
    fifo_agent_status: Optional[str] = Field(default=None, max_length=128)
    fifo_enabled: bool = Field(default=True)
    fifo_description: Optional[str] = Field(default=None, max_length=512)
    fifo_strategy: Optional[str] = Field(default="fifo", max_length=32)
    fifo_members: Optional[str] = Field(default=None, max_length=4096)
    fifo_timeout_seconds: Optional[str] = Field(default=None, max_length=16)
    fifo_exit_key: Optional[str] = Field(default=None, max_length=8)
    fifo_exit_action: Optional[str] = Field(default=None, max_length=512)
    fifo_music: Optional[str] = Field(default=None, max_length=512)
    fifo_order: Optional[int] = Field(default=None)
    fifo_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    fifo_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    insert_user: Optional[UUID] = Field(default=None)
    update_user: Optional[UUID] = Field(default=None)

class Fifo(FifoBase, table=True):
    __tablename__ = "fifo"
    fifo_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())

class FifoMemberBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fifo_uuid: UUID = Field(foreign_key="fifo.fifo_uuid", index=True, ondelete="CASCADE")
    member_contact: Optional[str] = Field(default=None, max_length=512)
    member_call_timeout: Optional[int] = Field(default=None)
    member_simultaneous: Optional[int] = Field(default=None)
    member_wrap_up_time: Optional[int] = Field(default=None)
    member_enabled: bool = Field(default=True)
    insert_user: Optional[UUID] = Field(default=None)
    update_user: Optional[UUID] = Field(default=None)

class FifoMember(FifoMemberBase, table=True):
    __tablename__ = "fifo_members"
    fifo_member_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
