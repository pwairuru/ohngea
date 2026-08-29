from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DeviceBase(SQLModel):
    mac: str = Field(max_length=17, regex=r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$", description="MAC 00:11:22:33:44:55")
    vendor: str = Field(max_length=32, description="poly, yealink, etc.")
    model: Optional[str] = Field(default=None, max_length=64)
    label: Optional[str] = Field(default=None, max_length=128)
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    extension_uuid: Optional[UUID] = Field(default=None, foreign_key="extensions.extension_uuid")
    device_profile_uuid: Optional[UUID] = Field(default=None, foreign_key="device_profiles.device_profile_uuid", index=True, ondelete="SET NULL")
    # full parity hidden
    device_address: Optional[str] = Field(default=None, max_length=32, description="alias for mac, legacy device_address")
    device_label: Optional[str] = Field(default=None, max_length=128)
    device_vendor: Optional[str] = Field(default=None, max_length=32)
    device_model: Optional[str] = Field(default=None, max_length=64)
    device_firmware_version: Optional[str] = Field(default=None, max_length=64)
    device_template: Optional[str] = Field(default=None, max_length=128)
    device_location: Optional[str] = Field(default=None, max_length=128)
    device_serial_number: Optional[str] = Field(default=None, max_length=128)
    device_description: Optional[str] = Field(default=None, max_length=512)
    device_username: Optional[str] = Field(default=None, max_length=64)
    device_password: Optional[str] = Field(default=None, max_length=64)
    device_provisioned_date: Optional[datetime] = Field(default=None)
    device_provisioned_ip: Optional[str] = Field(default=None, max_length=64)
    device_provisioned_method: Optional[str] = Field(default=None, max_length=32)


class Device(DeviceBase, table=True):
    __tablename__ = "devices"
    device_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class DeviceLineBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    device_uuid: UUID = Field(foreign_key="devices.device_uuid", index=True, ondelete="CASCADE")
    line_number: str = Field(default="1", max_length=8)
    server_address: Optional[str] = Field(default=None, max_length=128)
    server_address_primary: Optional[str] = Field(default=None, max_length=128)
    server_address_secondary: Optional[str] = Field(default=None, max_length=128)
    outbound_proxy_primary: Optional[str] = Field(default=None, max_length=128)
    outbound_proxy_secondary: Optional[str] = Field(default=None, max_length=128)
    label: Optional[str] = Field(default=None, max_length=128)
    display_name: Optional[str] = Field(default=None, max_length=128)
    user_id: Optional[str] = Field(default=None, max_length=64)
    auth_id: Optional[str] = Field(default=None, max_length=64)
    password: Optional[str] = Field(default=None, max_length=64)
    sip_port: Optional[int] = Field(default=None)
    sip_transport: Optional[str] = Field(default=None, max_length=32)
    register_expires: Optional[int] = Field(default=None)
    shared_line: Optional[str] = Field(default=None, max_length=32)
    enabled: bool = Field(default=True)


class DeviceLine(DeviceLineBase, table=True):
    __tablename__ = "device_lines"
    device_line_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())


# --- device_settings ---
class DeviceSettingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    device_uuid: UUID = Field(foreign_key="devices.device_uuid", index=True, ondelete="CASCADE")
    device_profile_uuid: Optional[UUID] = Field(default=None, foreign_key="device_profiles.device_profile_uuid", ondelete="SET NULL", index=True)
    device_setting_category: Optional[str] = Field(default=None, max_length=64)
    device_setting_subcategory: Optional[str] = Field(default=None, max_length=64)
    device_setting_name: str = Field(max_length=128)
    device_setting_value: Optional[str] = Field(default=None, max_length=512)
    device_setting_enabled: bool = Field(default=True)
    device_setting_description: Optional[str] = Field(default=None, max_length=512)


class DeviceSetting(DeviceSettingBase, table=True):
    __tablename__ = "device_settings"
    device_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


# --- device_keys ---
class DeviceKeyBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    device_uuid: UUID = Field(foreign_key="devices.device_uuid", index=True, ondelete="CASCADE")
    device_profile_uuid: Optional[UUID] = Field(default=None, foreign_key="device_profiles.device_profile_uuid", ondelete="SET NULL", index=True)
    device_key_id: Optional[int] = Field(default=None)
    device_key_category: Optional[str] = Field(default=None, max_length=64)
    device_key_vendor: Optional[str] = Field(default=None, max_length=64)
    device_key_type: Optional[str] = Field(default=None, max_length=64)
    device_key_subtype: Optional[str] = Field(default=None, max_length=64)
    device_key_line: Optional[int] = Field(default=None)
    device_key_value: Optional[str] = Field(default=None, max_length=128)
    device_key_extension: Optional[str] = Field(default=None, max_length=64)
    device_key_protected: Optional[str] = Field(default=None, max_length=16)
    device_key_label: Optional[str] = Field(default=None, max_length=128)
    device_key_icon: Optional[str] = Field(default=None, max_length=64)


class DeviceKey(DeviceKeyBase, table=True):
    __tablename__ = "device_keys"
    device_key_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


# --- device_profiles ---
class DeviceProfileBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    device_profile_name: str = Field(max_length=128)
    device_profile_enabled: bool = Field(default=True)
    device_profile_description: Optional[str] = Field(default=None, max_length=512)


class DeviceProfile(DeviceProfileBase, table=True):
    __tablename__ = "device_profiles"
    device_profile_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


# --- device_profile_keys ---
class DeviceProfileKeyBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    device_profile_uuid: UUID = Field(foreign_key="device_profiles.device_profile_uuid", index=True, ondelete="CASCADE")
    profile_key_id: Optional[int] = Field(default=None)
    profile_key_category: Optional[str] = Field(default=None, max_length=64)
    profile_key_vendor: Optional[str] = Field(default=None, max_length=64)
    profile_key_type: Optional[str] = Field(default=None, max_length=64)
    profile_key_subtype: Optional[str] = Field(default=None, max_length=64)
    profile_key_line: Optional[int] = Field(default=None)
    profile_key_value: Optional[str] = Field(default=None, max_length=128)
    profile_key_extension: Optional[str] = Field(default=None, max_length=64)
    profile_key_protected: bool = Field(default=False)
    profile_key_label: Optional[str] = Field(default=None, max_length=128)
    profile_key_icon: Optional[str] = Field(default=None, max_length=64)


class DeviceProfileKey(DeviceProfileKeyBase, table=True):
    __tablename__ = "device_profile_keys"
    device_profile_key_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


# --- device_profile_settings ---
class DeviceProfileSettingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    device_profile_uuid: UUID = Field(foreign_key="device_profiles.device_profile_uuid", index=True, ondelete="CASCADE")
    profile_setting_name: str = Field(max_length=128)
    profile_setting_value: Optional[str] = Field(default=None, max_length=512)
    profile_setting_enabled: bool = Field(default=True)
    profile_setting_description: Optional[str] = Field(default=None, max_length=512)


class DeviceProfileSetting(DeviceProfileSettingBase, table=True):
    __tablename__ = "device_profile_settings"
    device_profile_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
