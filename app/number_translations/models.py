from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class NumberTranslationBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    number_translation_name: str = Field(max_length=128)
    # regex/replace moved to NumberTranslationDetail; kept on parent for compat (nullable/default)
    number_translation_regex: Optional[str] = Field(default="", max_length=512)
    number_translation_replace: Optional[str] = Field(default="", max_length=512)
    number_translation_enabled: bool = Field(default=True)
    number_translation_description: Optional[str]=Field(default=None, max_length=512)
class NumberTranslation(NumberTranslationBase, table=True):
    __tablename__="number_translations"
    number_translation_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

class NumberTranslationDetailBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    number_translation_uuid: UUID = Field(foreign_key="number_translations.number_translation_uuid", index=True, ondelete="CASCADE")
    number_translation_detail_regex: str = Field(max_length=512)
    number_translation_detail_replace: str = Field(max_length=512)
    number_translation_detail_order: int = Field(default=0)

class NumberTranslationDetail(NumberTranslationDetailBase, table=True):
    __tablename__ = "number_translation_details"
    number_translation_detail_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
