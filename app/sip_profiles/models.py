from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class SipProfileBase(SQLModel):
    name: str = Field(max_length=64)
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    sip_port: int = Field(default=5060)
    sip_ip: Optional[str] = Field(default=None, max_length=64)
    ext_sip_ip: Optional[str] = Field(default=None, max_length=64)
    ext_rtp_ip: Optional[str] = Field(default=None, max_length=64)
    realm: Optional[str] = Field(default=None, max_length=128)
    context: Optional[str] = Field(default=None, max_length=128)
    challenge_realm: Optional[str] = Field(default=None, max_length=128)
    wss_port: Optional[int] = Field(default=None)
    wss_binding: Optional[str] = Field(default=None, max_length=128)
    tls_cert_path: Optional[str] = Field(default=None, max_length=256)
    tls_key_path: Optional[str] = Field(default=None, max_length=256)
    # full parity
    sip_secure: Optional[bool] = Field(default=None)
    rtp_ip: Optional[str] = Field(default=None, max_length=64)
    rtp_port_range: Optional[str] = Field(default=None, max_length=32)
    hold_music: Optional[str] = Field(default=None, max_length=128)
    apply_inbound_acl: Optional[str] = Field(default=None, max_length=128)
    local_network_acl: Optional[str] = Field(default=None, max_length=128)
    record_path: Optional[str] = Field(default=None, max_length=256)
    record_template: Optional[str] = Field(default=None, max_length=128)
    description: Optional[str] = Field(default=None, max_length=512)
    # legacy compat alias fields
    hostname: Optional[str] = Field(default=None, max_length=128, description="sip_profile_hostname legacy")


class SipProfile(SipProfileBase, table=True):
    __tablename__ = "sip_profiles"
    sip_profile_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class SipProfileDomainBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    sip_profile_uuid: UUID = Field(foreign_key="sip_profiles.sip_profile_uuid", index=True, ondelete="CASCADE")
    domain_name: str = Field(max_length=128, description="sip_profile_domain_name legacy")
    domain_alias: Optional[str] = Field(default="false", max_length=16, description="alias true/false")
    domain_parse: Optional[str] = Field(default="true", max_length=16, description="parse true/false")


class SipProfileDomain(SipProfileDomainBase, table=True):
    __tablename__ = "sip_profile_domains"
    sip_profile_domain_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class SipProfileSettingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    sip_profile_uuid: UUID = Field(foreign_key="sip_profiles.sip_profile_uuid", index=True, ondelete="CASCADE")
    setting_name: str = Field(max_length=128, description="sip_profile_setting_name legacy")
    setting_value: Optional[str] = Field(default=None, max_length=512, description="sip_profile_setting_value legacy")
    enabled: bool = Field(default=True, description="sip_profile_setting_enabled legacy")
    description: Optional[str] = Field(default=None, max_length=512, description="sip_profile_setting_description legacy")


class SipProfileSetting(SipProfileSettingBase, table=True):
    __tablename__ = "sip_profile_settings"
    sip_profile_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())
