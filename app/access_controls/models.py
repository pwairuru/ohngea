from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class AccessControlBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    access_control_name: str = Field(max_length=128)
    access_control_default: str = Field(default="deny", max_length=16)
    access_control_description: Optional[str]=Field(default=None, max_length=512)
class AccessControl(AccessControlBase, table=True):
    __tablename__="access_controls"
    access_control_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
class AccessControlNodeBase(SQLModel):
    access_control_uuid: UUID = Field(foreign_key="access_controls.access_control_uuid", ondelete="CASCADE")
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    node_type: str = Field(max_length=16, description="allow/deny")
    node_cidr: str = Field(max_length=64)
    node_description: Optional[str]=Field(default=None, max_length=512)
class AccessControlNode(AccessControlNodeBase, table=True):
    __tablename__="access_control_nodes"
    access_control_node_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
