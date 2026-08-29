import re
from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import field_validator
from sqlmodel import Field, SQLModel

from app.db import uuid7


class FaxBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    dialplan_uuid: Optional[UUID] = Field(default=None, foreign_key="dialplans.dialplan_uuid", description="Dialplan UUID")
    fax_extension: str = Field(max_length=32, description="Fax extension 2-7 digits")
    fax_name: str = Field(max_length=128, description="Fax name")
    fax_destination_number: Optional[str] = Field(default=None, max_length=32, description="Destination number for inbound")
    fax_prefix: Optional[str] = Field(default=None, max_length=32, description="Prefix for outbound")
    fax_email: Optional[str] = Field(default=None, max_length=255)
    fax_pin_number: Optional[str] = Field(default=None, max_length=32)
    fax_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    fax_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    fax_forward_number: Optional[str] = Field(default=None, max_length=32)
    fax_toll_allow: Optional[str] = Field(default=None, max_length=32)
    fax_email_connection_type: Optional[str] = Field(default=None, max_length=32, description="IMAP or POP3")
    fax_email_connection_host: Optional[str] = Field(default=None, max_length=255)
    fax_email_connection_port: Optional[str] = Field(default=None, max_length=32)
    fax_email_connection_security: Optional[str] = Field(default=None, max_length=32, description="SSL or TLS")
    fax_email_connection_validate: Optional[bool] = Field(default=None, description="Validate certificate")
    fax_email_connection_username: Optional[str] = Field(default=None, max_length=255)
    fax_email_connection_password: Optional[str] = Field(default=None, max_length=255)
    fax_email_connection_mailbox: Optional[str] = Field(default=None, max_length=128)
    fax_email_inbound_subject_tag: Optional[str] = Field(default=None, max_length=128)
    fax_email_outbound_subject_tag: Optional[str] = Field(default=None, max_length=128)
    fax_email_outbound_authorized_senders: Optional[str] = Field(default=None, max_length=512)
    fax_send_channels: Optional[int] = Field(default=None, description="Number of send channels")
    accountcode: Optional[str] = Field(default=None, max_length=128)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)

    @field_validator("fax_extension")
    @classmethod
    def validate_ext(cls, v: str) -> str:
        if not re.match(r"^\d{2,7}$", v):
            raise ValueError("fax_extension must be 2-7 digits")
        return v


class Fax(FaxBase, table=True):
    __tablename__ = "fax"
    fax_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class FaxUser(SQLModel, table=True):
    __tablename__ = "fax_users"
    fax_user_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fax_uuid: UUID = Field(foreign_key="fax.fax_uuid", index=True, ondelete="CASCADE")
    user_uuid: UUID = Field(foreign_key="users.user_uuid", index=True, ondelete="CASCADE")
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class FaxFile(SQLModel, table=True):
    __tablename__ = "fax_files"
    fax_file_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fax_uuid: UUID = Field(foreign_key="fax.fax_uuid", index=True, ondelete="CASCADE")
    file_name: Optional[str] = Field(default=None, max_length=255, description="Original file name")
    file_path: Optional[str] = Field(default=None, max_length=512, description="Storage path")
    file_size: Optional[int] = Field(default=None, description="File size bytes")
    # legacy compat aliases
    fax_file_path: Optional[str] = Field(default=None, max_length=512)
    fax_file_type: Optional[str] = Field(default=None, max_length=32)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class FaxLog(SQLModel, table=True):
    __tablename__ = "fax_logs"
    fax_log_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    fax_uuid: UUID = Field(foreign_key="fax.fax_uuid", index=True, ondelete="CASCADE")
    fax_success: Optional[str] = Field(default=None, max_length=32, description="Success flag 1/0 or true/false")
    result_code: Optional[int] = Field(default=None, description="Result code numeric")
    fax_result_text: Optional[str] = Field(default=None, max_length=512)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
