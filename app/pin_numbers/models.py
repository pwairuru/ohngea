from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel
from app.db import uuid7

class PinNumberBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    pin_number: str = Field(max_length=32, description="PIN")
    accountcode: Optional[str] = Field(default=None, max_length=64)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)

class PinNumber(PinNumberBase, table=True):
    __tablename__ = "pin_numbers"
    pin_number_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
