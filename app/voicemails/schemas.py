from datetime import datetime
from typing import Optional, List, Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, field_validator, model_validator
import re


class VoicemailDestinationCreate(BaseModel):
    voicemail_uuid_copy: UUID


class VoicemailDestinationRead(BaseModel):
    voicemail_destination_uuid: UUID
    voicemail_uuid: UUID
    domain_id: UUID
    voicemail_uuid_copy: UUID

    class Config:
        from_attributes = True


VoicemailCopyCreate = VoicemailDestinationCreate
VoicemailCopyRead = VoicemailDestinationRead


class VoicemailCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)

    domain_id: Optional[UUID] = None
    voicemail_id: Optional[str] = None
    voicemail_password: Optional[str] = None
    enabled: Optional[bool] = True
    description: Optional[str] = None
    greeting_id: Optional[int] = None
    toll_allow: Optional[str] = None
    voicemail_mail_to: Optional[str] = None
    voicemail_file: Optional[str] = None
    alternate_greet_id: Optional[str] = None
    forward_destinations: Optional[List[Any]] = None
    destinations: Optional[List[Any]] = None  # alias

    @model_validator(mode="before")
    @classmethod
    def map_legacy(cls, data: Any):
        if not isinstance(data, dict):
            return data
        d = dict(data)
        # domain alias
        if not d.get("domain_id"):
            for k in ("domain_uuid", "domainId", "domain", "domain_name"):
                if d.get(k):
                    try:
                        # if domain_name string, keep as is? but domain_id expects UUID, leave mapping to fail later
                        d["domain_id"] = d.pop(k)
                    except Exception:
                        d["domain_id"] = d.pop(k)
                    break
                elif k in d and d[k] is None:
                    d.pop(k, None)
        # voicemail_id aliases
        if not d.get("voicemail_id"):
            for k in ("voicemail_mailbox", "mailbox", "extension", "voicemailId", "vm_id", "destination", "voicemail_number", "mailbox_number"):
                if d.get(k) not in (None, ""):
                    d["voicemail_id"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    # remove empty alias to avoid extra confusion
                    if d[k] == "" or d[k] is None:
                        d.pop(k, None)
        # password aliases
        if not d.get("voicemail_password"):
            for k in ("password", "voicemailPassword", "pin", "voicemail_pin", "pass", "passwd"):
                if d.get(k) not in (None, ""):
                    d["voicemail_password"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    d.pop(k, None)
        # enabled aliases
        if "enabled" not in d or d["enabled"] is None:
            for k in ("voicemail_enabled", "voicemailEnabled", "is_enabled", "isEnabled", "active"):
                if k in d and d[k] is not None:
                    val = d.pop(k)
                    if isinstance(val, str):
                        d["enabled"] = val.lower() in ("true", "1", "yes", "t")
                    else:
                        d["enabled"] = bool(val)
                    break
        else:
            # normalize if string
            if isinstance(d.get("enabled"), str):
                d["enabled"] = d["enabled"].lower() in ("true", "1", "yes", "t")
        # description aliases
        if not d.get("description"):
            for k in ("voicemail_description", "voicemailDescription", "descr"):
                if d.get(k):
                    d["description"] = d.pop(k)
                    break
        # greeting aliases
        if d.get("greeting_id") is None:
            for k in ("greeting", "greet_id", "voicemail_greeting", "greetingId", "greeting_name"):
                if k in d and d[k] not in (None, ""):
                    val = d.pop(k)
                    try:
                        d["greeting_id"] = int(val) if str(val).isdigit() else None
                        if d["greeting_id"] is None:
                            # if greeting is string like "Unavailable", ignore?
                            d["greeting_id"] = None
                    except:
                        d["greeting_id"] = None
                    break
                elif k in d:
                    d.pop(k, None)
        # toll_allow aliases
        if not d.get("toll_allow"):
            for k in ("tollAllow", "toll", "toll_allow_id", "class"):
                if d.get(k) not in (None, ""):
                    d["toll_allow"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    d.pop(k, None)
        # mail_to aliases
        if not d.get("voicemail_mail_to"):
            for k in ("mail_to", "email", "voicemailMailTo", "mailTo"):
                if d.get(k) not in (None, ""):
                    d["voicemail_mail_to"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    d.pop(k, None)
        # forward_destinations aliases — normalize to forward_destinations
        if not d.get("forward_destinations") and not d.get("destinations"):
            for k in ("forwardDestinations", "copies", "voicemail_destinations", "voicemail_copies", "destination_uuids", "copy_uuids", "forward_destinations_list"):
                if k in d and d[k] not in (None, ""):
                    d["forward_destinations"] = d.pop(k)
                    break
        # if both provided, merge
        if d.get("destinations") and not d.get("forward_destinations"):
            d["forward_destinations"] = d.pop("destinations")
        elif d.get("destinations") and d.get("forward_destinations"):
            # keep both? merge later in service
            d.pop("destinations", None)

        # voicemail_file alias
        if not d.get("voicemail_file"):
            for k in ("file", "attach_file", "voicemailFile"):
                if d.get(k) not in (None, ""):
                    d["voicemail_file"] = str(d.pop(k)).strip()
                    break

        # alternate_greet_id alias
        if not d.get("alternate_greet_id"):
            for k in ("voicemail_alternate_greet_id", "alternate_greet", "alt_greet_id"):
                if d.get(k) not in (None, ""):
                    d["alternate_greet_id"] = str(d.pop(k)).strip()
                    break

        return d

    @field_validator("voicemail_id")
    @classmethod
    def validate_id(cls, v):
        if v is None:
            return v
        s = str(v).strip()
        if not re.match(r"^\d{2,7}$", s):
            raise ValueError("voicemail_id must be 2-7 digits")
        return s

    @field_validator("voicemail_password")
    @classmethod
    def validate_pwd(cls, v):
        if v is None:
            return v
        s = str(v).strip()
        if not re.match(r"^\d{2,20}$", s):
            raise ValueError("voicemail_password must be 2-20 digits")
        return s

    @field_validator("forward_destinations", mode="before")
    @classmethod
    def normalize_forward(cls, v):
        if v is None:
            return v
        if not isinstance(v, list):
            raise ValueError("forward_destinations must be a list")
        return v


class VoicemailRead(BaseModel):
    voicemail_uuid: UUID
    domain_id: UUID
    voicemail_id: str
    voicemail_password: str
    enabled: bool
    description: Optional[str] = None
    greeting_id: Optional[int] = None
    toll_allow: Optional[str] = None
    voicemail_mail_to: Optional[str] = None
    voicemail_file: Optional[str] = None
    alternate_greet_id: Optional[str] = None
    created_at: datetime
    updated_at: datetime
    # destinations as embedded list if requested — optional
    forward_destinations: List[VoicemailDestinationRead] = []
    destinations: List[VoicemailDestinationRead] = []

    model_config = ConfigDict(from_attributes=True)


class VoicemailUpdate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)

    voicemail_id: Optional[str] = None
    voicemail_password: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None
    greeting_id: Optional[int] = None
    toll_allow: Optional[str] = None
    voicemail_mail_to: Optional[str] = None
    voicemail_file: Optional[str] = None
    alternate_greet_id: Optional[str] = None
    forward_destinations: Optional[List[Any]] = None
    destinations: Optional[List[Any]] = None

    @model_validator(mode="before")
    @classmethod
    def map_legacy_update(cls, data: Any):
        if not isinstance(data, dict):
            return data
        d = dict(data)
        # reuse same alias mapping but partial
        if "voicemail_mailbox" in d and not d.get("voicemail_id"):
            d["voicemail_id"] = str(d.pop("voicemail_mailbox")).strip()
        if "mailbox" in d and not d.get("voicemail_id"):
            d["voicemail_id"] = str(d.pop("mailbox")).strip()
        if "extension" in d and not d.get("voicemail_id"):
            d["voicemail_id"] = str(d.pop("extension")).strip()
        if "voicemail_enabled" in d and d.get("enabled") is None:
            val = d.pop("voicemail_enabled")
            if isinstance(val, str):
                d["enabled"] = val.lower() in ("true", "1", "yes", "t")
            else:
                d["enabled"] = bool(val)
        if "voicemail_description" in d and not d.get("description"):
            d["description"] = d.pop("voicemail_description")
        if "voicemail_password" in d and not d.get("voicemail_password"):
            # already correct key, keep
            pass
        if "password" in d and not d.get("voicemail_password"):
            d["voicemail_password"] = str(d.pop("password")).strip()
        if "greeting" in d and d.get("greeting_id") is None:
            val = d.pop("greeting")
            try:
                d["greeting_id"] = int(val) if str(val).isdigit() else None
            except:
                d["greeting_id"] = None
        if "tollAllow" in d and not d.get("toll_allow"):
            d["toll_allow"] = str(d.pop("tollAllow")).strip()
        if "mail_to" in d and not d.get("voicemail_mail_to"):
            d["voicemail_mail_to"] = str(d.pop("mail_to")).strip()
        if "email" in d and not d.get("voicemail_mail_to"):
            d["voicemail_mail_to"] = str(d.pop("email")).strip()
        if "forwardDestinations" in d and not d.get("forward_destinations"):
            d["forward_destinations"] = d.pop("forwardDestinations")
        if "destinations" in d and not d.get("forward_destinations"):
            d["forward_destinations"] = d.pop("destinations")
        if "voicemail_file" in d and not d.get("voicemail_file"):
            d["voicemail_file"] = str(d.pop("voicemail_file")).strip()
        return d

    @field_validator("voicemail_id")
    @classmethod
    def validate_id(cls, v):
        if v is None:
            return v
        s = str(v).strip()
        if not re.match(r"^\d{2,7}$", s):
            raise ValueError("voicemail_id must be 2-7 digits")
        return s

    @field_validator("voicemail_password")
    @classmethod
    def validate_pwd(cls, v):
        if v is None:
            return v
        s = str(v).strip()
        if not re.match(r"^\d{2,20}$", s):
            raise ValueError("voicemail_password must be 2-20 digits")
        return s
