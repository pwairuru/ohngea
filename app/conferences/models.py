import re
from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlmodel import Field, SQLModel

from app.db import uuid7


class ConferenceBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_name: str = Field(max_length=128, description="Conference name")
    conference_extension: str = Field(max_length=32, description="Conference extension 2-7 digits")
    conference_pin_number: Optional[str] = Field(default=None, max_length=32, description="Optional PIN")
    conference_profile: str = Field(default="default", max_length=64, description="Conference profile (default, wideband, etc.)")
    conference_enabled: bool = Field(default=True)
    conference_description: Optional[str] = Field(default=None, max_length=512)
    conference_flags: Optional[str] = Field(default=None, max_length=255, description="Optional flags mute|deaf|waste|moderator")
    conference_order: Optional[int] = Field(default=None, description="Order number")
    conference_context: Optional[str] = Field(default=None, max_length=128, description="Dialplan context")
    conference_email_address: Optional[str] = Field(default=None, max_length=512)
    conference_account_code: Optional[str] = Field(default=None, max_length=64)
    dialplan_uuid: Optional[UUID] = Field(default=None, foreign_key="dialplans.dialplan_uuid", index=True, ondelete="SET NULL")

    @field_validator("conference_extension")
    @classmethod
    def validate_extension(cls, v: str) -> str:
        if v is None:
            return v
        s = str(v).strip()
        if not re.match(r"^\d{2,7}$", s):
            raise ValueError("conference_extension must be 2-7 digits")
        return s

    @field_validator("conference_name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        if v is None:
            return v
        s = str(v).strip()
        if len(s) == 0:
            raise ValueError("conference_name must not be empty")
        return s

    @field_validator("conference_profile")
    @classmethod
    def validate_profile(cls, v: str) -> str:
        if v is None or str(v).strip() == "":
            return "default"
        return str(v).strip()

    # modern aliases as properties for drop-in convenience
    @property
    def name(self) -> str:
        return self.conference_name

    @name.setter
    def name(self, value: str):
        self.conference_name = value

    @property
    def extension(self) -> str:
        return self.conference_extension

    @extension.setter
    def extension(self, value: str):
        self.conference_extension = value

    @property
    def pin_number(self) -> Optional[str]:
        return self.conference_pin_number

    @pin_number.setter
    def pin_number(self, value: Optional[str]):
        self.conference_pin_number = value

    @property
    def profile(self) -> str:
        return self.conference_profile

    @profile.setter
    def profile(self, value: str):
        self.conference_profile = value

    @property
    def enabled(self) -> bool:
        return self.conference_enabled

    @enabled.setter
    def enabled(self, value: bool):
        self.conference_enabled = value

    @property
    def description(self) -> Optional[str]:
        return self.conference_description

    @description.setter
    def description(self, value: Optional[str]):
        self.conference_description = value


class Conference(ConferenceBase, table=True):
    __tablename__ = "conferences"
    conference_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ConferenceUserBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_uuid: UUID = Field(foreign_key="conferences.conference_uuid", index=True, ondelete="CASCADE")
    user_uuid: UUID = Field(foreign_key="users.user_uuid", index=True, ondelete="CASCADE")


class ConferenceUser(ConferenceUserBase, table=True):
    __tablename__ = "conference_users"
    conference_user_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


# Alias for task spec: ConferenceMember
ConferenceMemberBase = ConferenceUserBase
ConferenceMember = ConferenceUser
