from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class NumberTranslationBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    number_translation_name: str = Field(max_length=128)
    number_translation_regex: str = Field(max_length=512)
    number_translation_replace: str = Field(max_length=512)
    number_translation_enabled: bool = Field(default=True)
    number_translation_description: Optional[str]=Field(default=None, max_length=512)
class NumberTranslation(NumberTranslationBase, table=True):
    __tablename__="number_translations"
    number_translation_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
