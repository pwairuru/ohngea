import re
from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlalchemy import JSON, Column
from sqlmodel import Field, SQLModel

from app.db import uuid7


class TimeConditionBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    name: str = Field(max_length=128, description="Time condition name")
    extension: str = Field(max_length=32, description="Time condition extension 2-7 digits")
    context: Optional[str] = Field(default=None, max_length=128, description="Dialplan context, defaults to domain name")
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)
    dialplan_uuid: Optional[UUID] = Field(default=None, foreign_key="dialplans.dialplan_uuid", index=True, ondelete="SET NULL")
    # FreeSWITCH condition types
    time_hour: Optional[str] = Field(default=None, max_length=64)
    time_minute: Optional[str] = Field(default=None, max_length=64)
    time_wday: Optional[str] = Field(default=None, max_length=64, description="Day of week 1-7")
    time_mday: Optional[str] = Field(default=None, max_length=64, description="Day of month 1-31")
    time_mon: Optional[str] = Field(default=None, max_length=64)
    time_year: Optional[str] = Field(default=None, max_length=32)
    time_yday: Optional[str] = Field(default=None, max_length=64)
    time_week: Optional[str] = Field(default=None, max_length=32)
    time_mweek: Optional[str] = Field(default=None, max_length=32)
    minute_of_day: Optional[str] = Field(default=None, max_length=64)
    time_of_day: Optional[str] = Field(default=None, max_length=64)
    date_time: Optional[str] = Field(default=None, max_length=128)
    condition: Optional[str] = Field(default=None, max_length=512, description="Cron-like or human readable e.g. 09:00-17:00 Mon-Fri")
    true_destination: Optional[str] = Field(default=None, max_length=128, description="Destination when condition true")
    false_destination: Optional[str] = Field(default=None, max_length=128, description="Destination when condition false")
    timezone: Optional[str] = Field(default=None, max_length=64)
    details: Optional[dict] = Field(default=None, sa_column=Column(JSON), description="JSONB details for complex conditions / dialplan groups")

    @field_validator("extension")
    @classmethod
    def validate_extension(cls, v: str) -> str:
        if v is None:
            return v
        if not re.match(r"^\d{2,7}$", str(v)):
            raise ValueError("extension must be 2-7 digits")
        return str(v)

    @field_validator("name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        if v is not None and len(v.strip()) == 0:
            raise ValueError("name must not be empty")
        return v.strip() if isinstance(v, str) else v


class TimeCondition(TimeConditionBase, table=True):
    __tablename__ = "time_conditions"
    time_condition_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
