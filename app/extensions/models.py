import re
from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlmodel import Field, SQLModel

from app.db import uuid7


class ExtensionBase(SQLModel):
    extension: str = Field(description="Extension number 2-7 digits")

    @field_validator("extension")
    @classmethod
    def validate_extension(cls, v: str) -> str:
        if not re.match(r"^\d{2,7}$", str(v)):
            raise ValueError("extension must be 2-7 digits")
        return str(v)

    number_alias: Optional[str] = Field(default=None, max_length=32)
    password: Optional[str] = Field(default=None, max_length=64, description="SIP password")
    accountcode: Optional[str] = Field(default=None, max_length=64)
    effective_caller_id_name: str = Field(max_length=128)
    effective_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    outbound_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    outbound_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    emergency_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    emergency_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    directory_first_name: Optional[str] = Field(default=None, max_length=128)
    directory_last_name: Optional[str] = Field(default=None, max_length=128)
    directory_visible: bool = Field(default=False)
    directory_exten_visible: bool = Field(default=False)
    max_registrations: Optional[str] = Field(default=None, max_length=32)
    limit_max: Optional[str] = Field(default="5", max_length=32)
    limit_destination: Optional[str] = Field(default="!USER_BUSY", max_length=128)
    missed_call_app: Optional[str] = Field(default=None, max_length=32)
    missed_call_data: Optional[str] = Field(default=None, max_length=128)
    user_context: Optional[str] = Field(default=None, max_length=128)
    toll_allow: Optional[str] = Field(default=None, max_length=128)
    call_timeout: Optional[int] = Field(default=30)
    call_group: Optional[str] = Field(default=None, max_length=32)
    call_screen_enabled: bool = Field(default=False)
    user_record: Optional[str] = Field(default=None, max_length=32)
    hold_music: Optional[str] = Field(default=None, max_length=128)
    auth_acl: Optional[str] = Field(default=None, max_length=128)
    cidr: Optional[str] = Field(default=None, max_length=512)
    sip_force_contact: Optional[str] = Field(default=None, max_length=128)
    sip_force_expires: Optional[int] = Field(default=None)
    nibble_account: Optional[str] = Field(default=None, max_length=128)
    mwi_account: Optional[str] = Field(default=None, max_length=128)
    sip_bypass_media: Optional[str] = Field(default=None, max_length=32)
    absolute_codec_string: Optional[str] = Field(default=None, max_length=256)
    force_ping: bool = Field(default=False)
    dial_string: Optional[str] = Field(default=None, max_length=1024)
    do_not_disturb: bool = Field(default=False)
    forward_all_destination: Optional[str] = Field(default=None, max_length=128)
    forward_all_enabled: bool = Field(default=False)
    forward_busy_destination: Optional[str] = Field(default=None, max_length=128)
    forward_busy_enabled: bool = Field(default=False)
    forward_no_answer_destination: Optional[str] = Field(default=None, max_length=128)
    forward_no_answer_enabled: bool = Field(default=False)
    forward_user_not_registered_destination: Optional[str] = Field(default=None, max_length=128)
    forward_user_not_registered_enabled: bool = Field(default=False)
    follow_me_uuid: Optional[UUID] = Field(default=None)
    follow_me_enabled: bool = Field(default=False)
    extension_language: Optional[str] = Field(default=None, max_length=32)
    extension_dialect: Optional[str] = Field(default=None, max_length=32)
    extension_voice: Optional[str] = Field(default=None, max_length=32)
    extension_type: Optional[str] = Field(default="default", max_length=32)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")


class Extension(ExtensionBase, table=True):
    __tablename__ = "extensions"
    extension_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
