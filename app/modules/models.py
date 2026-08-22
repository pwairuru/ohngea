from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class ModuleBase(SQLModel):
    module_label: str = Field(max_length=128)
    module_name: str = Field(max_length=128)
    module_category: Optional[str]=Field(default=None, max_length=64)
    module_enabled: bool = Field(default=True)
    module_default_enabled: bool = Field(default=False)
    module_description: Optional[str]=Field(default=None, max_length=512)
class Module(ModuleBase, table=True):
    __tablename__="modules"
    module_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
