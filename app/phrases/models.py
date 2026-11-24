from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class PhraseBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    phrase_name: str = Field(max_length=128)
    phrase_language: Optional[str] = Field(default="en-us", max_length=16)
    phrase_enabled: bool = Field(default=True)
    phrase_description: Optional[str] = Field(default=None, max_length=512)
class Phrase(PhraseBase, table=True):
    __tablename__ = "phrases"
    phrase_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
class PhraseDetailBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    phrase_uuid: UUID = Field(foreign_key="phrases.phrase_uuid", index=True, ondelete="CASCADE")
    phrase_detail_group: Optional[int] = Field(default=None)
    phrase_detail_tag: Optional[str] = Field(default=None, max_length=64)
    phrase_detail_pattern: Optional[str] = Field(default=None, max_length=128)
    phrase_detail_function: Optional[str] = Field(default=None, max_length=64)
    phrase_detail_data: Optional[str] = Field(default=None, max_length=512)
    phrase_detail_method: Optional[str] = Field(default=None, max_length=32)
    phrase_detail_type: Optional[str] = Field(default=None, max_length=32)
    phrase_detail_order: int = Field(default=0)
class PhraseDetail(PhraseDetailBase, table=True):
    __tablename__ = "phrase_details"
    phrase_detail_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
