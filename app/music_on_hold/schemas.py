from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel, ConfigDict, field_validator, model_validator


class MusicOnHoldCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    music_name: Optional[str] = None
    name: Optional[str] = None
    music_path: Optional[str] = None
    path: Optional[str] = None
    music_rate: Optional[int] = None
    rate: Optional[int] = None
    music_shuffle: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "name" in data and "music_name" not in data:
                data["music_name"] = data.pop("name")
            if "path" in data and "music_path" not in data:
                data["music_path"] = data.pop("path")
            if "rate" in data and "music_rate" not in data:
                data["music_rate"] = data.pop("rate")
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
        return data

    @field_validator("music_rate")
    @classmethod
    def validate_rate(cls, v):
        if v is None:
            return v
        if v not in (8000, 16000, 32000, 48000):
            raise ValueError("music_rate must be 8000, 16000, 32000, 48000")
        return v


class MusicOnHoldRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    music_on_hold_uuid: UUID
    domain_id: UUID
    music_name: str
    music_path: str
    music_rate: Optional[int] = None
    music_shuffle: Optional[str] = None
    enabled: bool
    description: Optional[str] = None
    created_at: datetime
    updated_at: datetime


class MusicOnHoldUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    music_name: Optional[str] = None
    music_path: Optional[str] = None
    music_rate: Optional[int] = None
    music_shuffle: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None
