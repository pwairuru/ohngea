from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class ConferenceCenterBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_center_name: str = Field(max_length=128)
    conference_center_extension: Optional[str]=Field(default=None, max_length=32)
    conference_center_pin_length: Optional[int]=Field(default=6)
    conference_center_greeting: Optional[str]=Field(default=None, max_length=512)
    conference_center_description: Optional[str]=Field(default=None, max_length=512)
    conference_center_enabled: bool = Field(default=True)
    dialplan_uuid: Optional[UUID]=Field(default=None)
class ConferenceCenter(ConferenceCenterBase, table=True):
    __tablename__="conference_centers"
    conference_center_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class ConferenceRoomBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_center_uuid: UUID = Field(foreign_key="conference_centers.conference_center_uuid", index=True, ondelete="CASCADE")
    conference_room_name: str = Field(max_length=128, description="Room name")
    room_name: Optional[str] = Field(default=None, max_length=128, description="Alias for conference_room_name")
    room_pin: Optional[str] = Field(default=None, max_length=32, description="Room pin alias")
    participant_pin: Optional[str] = Field(default=None, max_length=32)
    moderator_pin: Optional[str] = Field(default=None, max_length=32)
    max_members: Optional[int] = Field(default=None)
    record: Optional[bool] = Field(default=False)
    profile: Optional[str] = Field(default=None, max_length=32)
    wait_mod: Optional[bool] = Field(default=False)
    moderator_endconf: Optional[bool] = Field(default=False)
    announce_name: Optional[bool] = Field(default=False)
    announce_count: Optional[bool] = Field(default=False)
    announce_recording: Optional[bool] = Field(default=False)
    sounds: Optional[bool] = Field(default=True)
    mute: Optional[bool] = Field(default=False)
    enabled: bool = Field(default=True)
    description: Optional[str] = Field(default=None, max_length=512)
    email_address: Optional[str] = Field(default=None, max_length=255)
    account_code: Optional[str] = Field(default=None, max_length=128)
    start_datetime: Optional[str] = Field(default=None, max_length=64)
    stop_datetime: Optional[str] = Field(default=None, max_length=64)


class ConferenceRoom(ConferenceRoomBase, table=True):
    __tablename__ = "conference_rooms"
    conference_room_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class ConferenceRoomUser(SQLModel, table=True):
    __tablename__ = "conference_room_users"
    conference_room_user_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_room_uuid: UUID = Field(foreign_key="conference_rooms.conference_room_uuid", index=True, ondelete="CASCADE")
    user_uuid: UUID = Field(foreign_key="users.user_uuid", index=True, ondelete="CASCADE")
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class ConferenceSessionBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_room_uuid: Optional[UUID] = Field(default=None, foreign_key="conference_rooms.conference_room_uuid", index=True, ondelete="CASCADE")
    conference_center_uuid: Optional[UUID] = Field(default=None, foreign_key="conference_centers.conference_center_uuid", index=True, ondelete="CASCADE")
    profile: Optional[str] = Field(default=None, max_length=64)
    recording: Optional[str] = Field(default=None, max_length=512)
    start_epoch: Optional[int] = Field(default=None)
    end_epoch: Optional[int] = Field(default=None)


class ConferenceSession(ConferenceSessionBase, table=True):
    __tablename__ = "conference_sessions"
    conference_session_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class ConferenceSessionDetailBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    conference_session_uuid: UUID = Field(foreign_key="conference_sessions.conference_session_uuid", index=True, ondelete="CASCADE")
    username: Optional[str] = Field(default=None, max_length=32)
    caller_id_name: Optional[str] = Field(default=None, max_length=128)
    caller_id_number: Optional[str] = Field(default=None, max_length=32)
    uuid: Optional[UUID] = Field(default=None, description="Call UUID")
    moderator: Optional[str] = Field(default=None, max_length=8)
    network_addr: Optional[str] = Field(default=None, max_length=64)
    start_epoch: Optional[int] = Field(default=None)
    end_epoch: Optional[int] = Field(default=None)


class ConferenceSessionDetail(ConferenceSessionDetailBase, table=True):
    __tablename__ = "conference_session_details"
    conference_session_detail_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
