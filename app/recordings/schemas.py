from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator

class RecordingCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID] = None
    name: str
    file_path: Optional[str] = None
    duration: Optional[int] = None
    file_size: Optional[int] = None
    s3_bucket: Optional[str] = None
    content_type: Optional[str] = None
    storage_backend: Optional[str] = None
    recording_voice: Optional[str] = None
    recording_speed: Optional[float] = None
    recording_message: Optional[str] = None
    recording_base64: Optional[str] = None

    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls, data):
        if isinstance(data, dict):
            if "domain_uuid" in data and "domain_id" not in data:
                data["domain_id"] = data.pop("domain_uuid")
            # allow file_name -> file_path
            if "file_name" in data and "file_path" not in data:
                data["file_path"] = data.pop("file_name")
        return data

class RecordingRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    recording_uuid: UUID
    domain_id: UUID
    name: str
    file_path: str
    duration: Optional[int] = None
    file_size: Optional[int] = None
    s3_bucket: Optional[str] = None
    content_type: Optional[str] = None
    storage_backend: str = "s3"
    recording_voice: Optional[str] = None
    recording_speed: Optional[float] = None
    recording_message: Optional[str] = None
    recording_base64: Optional[str] = None
    created_at: datetime
    created_by: Optional[UUID] = None

class RecordingUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    name: Optional[str] = None
    file_path: Optional[str] = None
    duration: Optional[int] = None
    file_size: Optional[int] = None
    content_type: Optional[str] = None
    recording_voice: Optional[str] = None
    recording_speed: Optional[float] = None
    recording_message: Optional[str] = None
    recording_base64: Optional[str] = None
