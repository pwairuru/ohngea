from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DomainBase(SQLModel):
    domain_name: str = Field(index=True, unique=True, max_length=255, description="FQDN like tenant.example.com")
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)


class Domain(DomainBase, table=True):
    __tablename__ = "domains"
    domain_id: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class DomainSettings(SQLModel, table=True):
    __tablename__ = "domain_settings"
    id: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    category: str = Field(max_length=64)
    subcategory: str = Field(max_length=64, default="")
    key: str = Field(max_length=64)
    value: str = Field(max_length=1024)
    enabled: bool = Field(default=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class DomainPlugin(SQLModel, table=True):
    __tablename__ = "domain_plugins"
    id: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    plugin_name: str = Field(max_length=64, index=True)
    enabled: bool = Field(default=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
