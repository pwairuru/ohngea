from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class UserBase(SQLModel):
    username: str = Field(index=True, max_length=64)
    email: Optional[str] = Field(default=None, max_length=255)
    enabled: bool = Field(default=True)


class User(UserBase, table=True):
    __tablename__ = "users"
    user_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    password_hash: str = Field(max_length=255)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    # parity: legacy v_users - contacts FK deferred to avoid circular create
    contact_uuid: Optional[UUID] = Field(default=None, index=True)
    user_status: Optional[str] = Field(default=None, max_length=64)  # Available, On Break, etc
    user_type: Optional[str] = Field(default="default", max_length=32)
    api_key: Optional[str] = Field(default=None, max_length=64)
    user_totp_secret: Optional[str] = Field(default=None, max_length=64)


class Group(SQLModel, table=True):
    __tablename__ = "groups"
    group_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: Optional[UUID] = Field(default=None, foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    name: str = Field(max_length=64, index=True)
    description: Optional[str] = Field(default=None)
    group_level: int = Field(default=10, index=True)  # 10..90 step 10, legacy group_level
    group_protected: bool = Field(default=False)  # legacy group_protected
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class UserGroup(SQLModel, table=True):
    __tablename__ = "user_groups"
    id: UUID = Field(default_factory=uuid7, primary_key=True)
    user_uuid: UUID = Field(foreign_key="users.user_uuid", ondelete="CASCADE")
    group_uuid: UUID = Field(foreign_key="groups.group_uuid", ondelete="CASCADE")
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    group_name: Optional[str] = Field(default=None, max_length=64)  # denorm for legacy parity


class UserSetting(SQLModel, table=True):
    __tablename__ = "user_settings"
    user_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    user_uuid: UUID = Field(foreign_key="users.user_uuid", ondelete="CASCADE", index=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    user_setting_category: str = Field(max_length=64, index=True)
    user_setting_subcategory: Optional[str] = Field(default=None, max_length=64, index=True)
    user_setting_name: Optional[str] = Field(default=None, max_length=64)
    user_setting_value: Optional[str] = Field(default=None, max_length=512)
    user_setting_enabled: bool = Field(default=True)
    user_setting_description: Optional[str] = Field(default=None)


class Permission(SQLModel, table=True):
    __tablename__ = "permissions"
    permission_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    name: str = Field(max_length=64, unique=True, index=True)  # e.g. extension_view
    description: Optional[str] = Field(default=None)


class GroupPermission(SQLModel, table=True):
    __tablename__ = "group_permissions"
    id: UUID = Field(default_factory=uuid7, primary_key=True)
    group_uuid: UUID = Field(foreign_key="groups.group_uuid", ondelete="CASCADE")
    permission_uuid: UUID = Field(foreign_key="permissions.permission_uuid", ondelete="CASCADE")
    # parity: legacy v_group_permissions denorm + flags
    domain_id: Optional[UUID] = Field(default=None, foreign_key="domains.domain_id", index=True)
    group_name: Optional[str] = Field(default=None, max_length=64, index=True)
    permission_name: Optional[str] = Field(default=None, max_length=64, index=True)
    permission_protected: bool = Field(default=False)
    permission_assigned: bool = Field(default=True)
