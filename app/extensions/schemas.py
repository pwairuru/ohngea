from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel

from app.extensions.models import ExtensionBase
from typing import Optional as _Optional
from uuid import UUID as _UUID


class ExtensionCreate(ExtensionBase):
    # allow missing domain_id — router auto-fills from JWT
    domain_id: _Optional[_UUID] = None  # type: ignore[assignment]
    # password optional — service generates if missing
    password: _Optional[str] = None  # type: ignore[assignment]


class ExtensionUpdate(BaseModel):
    extension: Optional[str] = None
    number_alias: Optional[str] = None
    password: Optional[str] = None
    accountcode: Optional[str] = None
    effective_caller_id_name: Optional[str] = None
    effective_caller_id_number: Optional[str] = None
    outbound_caller_id_name: Optional[str] = None
    outbound_caller_id_number: Optional[str] = None
    emergency_caller_id_name: Optional[str] = None
    emergency_caller_id_number: Optional[str] = None
    directory_first_name: Optional[str] = None
    directory_last_name: Optional[str] = None
    directory_visible: Optional[bool] = None
    directory_exten_visible: Optional[bool] = None
    max_registrations: Optional[str] = None
    limit_max: Optional[str] = None
    limit_destination: Optional[str] = None
    missed_call_app: Optional[str] = None
    missed_call_data: Optional[str] = None
    user_context: Optional[str] = None
    toll_allow: Optional[str] = None
    call_timeout: Optional[int] = None
    call_group: Optional[str] = None
    call_screen_enabled: Optional[bool] = None
    user_record: Optional[str] = None
    hold_music: Optional[str] = None
    auth_acl: Optional[str] = None
    cidr: Optional[str] = None
    sip_force_contact: Optional[str] = None
    sip_force_expires: Optional[int] = None
    nibble_account: Optional[str] = None
    mwi_account: Optional[str] = None
    sip_bypass_media: Optional[str] = None
    absolute_codec_string: Optional[str] = None
    force_ping: Optional[bool] = None
    dial_string: Optional[str] = None
    do_not_disturb: Optional[bool] = None
    forward_all_destination: Optional[str] = None
    forward_all_enabled: Optional[bool] = None
    forward_busy_destination: Optional[str] = None
    forward_busy_enabled: Optional[bool] = None
    forward_no_answer_destination: Optional[str] = None
    forward_no_answer_enabled: Optional[bool] = None
    forward_user_not_registered_destination: Optional[str] = None
    forward_user_not_registered_enabled: Optional[bool] = None
    follow_me_uuid: Optional[UUID] = None
    follow_me_enabled: Optional[bool] = None
    extension_language: Optional[str] = None
    extension_dialect: Optional[str] = None
    extension_voice: Optional[str] = None
    extension_type: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None


class ExtensionRead(ExtensionBase):
    extension_uuid: UUID
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True
