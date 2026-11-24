from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DestinationBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    destination_type: str = Field(max_length=32, description="inbound, outbound, etc")
    destination_number: str = Field(max_length=32, description="DID number")
    destination_prefix: Optional[str] = Field(default=None, max_length=32)
    destination_enabled: bool = Field(default=True)
    destination_description: Optional[str] = Field(default=None, max_length=512)
    destination_context: Optional[str] = Field(default="public", max_length=128)
    destination_app: Optional[str] = Field(default=None, max_length=64)
    destination_data: Optional[str] = Field(default=None, max_length=512)
    destination_caller_id_name: Optional[str] = Field(default=None, max_length=128)
    destination_caller_id_number: Optional[str] = Field(default=None, max_length=32)
    destination_hold_music: Optional[str] = Field(default=None, max_length=128)
    destination_record: Optional[bool] = Field(default=False)
    # --- legacy parity fields (nullable) ---
    dialplan_uuid: Optional[UUID] = Field(default=None, foreign_key="dialplans.dialplan_uuid", index=True)
    fax_uuid: Optional[UUID] = Field(default=None, index=True)
    user_uuid: Optional[UUID] = Field(default=None, index=True)
    group_uuid: Optional[UUID] = Field(default=None, index=True)
    provider_uuid: Optional[UUID] = Field(default=None, index=True)
    destination_trunk_prefix: Optional[str] = Field(default=None, max_length=32)
    trunk_prefix: Optional[str] = Field(default=None, max_length=32)
    destination_area_code: Optional[str] = Field(default=None, max_length=16)
    area_code: Optional[str] = Field(default=None, max_length=16)
    destination_condition_field: Optional[str] = Field(default=None, max_length=128)
    condition_field: Optional[str] = Field(default=None, max_length=128)
    destination_number_regex: Optional[str] = Field(default=None, max_length=512)
    number_regex: Optional[str] = Field(default=None, max_length=512)
    destination_distinctive_ring: Optional[str] = Field(default=None, max_length=128)
    distinctive_ring: Optional[str] = Field(default=None, max_length=128)
    destination_ringback: Optional[str] = Field(default=None, max_length=128)
    ringback: Optional[str] = Field(default=None, max_length=128)
    destination_accountcode: Optional[str] = Field(default=None, max_length=64)
    accountcode: Optional[str] = Field(default=None, max_length=64)
    destination_type_voice: Optional[bool] = Field(default=None)
    type_voice: Optional[bool] = Field(default=None)
    destination_type_fax: Optional[bool] = Field(default=None)
    type_fax: Optional[bool] = Field(default=None)
    destination_type_emergency: Optional[bool] = Field(default=None)
    type_emergency: Optional[bool] = Field(default=None)
    destination_type_text: Optional[bool] = Field(default=None)
    type_text: Optional[bool] = Field(default=None)
    destination_conditions: Optional[str] = Field(default=None, max_length=4096)
    conditions: Optional[str] = Field(default=None, max_length=4096)
    destination_actions: Optional[str] = Field(default=None, max_length=4096)
    actions: Optional[str] = Field(default=None, max_length=4096)
    destination_alternate_app: Optional[str] = Field(default=None, max_length=64)
    alternate_app: Optional[str] = Field(default=None, max_length=64)
    destination_alternate_data: Optional[str] = Field(default=None, max_length=512)
    alternate_data: Optional[str] = Field(default=None, max_length=512)
    destination_caller_id: Optional[str] = Field(default=None, max_length=64)
    destination_insert_user: Optional[UUID] = Field(default=None)
    insert_user: Optional[UUID] = Field(default=None)
    destination_insert_date: Optional[datetime] = Field(default=None)
    destination_update_user: Optional[UUID] = Field(default=None)


class Destination(DestinationBase, table=True):
    __tablename__ = "destinations"
    destination_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
