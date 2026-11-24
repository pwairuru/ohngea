from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class BridgeBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    bridge_name: str = Field(max_length=128)
    bridge_destination: Optional[str] = Field(default=None, max_length=128)
    bridge_enabled: bool = Field(default=True)
    bridge_description: Optional[str] = Field(default=None, max_length=512)
class Bridge(BridgeBase, table=True):
    __tablename__ = "bridges"
    bridge_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
