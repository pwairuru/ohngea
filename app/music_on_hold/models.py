from datetime import datetime
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class MusicOnHoldBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    music_name: str = Field(max_length=128, description="MOH name")
    music_path: str = Field(max_length=512, description="Path or streaming URL")
    music_rate: Optional[int] = Field(default=8000, description="Sample rate 8000/16000/32000/48000")
    music_shuffle: Optional[str] = Field(default="false", max_length=16)
    music_channels: Optional[int] = Field(default=1)
    music_interval: Optional[int] = Field(default=20)
    music_timer_name: Optional[str] = Field(default=None, max_length=64)
    music_chime_list: Optional[str] = Field(default=None, max_length=512)
    music_chime_freq: Optional[str] = Field(default=None, max_length=64)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)


class MusicOnHold(MusicOnHoldBase, table=True):
    __tablename__ = "music_on_hold"
    music_on_hold_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
