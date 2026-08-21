from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class ContactBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    name: str = Field(max_length=128)
    number: str = Field(max_length=32, index=True)
    email: Optional[str] = Field(default=None, max_length=255)
    organization: Optional[str] = Field(default=None, max_length=128)


class Contact(ContactBase, table=True):
    __tablename__ = "contacts"
    contact_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
