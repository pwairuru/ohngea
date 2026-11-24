from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class ExtensionSettingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    extension_uuid: UUID = Field(foreign_key="extensions.extension_uuid", ondelete="CASCADE")
    extension_setting_type: Optional[str]=Field(default=None, max_length=64)
    extension_setting_name: str = Field(max_length=128)
    extension_setting_value: str = Field(max_length=512)
    extension_setting_enabled: bool = Field(default=True)
    extension_setting_description: Optional[str]=Field(default=None, max_length=512)
class ExtensionSetting(ExtensionSettingBase, table=True):
    __tablename__="extension_settings"
    extension_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
