from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class RecordingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    name: str = Field(max_length=255)
    file_path: str = Field(max_length=512, description="S3 key recordings/{domain_id}/...")
    duration: Optional[int] = Field(default=None)
    file_size: Optional[int] = Field(default=None)
    created_by: Optional[UUID] = Field(default=None, foreign_key="users.user_uuid")
    s3_bucket: Optional[str] = Field(default=None, max_length=128, description="S3 bucket override")
    content_type: Optional[str] = Field(default=None, max_length=128)
    storage_backend: str = Field(default="s3", max_length=32, description="s3|local")


class Recording(RecordingBase, table=True):
    __tablename__ = "recordings"
    recording_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
