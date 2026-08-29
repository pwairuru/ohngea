from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class VarBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    var_category: str = Field(max_length=64, description="category like Provision / Features")
    var_name: str = Field(max_length=128)
    var_value: str = Field(max_length=512)
    var_enabled: bool = Field(default=True)
    var_description: Optional[str]=Field(default=None, max_length=512)
    var_order: Optional[int]=Field(default=0)
    var_command: Optional[str]=Field(default=None, max_length=512, description="command for provision vars")
    var_hostname: Optional[str]=Field(default=None, max_length=255, description="hostname for provision vars")
class Var(VarBase, table=True):
    __tablename__="vars"
    var_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
