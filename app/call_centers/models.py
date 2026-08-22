from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class CallCenterBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    queue_name: str = Field(max_length=128)
    queue_extension: Optional[str]=Field(default=None, max_length=32)
    queue_strategy: Optional[str]=Field(default="longest-idle-agent", max_length=32)
    queue_enabled: bool = Field(default=True)
    queue_description: Optional[str]=Field(default=None, max_length=512)
    queue_greeting: Optional[str]=Field(default=None, max_length=128)
    queue_moh_sound: Optional[str]=Field(default=None, max_length=128)
class CallCenter(CallCenterBase, table=True):
    __tablename__="call_centers"
    call_center_queue_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
class CallCenterAgentBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    call_center_queue_uuid: UUID = Field(foreign_key="call_centers.call_center_queue_uuid", index=True, ondelete="CASCADE")
    agent_name: str = Field(max_length=128)
    agent_type: Optional[str]=Field(default="callback", max_length=32)
    agent_contact: Optional[str]=Field(default=None, max_length=128)
    agent_status: Optional[str]=Field(default="Logged Out", max_length=32)
class CallCenterAgent(CallCenterAgentBase, table=True):
    __tablename__="call_center_agents"
    call_center_agent_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
