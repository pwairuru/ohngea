from datetime import datetime
from typing import Optional, Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, field_validator, model_validator
import re


class ConferenceCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)

    domain_id: Optional[UUID] = None
    conference_name: Optional[str] = None
    conference_extension: Optional[str] = None
    conference_pin_number: Optional[str] = None
    conference_profile: Optional[str] = "default"
    conference_enabled: Optional[bool] = True
    conference_description: Optional[str] = None
    conference_flags: Optional[str] = None
    conference_order: Optional[int] = None
    conference_context: Optional[str] = None
    conference_email_address: Optional[str] = None
    conference_account_code: Optional[str] = None
    dialplan_uuid: Optional[UUID] = None

    # also accept modern simple aliases via validator
    @model_validator(mode="before")
    @classmethod
    def map_legacy(cls, data: Any):
        if not isinstance(data, dict):
            return data
        d = dict(data)
        # domain alias
        if not d.get("domain_id"):
            for k in ("domain_uuid", "domainId", "domain", "domain_name"):
                if d.get(k) not in (None, ""):
                    try:
                        d["domain_id"] = d.pop(k)
                    except Exception:
                        d["domain_id"] = d.pop(k)
                    break
                elif k in d and d[k] in (None, ""):
                    d.pop(k, None)
        # conference_name aliases
        if not d.get("conference_name"):
            for k in ("name", "conferenceName", "conference_name_text", "label", "extension_name"):
                if d.get(k) not in (None, ""):
                    d["conference_name"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    if d[k] == "" or d[k] is None:
                        d.pop(k, None)
        # conference_extension aliases
        if not d.get("conference_extension"):
            for k in ("extension", "conferenceExtension", "destination", "conference_number", "number", "dialplan_number", "extension_number"):
                if d.get(k) not in (None, ""):
                    d["conference_extension"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    pass
        # conference_pin_number aliases
        if not d.get("conference_pin_number") and d.get("conference_pin_number") != "":
            for k in ("pin_number", "pinNumber", "pin", "conferencePinNumber", "password", "conference_password", "participant_pin", "moderator_pin"):
                if d.get(k) not in (None, ""):
                    d["conference_pin_number"] = str(d.pop(k)).strip()
                    break
                elif k in d and d[k] in (None, ""):
                    d.pop(k, None)
        # conference_profile aliases (wideband)
        if not d.get("conference_profile") or d.get("conference_profile") in (None, ""):
            for k in ("profile", "conferenceProfile", "conference_profile_name", "profile_name"):
                if d.get(k) not in (None, ""):
                    d["conference_profile"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    d.pop(k, None)
            if not d.get("conference_profile"):
                d["conference_profile"] = "default"
        # conference_enabled aliases
        if "conference_enabled" not in d or d["conference_enabled"] is None:
            for k in ("enabled", "conferenceEnabled", "is_enabled", "isEnabled", "active", "conference_active"):
                if k in d and d[k] is not None:
                    val = d.pop(k)
                    if isinstance(val, str):
                        d["conference_enabled"] = val.lower() in ("true", "1", "yes", "t")
                    else:
                        d["conference_enabled"] = bool(val)
                    break
        else:
            if isinstance(d.get("conference_enabled"), str):
                d["conference_enabled"] = d["conference_enabled"].lower() in ("true", "1", "yes", "t")
        # description aliases
        if not d.get("conference_description"):
            for k in ("description", "conferenceDescription", "descr", "conference_descr"):
                if d.get(k) not in (None, ""):
                    d["conference_description"] = str(d.pop(k))
                    break
        # flags
        if not d.get("conference_flags"):
            for k in ("flags", "conferenceFlags", "conference_flags_text"):
                if d.get(k) not in (None, ""):
                    d["conference_flags"] = str(d.pop(k))
                    break
        # order
        if d.get("conference_order") is None:
            for k in ("order", "conferenceOrder", "conference_order_number"):
                if k in d and d[k] not in (None, ""):
                    try:
                        d["conference_order"] = int(d.pop(k))
                    except Exception:
                        d["conference_order"] = d.pop(k)
                    break
        # context
        if not d.get("conference_context"):
            for k in ("context", "conferenceContext", "dialplan_context"):
                if d.get(k) not in (None, ""):
                    d["conference_context"] = str(d.pop(k))
                    break
        # email
        if not d.get("conference_email_address"):
            for k in ("email_address", "emailAddress", "email", "conferenceEmailAddress", "mail_to"):
                if d.get(k) not in (None, ""):
                    d["conference_email_address"] = str(d.pop(k)).strip()
                    break
        # account_code
        if not d.get("conference_account_code"):
            for k in ("account_code", "accountCode", "conferenceAccountCode"):
                if d.get(k) not in (None, ""):
                    d["conference_account_code"] = str(d.pop(k)).strip()
                    break
        # dialplan_uuid alias
        if not d.get("dialplan_uuid"):
            for k in ("dialplanUuid", "dialplan_id"):
                if d.get(k) not in (None, ""):
                    d["dialplan_uuid"] = d.pop(k)
                    break
        return d

    @field_validator("conference_extension")
    @classmethod
    def validate_extension(cls, v):
        if v is None:
            return v
        s = str(v).strip()
        if not re.match(r"^\d{2,7}$", s):
            raise ValueError("conference_extension must be 2-7 digits")
        return s

    @field_validator("conference_name")
    @classmethod
    def validate_name(cls, v):
        if v is None:
            return v
        s = str(v).strip()
        if len(s) == 0:
            raise ValueError("conference_name is required")
        return s


class ConferenceRead(BaseModel):
    conference_uuid: UUID
    domain_id: UUID
    conference_name: str
    conference_extension: str
    conference_pin_number: Optional[str] = None
    conference_profile: str
    conference_enabled: bool
    conference_description: Optional[str] = None
    conference_flags: Optional[str] = None
    conference_order: Optional[int] = None
    conference_context: Optional[str] = None
    conference_email_address: Optional[str] = None
    conference_account_code: Optional[str] = None
    dialplan_uuid: Optional[UUID] = None
    created_at: datetime
    updated_at: datetime

    model_config = ConfigDict(from_attributes=True)


class ConferenceUpdate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)

    conference_name: Optional[str] = None
    conference_extension: Optional[str] = None
    conference_pin_number: Optional[str] = None
    conference_profile: Optional[str] = None
    conference_enabled: Optional[bool] = None
    conference_description: Optional[str] = None
    conference_flags: Optional[str] = None
    conference_order: Optional[int] = None
    conference_context: Optional[str] = None
    conference_email_address: Optional[str] = None
    conference_account_code: Optional[str] = None
    dialplan_uuid: Optional[UUID] = None

    @model_validator(mode="before")
    @classmethod
    def map_legacy_update(cls, data: Any):
        if not isinstance(data, dict):
            return data
        d = dict(data)
        if "name" in d and not d.get("conference_name"):
            d["conference_name"] = str(d.pop("name")).strip()
        if "conferenceName" in d and not d.get("conference_name"):
            d["conference_name"] = str(d.pop("conferenceName")).strip()
        if "extension" in d and not d.get("conference_extension"):
            d["conference_extension"] = str(d.pop("extension")).strip()
        if "conferenceExtension" in d and not d.get("conference_extension"):
            d["conference_extension"] = str(d.pop("conferenceExtension")).strip()
        if "pin_number" in d and not d.get("conference_pin_number"):
            d["conference_pin_number"] = str(d.pop("pin_number")).strip()
        if "pin" in d and not d.get("conference_pin_number"):
            d["conference_pin_number"] = str(d.pop("pin")).strip()
        if "pinNumber" in d and not d.get("conference_pin_number"):
            d["conference_pin_number"] = str(d.pop("pinNumber")).strip()
        if "password" in d and not d.get("conference_pin_number"):
            d["conference_pin_number"] = str(d.pop("password")).strip()
        if "profile" in d and not d.get("conference_profile"):
            d["conference_profile"] = str(d.pop("profile")).strip()
        if "conferenceProfile" in d and not d.get("conference_profile"):
            d["conference_profile"] = str(d.pop("conferenceProfile")).strip()
        if "enabled" in d and d.get("conference_enabled") is None:
            val = d.pop("enabled")
            if isinstance(val, str):
                d["conference_enabled"] = val.lower() in ("true", "1", "yes", "t")
            else:
                d["conference_enabled"] = bool(val)
        if "conferenceEnabled" in d and d.get("conference_enabled") is None:
            val = d.pop("conferenceEnabled")
            if isinstance(val, str):
                d["conference_enabled"] = val.lower() in ("true", "1", "yes", "t")
            else:
                d["conference_enabled"] = bool(val)
        if "description" in d and not d.get("conference_description"):
            d["conference_description"] = d.pop("description")
        if "conferenceDescription" in d and not d.get("conference_description"):
            d["conference_description"] = d.pop("conferenceDescription")
        if "flags" in d and not d.get("conference_flags"):
            d["conference_flags"] = str(d.pop("flags"))
        if "order" in d and d.get("conference_order") is None:
            try:
                d["conference_order"] = int(d.pop("order"))
            except:
                d["conference_order"] = d.pop("order")
        if "context" in d and not d.get("conference_context"):
            d["conference_context"] = str(d.pop("context"))
        if "email" in d and not d.get("conference_email_address"):
            d["conference_email_address"] = str(d.pop("email"))
        if "email_address" in d and not d.get("conference_email_address"):
            d["conference_email_address"] = str(d.pop("email_address"))
        if "account_code" in d and not d.get("conference_account_code"):
            d["conference_account_code"] = str(d.pop("account_code"))
        return d

    @field_validator("conference_extension")
    @classmethod
    def validate_extension(cls, v):
        if v is None:
            return v
        s = str(v).strip()
        if not re.match(r"^\d{2,7}$", s):
            raise ValueError("conference_extension must be 2-7 digits")
        return s


# For member handling
class ConferenceUserCreate(BaseModel):
    user_uuid: UUID


class ConferenceUserRead(BaseModel):
    conference_user_uuid: UUID
    conference_uuid: UUID
    domain_id: UUID
    user_uuid: UUID

    class Config:
        from_attributes = True
