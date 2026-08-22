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
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class Group(SQLModel, table=True):
    __tablename__ = "groups"
    group_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    name: str = Field(max_length=64, index=True)
    description: Optional[str] = Field(default=None)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class UserGroup(SQLModel, table=True):
    __tablename__ = "user_groups"
    id: UUID = Field(default_factory=uuid7, primary_key=True)
    user_uuid: UUID = Field(foreign_key="users.user_uuid", ondelete="CASCADE")
    group_uuid: UUID = Field(foreign_key="groups.group_uuid", ondelete="CASCADE")
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")


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
