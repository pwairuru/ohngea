from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class ConferenceProfileBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    profile_name: str = Field(max_length=128)
    profile_enabled: bool = Field(default=True)
    profile_description: Optional[str]=Field(default=None, max_length=512)
class ConferenceProfile(ConferenceProfileBase, table=True):
    __tablename__="conference_profiles"
    conference_profile_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
class ConferenceProfileParamBase(SQLModel):
    conference_profile_uuid: UUID = Field(foreign_key="conference_profiles.conference_profile_uuid", ondelete="CASCADE")
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    profile_param_name: str = Field(max_length=128)
    profile_param_value: str = Field(max_length=512)
    profile_param_enabled: bool = Field(default=True)
    profile_param_description: Optional[str]=Field(default=None, max_length=512)
class ConferenceProfileParam(ConferenceProfileParamBase, table=True):
    __tablename__="conference_profile_params"
    conference_profile_param_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
