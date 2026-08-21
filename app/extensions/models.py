from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class ExtensionBase(SQLModel):
    extension: str = Field(regex=r"^\d{2,7}$", description="Extension number 2-7 digits")
    number_alias: Optional[str] = Field(default=None, max_length=32)
    effective_caller_id_name: str = Field(max_length=128)
    effective_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    call_group: Optional[str] = Field(default=None, max_length=32)
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")


class Extension(ExtensionBase, table=True):
    __tablename__ = "extensions"
    extension_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
