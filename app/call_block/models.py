from datetime import datetime
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class CallBlockBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    call_block_name: str = Field(max_length=128, description="Block name")
    call_block_number: str = Field(max_length=32, description="Blocked number pattern")
    call_block_enabled: bool = Field(default=True)
    call_block_description: Optional[str] = Field(default=None, max_length=512)
    call_block_action: Optional[str] = Field(default="reject", max_length=32)
    call_block_count: Optional[int] = Field(default=0)


class CallBlock(CallBlockBase, table=True):
    __tablename__ = "call_block"
    call_block_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
