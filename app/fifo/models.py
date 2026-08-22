from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class FifoBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fifo_name: str = Field(max_length=128)
    fifo_extension: Optional[str] = Field(default=None, max_length=32)
    fifo_enabled: bool = Field(default=True)
    fifo_description: Optional[str]=Field(default=None, max_length=512)
    fifo_strategy: Optional[str]=Field(default="fifo", max_length=32)
    fifo_caller_id_name: Optional[str]=Field(default=None, max_length=128)
    fifo_caller_id_number: Optional[str]=Field(default=None, max_length=32)
class Fifo(FifoBase, table=True):
    __tablename__="fifo"
    fifo_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
