from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7

class DomainLimitBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    limit_name: str = Field(max_length=64, index=True)
    limit_value: Optional[str] = Field(default=None, max_length=128)
    limit_category: Optional[str] = Field(default=None, max_length=64)
    limit_enabled: bool = Field(default=True)
    limit_description: Optional[str] = Field(default=None, max_length=512)

class DomainLimit(DomainLimitBase, table=True):
    __tablename__ = "domain_limits"
    domain_limit_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
