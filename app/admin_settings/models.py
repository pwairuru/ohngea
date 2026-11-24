"""Core admin editor tables: default/domain settings, email templates, menu items, user logs, notifications, services."""
from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DefaultSetting(SQLModel, table=True):
    # Legacy shape from 035_provision_defaults (provision vendor defaults live here).
    __tablename__ = "default_settings"
    default_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    default_setting_category: str = Field(max_length=64, index=True)
    default_setting_subcategory: str = Field(max_length=64, default="")
    default_setting_name: str = Field(max_length=64, index=True)
    default_setting_value: Optional[str] = Field(default=None, max_length=1024)
    default_setting_enabled: bool = Field(default=True)
    default_setting_description: Optional[str] = Field(default=None, max_length=512)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class EmailTemplate(SQLModel, table=True):
    __tablename__ = "email_templates"
    email_template_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: Optional[UUID] = Field(default=None, foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    template_name: str = Field(max_length=128, index=True)
    template_language: Optional[str] = Field(default=None, max_length=16)
    template_subject: Optional[str] = Field(default=None, max_length=512)
    template_body: Optional[str] = Field(default=None)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class MenuItem(SQLModel, table=True):
    __tablename__ = "menu_items"
    menu_item_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    parent_uuid: Optional[UUID] = Field(default=None, foreign_key="menu_items.menu_item_uuid", ondelete="CASCADE")
    title: str = Field(max_length=128)
    path: Optional[str] = Field(default=None, max_length=256)
    icon: Optional[str] = Field(default=None, max_length=64)
    item_order: int = Field(default=0)
    groups: Optional[str] = Field(default=None, max_length=512, description="csv group names allowed")
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class UserLog(SQLModel, table=True):
    __tablename__ = "user_logs"
    user_log_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: Optional[UUID] = Field(default=None, foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    user_uuid: Optional[UUID] = Field(default=None, index=True)
    username: Optional[str] = Field(default=None, max_length=128, index=True)
    action: str = Field(max_length=64, index=True)
    detail: Optional[str] = Field(default=None, max_length=1024)
    ip_address: Optional[str] = Field(default=None, max_length=64)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc), index=True)


class Notification(SQLModel, table=True):
    __tablename__ = "notifications"
    notification_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: Optional[UUID] = Field(default=None, foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    user_uuid: Optional[UUID] = Field(default=None, index=True)
    title: str = Field(max_length=256)
    body: Optional[str] = Field(default=None, max_length=2048)
    link: Optional[str] = Field(default=None, max_length=512)
    read: bool = Field(default=False)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc), index=True)


class Service(SQLModel, table=True):
    __tablename__ = "services"
    service_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    name: str = Field(max_length=128, index=True)
    host: Optional[str] = Field(default=None, max_length=128)
    port: Optional[int] = Field(default=None)
    enabled: bool = Field(default=True)
    service_order: int = Field(default=0)
    description: Optional[str] = Field(default=None, max_length=512)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
