from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class EmailQueueBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    email_date: datetime = Field(default_factory=lambda: datetime.utcnow())
    email_from: str = Field(max_length=512)
    email_to: str = Field(max_length=512)
    email_subject: Optional[str]=Field(default=None, max_length=512)
    email_body: Optional[str]=Field(default=None)
    email_status: str = Field(default="waiting", max_length=32)
    email_retry_count: int = Field(default=0)
    email_error: str | None = Field(default=None, max_length=512)
    email_attempts: int = Field(default=0)
class EmailQueue(EmailQueueBase, table=True):
    __tablename__="email_queue"
    email_queue_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())

class EmailQueueAttachmentBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    email_queue_uuid: UUID = Field(foreign_key="email_queue.email_queue_uuid", index=True, ondelete="CASCADE")
    email_attachment_mime_type: Optional[str] = Field(default=None, max_length=128)
    email_attachment_type: Optional[str] = Field(default=None, max_length=32)
    email_attachment_path: Optional[str] = Field(default=None, max_length=512)
    email_attachment_name: Optional[str] = Field(default=None, max_length=255)
    email_attachment_base64: Optional[str] = Field(default=None)
    email_attachment_cid: Optional[str] = Field(default=None, max_length=255)

class EmailQueueAttachment(EmailQueueAttachmentBase, table=True):
    __tablename__ = "email_queue_attachments"
    email_queue_attachment_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
