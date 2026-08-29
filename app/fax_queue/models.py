from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7

class FaxQueueBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fax_uuid: Optional[UUID] = Field(default=None, foreign_key="fax.fax_uuid", ondelete="SET NULL")
    origination_uuid: Optional[UUID] = Field(default=None, index=True)
    fax_log_uuid: Optional[UUID] = Field(default=None, index=True)
    fax_date: Optional[datetime] = Field(default=None)
    hostname: Optional[str] = Field(default=None, max_length=255)
    fax_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    fax_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    fax_recipient: Optional[str] = Field(default=None, max_length=256)
    fax_queue_name: Optional[str] = Field(default=None, max_length=128)
    fax_number: Optional[str] = Field(default=None, max_length=32)
    fax_prefix: Optional[str] = Field(default=None, max_length=16)
    fax_email_address: Optional[str] = Field(default=None, max_length=256)
    fax_file: Optional[str] = Field(default=None, max_length=512)
    fax_status: str = Field(default="waiting", max_length=32)
    fax_retry_date: Optional[datetime] = Field(default=None)
    fax_notify_sent: Optional[bool] = Field(default=None)
    fax_notify_date: Optional[datetime] = Field(default=None)
    fax_accountcode: Optional[str] = Field(default=None, max_length=64)
    fax_command: Optional[str] = Field(default=None, max_length=1024)
    fax_response: Optional[str] = Field(default=None, max_length=4096)
    insert_user: Optional[UUID] = Field(default=None)
    update_user: Optional[UUID] = Field(default=None)

class FaxQueue(FaxQueueBase, table=True):
    __tablename__ = "fax_queue"
    fax_queue_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    # worker helpers — nullable for backwards compat
    fax_retry_count: int = Field(default=0)
    fax_error: str | None = Field(default=None, max_length=512)
