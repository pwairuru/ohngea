from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class CallRecordingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    cdr_uuid: Optional[UUID] = Field(default=None, foreign_key="cdr.cdr_uuid", index=True, ondelete="SET NULL")
    recording_path: Optional[str] = Field(default=None, max_length=512, description="Path to recording file")
    recording_name: Optional[str] = Field(default=None, max_length=255, description="Recording filename")
    transcription: Optional[str] = Field(default=None, description="Transcribed text")
    summary: Optional[str] = Field(default=None, description="AI summary")


class CallRecording(CallRecordingBase, table=True):
    __tablename__ = "call_recordings"
    call_recording_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
