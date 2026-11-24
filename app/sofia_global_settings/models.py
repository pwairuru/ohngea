from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class SofiaGlobalSettingBase(SQLModel):
    global_setting_name: str = Field(max_length=128)
    global_setting_value: str = Field(max_length=512)
    global_setting_enabled: bool = Field(default=True)
    global_setting_description: Optional[str]=Field(default=None, max_length=512)
class SofiaGlobalSetting(SofiaGlobalSettingBase, table=True):
    __tablename__="sofia_global_settings"
    sofia_global_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
