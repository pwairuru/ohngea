from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class VoicemailGreetingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    voicemail_id: str = Field(max_length=32, description="voicemail mailbox id")
    greeting_id: int = Field(default=1)
    greeting_name: Optional[str]=Field(default=None, max_length=128)
    greeting_filename: Optional[str]=Field(default=None, max_length=512)
    greeting_description: Optional[str]=Field(default=None, max_length=512)
    greeting_base64: Optional[str]=Field(default=None)
    greeting_voice: Optional[str]=Field(default=None, max_length=32, description="TTS voice for greeting")
    greeting_message: Optional[str]=Field(default=None, max_length=1024, description="Text for TTS greeting message")
class VoicemailGreeting(VoicemailGreetingBase, table=True):
    __tablename__="voicemail_greetings"
    voicemail_greeting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
