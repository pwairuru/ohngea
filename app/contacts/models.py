from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class ContactBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    name: str = Field(max_length=128)
    number: str = Field(max_length=32, index=True)
    email: Optional[str] = Field(default=None, max_length=255)
    organization: Optional[str] = Field(default=None, max_length=128)
    # legacy FusionPBX fields (nullable for parity with v_contacts)
    contact_type: Optional[str] = Field(default=None, max_length=32)
    contact_parent_uuid: Optional[UUID] = Field(default=None, foreign_key="contacts.contact_uuid", index=True, ondelete="SET NULL")
    contact_name_prefix: Optional[str] = Field(default=None, max_length=64)
    contact_name_given: Optional[str] = Field(default=None, max_length=128)
    contact_name_middle: Optional[str] = Field(default=None, max_length=128)
    contact_name_family: Optional[str] = Field(default=None, max_length=128)
    contact_name_suffix: Optional[str] = Field(default=None, max_length=64)
    contact_nickname: Optional[str] = Field(default=None, max_length=128)
    contact_title: Optional[str] = Field(default=None, max_length=128)
    contact_role: Optional[str] = Field(default=None, max_length=128)
    contact_category: Optional[str] = Field(default=None, max_length=64)
    contact_url: Optional[str] = Field(default=None, max_length=512)
    contact_time_zone: Optional[str] = Field(default=None, max_length=64)
    contact_note: Optional[str] = Field(default=None, max_length=1024)


class Contact(ContactBase, table=True):
    __tablename__ = "contacts"
    contact_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


# --- child tables ---

class ContactAddressBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    address_type: Optional[str] = Field(default=None, max_length=32)
    address_label: Optional[str] = Field(default=None, max_length=64)
    address_primary: bool = Field(default=False)
    address_street: Optional[str] = Field(default=None, max_length=255)
    address_extended: Optional[str] = Field(default=None, max_length=255)
    address_community: Optional[str] = Field(default=None, max_length=128)
    address_locality: Optional[str] = Field(default=None, max_length=128)  # city
    address_region: Optional[str] = Field(default=None, max_length=128)
    address_postal_code: Optional[str] = Field(default=None, max_length=32)
    address_country: Optional[str] = Field(default=None, max_length=64)
    address_description: Optional[str] = Field(default=None, max_length=512)


class ContactAddress(ContactAddressBase, table=True):
    __tablename__ = "contact_addresses"
    contact_address_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactPhoneBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    phone_label: Optional[str] = Field(default=None, max_length=64)
    phone_number: str = Field(max_length=32)
    phone_extension: Optional[str] = Field(default=None, max_length=32)
    phone_type: Optional[str] = Field(default=None, max_length=32)  # voice/fax/video/text
    phone_primary: bool = Field(default=False)
    phone_description: Optional[str] = Field(default=None, max_length=512)


class ContactPhone(ContactPhoneBase, table=True):
    __tablename__ = "contact_phones"
    contact_phone_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactEmailBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    email_label: Optional[str] = Field(default=None, max_length=64)
    email_address: str = Field(max_length=255)
    email_primary: bool = Field(default=False)
    email_description: Optional[str] = Field(default=None, max_length=512)


class ContactEmail(ContactEmailBase, table=True):
    __tablename__ = "contact_emails"
    contact_email_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactUrlBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    url_type: Optional[str] = Field(default=None, max_length=32)
    url_label: Optional[str] = Field(default=None, max_length=64)
    url_address: str = Field(max_length=512)
    url_primary: bool = Field(default=False)
    url_description: Optional[str] = Field(default=None, max_length=512)


class ContactUrl(ContactUrlBase, table=True):
    __tablename__ = "contact_urls"
    contact_url_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactNoteBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    contact_note: str = Field(max_length=1024)


class ContactNote(ContactNoteBase, table=True):
    __tablename__ = "contact_notes"
    contact_note_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactGroupBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    group_uuid: UUID = Field(foreign_key="groups.group_uuid", index=True, ondelete="CASCADE")


class ContactGroup(ContactGroupBase, table=True):
    __tablename__ = "contact_groups"
    contact_group_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactUserBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    user_uuid: UUID = Field(foreign_key="users.user_uuid", index=True, ondelete="CASCADE")


class ContactUser(ContactUserBase, table=True):
    __tablename__ = "contact_users"
    contact_user_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactSettingBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    contact_setting_category: Optional[str] = Field(default=None, max_length=64)
    contact_setting_subcategory: Optional[str] = Field(default=None, max_length=64)
    contact_setting_name: str = Field(max_length=128)
    contact_setting_value: Optional[str] = Field(default=None, max_length=512)
    contact_setting_order: Optional[int] = Field(default=0)
    contact_setting_enabled: bool = Field(default=True)
    contact_setting_description: Optional[str] = Field(default=None, max_length=512)


class ContactSetting(ContactSettingBase, table=True):
    __tablename__ = "contact_settings"
    contact_setting_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactRelationBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    relation_label: Optional[str] = Field(default=None, max_length=64)
    relation_contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")


class ContactRelation(ContactRelationBase, table=True):
    __tablename__ = "contact_relations"
    contact_relation_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactTimeBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    user_uuid: Optional[UUID] = Field(default=None, foreign_key="users.user_uuid", index=True, ondelete="SET NULL")
    time_start: Optional[datetime] = Field(default=None)
    time_stop: Optional[datetime] = Field(default=None)
    time_description: Optional[str] = Field(default=None, max_length=512)


class ContactTime(ContactTimeBase, table=True):
    __tablename__ = "contact_times"
    contact_time_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class ContactAttachmentBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    contact_uuid: UUID = Field(foreign_key="contacts.contact_uuid", index=True, ondelete="CASCADE")
    attachment_primary: bool = Field(default=False)
    attachment_filename: Optional[str] = Field(default=None, max_length=255)
    attachment_content: Optional[str] = Field(default=None)
    attachment_description: Optional[str] = Field(default=None, max_length=512)


class ContactAttachment(ContactAttachmentBase, table=True):
    __tablename__ = "contact_attachments"
    contact_attachment_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
