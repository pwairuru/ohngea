from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class IvrMenuBase(SQLModel):
    name: str = Field(max_length=128)
    extension: Optional[str] = Field(default=None, max_length=32)
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    # full parity
    greet_long: Optional[str] = Field(default=None, max_length=256)
    greet_short: Optional[str] = Field(default=None, max_length=256)
    invalid_sound: Optional[str] = Field(default=None, max_length=256)
    exit_sound: Optional[str] = Field(default=None, max_length=256)
    confirm_macro: Optional[str] = Field(default=None, max_length=128)
    confirm_key: Optional[str] = Field(default=None, max_length=8)
    tts_engine: Optional[str] = Field(default=None, max_length=32)
    tts_voice: Optional[str] = Field(default=None, max_length=32)
    confirm_attempts: Optional[int] = Field(default=3)
    timeout: Optional[int] = Field(default=10000)
    inter_digit_timeout: Optional[int] = Field(default=2000)
    max_failures: Optional[int] = Field(default=3)
    max_timeouts: Optional[int] = Field(default=3)
    digit_len: Optional[int] = Field(default=4)
    description: Optional[str] = Field(default=None, max_length=512)


class IvrMenu(IvrMenuBase, table=True):
    __tablename__ = "ivr_menus"
    ivr_menu_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class IvrMenuOption(SQLModel, table=True):
    __tablename__ = "ivr_menu_options"
    option_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    ivr_menu_uuid: UUID = Field(foreign_key="ivr_menus.ivr_menu_uuid", ondelete="CASCADE", index=True)
    digits: str = Field(max_length=8)
    action: str = Field(max_length=64)
    param: str = Field(max_length=255)
    order: int = Field(default=0)
