from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class FaxQueueBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fax_uuid: Optional[UUID]=Field(default=None, foreign_key="fax.fax_uuid", ondelete="SET NULL")
    fax_queue_name: Optional[str]=Field(default=None, max_length=128)
    fax_number: Optional[str]=Field(default=None, max_length=32)
    fax_status: str = Field(default="waiting", max_length=32)
    fax_file: Optional[str]=Field(default=None, max_length=512)
class FaxQueue(FaxQueueBase, table=True):
    __tablename__="fax_queue"
    fax_queue_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    # worker helpers — nullable for backwards compat
    fax_retry_count: int = Field(default=0)
    fax_error: str | None = Field(default=None, max_length=512)
