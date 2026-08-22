from datetime import datetime
from typing import Optional, Any
from uuid import UUID

from pydantic import BaseModel, ConfigDict, field_validator, model_validator
import re


class TimeConditionCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)

    domain_id: Optional[UUID] = None
    name: Optional[str] = None
    extension: Optional[str] = None
    context: Optional[str] = None
    enabled: Optional[bool] = True
    description: Optional[str] = None
    dialplan_uuid: Optional[UUID] = None
    time_hour: Optional[str] = None
    time_minute: Optional[str] = None
    time_wday: Optional[str] = None
    time_mday: Optional[str] = None
    time_mon: Optional[str] = None
    time_year: Optional[str] = None
    time_yday: Optional[str] = None
    time_week: Optional[str] = None
    time_mweek: Optional[str] = None
    minute_of_day: Optional[str] = None
    time_of_day: Optional[str] = None
    date_time: Optional[str] = None
    condition: Optional[str] = None
    true_destination: Optional[str] = None
    false_destination: Optional[str] = None
    timezone: Optional[str] = None
    details: Optional[Any] = None

    # allow legacy aliases
    @model_validator(mode="before")
    @classmethod
    def map_legacy(cls, data: Any):
        if not isinstance(data, dict):
            return data
        # work on copy
        d = dict(data)
        # name aliases
        if not d.get("name"):
            for k in ("time_condition_name", "dialplan_name", "timeConditionName", "label"):
                if d.get(k):
                    d["name"] = d.pop(k)
                    break
        # extension aliases
        if not d.get("extension"):
            for k in ("time_condition_extension", "dialplan_number", "dialplanNumber", "extension_number", "destination", "dialplan_extension"):
                if d.get(k) is not None and str(d.get(k)).strip() != "":
                    d["extension"] = str(d.pop(k)).strip()
                    break
                elif k in d:
                    # remove alias to avoid extra confusion but keep empty handling
                    pass
        # description aliases
        if not d.get("description"):
            for k in ("dialplan_description", "time_condition_description"):
                if d.get(k):
                    d["description"] = d.pop(k)
                    break
        # context aliases
        if not d.get("context"):
            for k in ("dialplan_context", "time_condition_context"):
                if d.get(k):
                    d["context"] = d.pop(k)
                    break
        # enabled aliases
        if "enabled" not in d or d["enabled"] is None:
            for k in ("dialplan_enabled", "time_condition_enabled"):
                if k in d and d[k] is not None:
                    val = d.pop(k)
                    if isinstance(val, str):
                        d["enabled"] = val.lower() in ("true", "1", "yes")
                    else:
                        d["enabled"] = bool(val)
                    break
        # dialplan_uuid aliases
        if not d.get("dialplan_uuid"):
            for k in ("time_condition_uuid", "dialplanUuid"):
                if d.get(k):
                    d["dialplan_uuid"] = d.pop(k)
                    break
        # condition aliases
        if not d.get("condition"):
            for k in ("time_condition", "cron", "expression"):
                if d.get(k):
                    d["condition"] = d.pop(k)
                    break
        # true/false destination aliases (mock frontend uses true_dest/false_dest)
        if not d.get("true_destination"):
            for k in ("true_dest", "trueDest", "dialplan_action", "action_true", "app_true"):
                if d.get(k):
                    d["true_destination"] = str(d.pop(k))
                    break
        if not d.get("false_destination"):
            for k in ("false_dest", "falseDest", "dialplan_anti_action", "action_false", "app_false", "anti_action"):
                if d.get(k):
                    d["false_destination"] = str(d.pop(k))
                    break
        # time fields mapping from various possible keys
        mapping = {
            "time_hour": ("hour", "timeHour", "hour_of_day"),
            "time_minute": ("minute", "timeMinute"),
            "time_wday": ("wday", "day_of_week", "dayOfWeek", "weekday", "time_day_of_week"),
            "time_mday": ("mday", "day_of_month", "dayOfMonth", "time_day_of_month"),
            "time_mon": ("mon", "month", "time_month"),
            "time_year": ("year", "time_year"),
            "minute_of_day": ("minute-of-day", "minuteOfDay"),
            "time_of_day": ("time-of-day", "timeOfDay"),
            "date_time": ("date-time", "dateTime"),
        }
        for canonical, aliases in mapping.items():
            if not d.get(canonical):
                for ak in aliases:
                    if ak in d and d[ak] is not None:
                        d[canonical] = str(d.pop(ak))
                        break
        # timezone aliases
        if not d.get("timezone"):
            for k in ("dialplan_time_zone", "time_zone", "tz"):
                if d.get(k):
                    d["timezone"] = str(d.pop(k))
                    break
        return d

    @field_validator("extension")
    @classmethod
    def validate_extension(cls, v):
        if v is None:
            return v
        if not re.match(r"^\d{2,7}$", str(v)):
            raise ValueError("extension must be 2-7 digits")
        return str(v)

    @field_validator("name")
    @classmethod
    def validate_name(cls, v):
        if v is None:
            return v
        if isinstance(v, str) and len(v.strip()) == 0:
            raise ValueError("name is required")
        return v.strip() if isinstance(v, str) else v


class TimeConditionRead(BaseModel):
    time_condition_uuid: UUID
    domain_id: UUID
    name: str
    extension: str
    context: Optional[str] = None
    enabled: bool
    description: Optional[str] = None
    dialplan_uuid: Optional[UUID] = None
    time_hour: Optional[str] = None
    time_minute: Optional[str] = None
    time_wday: Optional[str] = None
    time_mday: Optional[str] = None
    time_mon: Optional[str] = None
    time_year: Optional[str] = None
    time_yday: Optional[str] = None
    time_week: Optional[str] = None
    time_mweek: Optional[str] = None
    minute_of_day: Optional[str] = None
    time_of_day: Optional[str] = None
    date_time: Optional[str] = None
    condition: Optional[str] = None
    true_destination: Optional[str] = None
    false_destination: Optional[str] = None
    timezone: Optional[str] = None
    details: Optional[Any] = None
    created_at: datetime
    updated_at: datetime

    model_config = ConfigDict(from_attributes=True)


class TimeConditionUpdate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)

    name: Optional[str] = None
    extension: Optional[str] = None
    context: Optional[str] = None
    enabled: Optional[bool] = None
    description: Optional[str] = None
    dialplan_uuid: Optional[UUID] = None
    time_hour: Optional[str] = None
    time_minute: Optional[str] = None
    time_wday: Optional[str] = None
    time_mday: Optional[str] = None
    time_mon: Optional[str] = None
    time_year: Optional[str] = None
    time_yday: Optional[str] = None
    time_week: Optional[str] = None
    time_mweek: Optional[str] = None
    minute_of_day: Optional[str] = None
    time_of_day: Optional[str] = None
    date_time: Optional[str] = None
    condition: Optional[str] = None
    true_destination: Optional[str] = None
    false_destination: Optional[str] = None
    timezone: Optional[str] = None
    details: Optional[Any] = None

    @model_validator(mode="before")
    @classmethod
    def map_legacy_update(cls, data: Any):
        if not isinstance(data, dict):
            return data
        d = dict(data)
        # same mapping as create but allow partial
        if "time_condition_name" in d and not d.get("name"):
            d["name"] = d.pop("time_condition_name")
        if "dialplan_name" in d and not d.get("name"):
            d["name"] = d.pop("dialplan_name")
        if "time_condition_extension" in d and not d.get("extension"):
            d["extension"] = str(d.pop("time_condition_extension"))
        if "dialplan_number" in d and not d.get("extension"):
            d["extension"] = str(d.pop("dialplan_number"))
        if "dialplan_description" in d and not d.get("description"):
            d["description"] = d.pop("dialplan_description")
        if "dialplan_context" in d and not d.get("context"):
            d["context"] = d.pop("dialplan_context")
        if "true_dest" in d and not d.get("true_destination"):
            d["true_destination"] = str(d.pop("true_dest"))
        if "false_dest" in d and not d.get("false_destination"):
            d["false_destination"] = str(d.pop("false_dest"))
        if "dialplan_anti_action" in d and not d.get("false_destination"):
            d["false_destination"] = str(d.pop("dialplan_anti_action"))
        return d

    @field_validator("extension")
    @classmethod
    def validate_extension(cls, v):
        if v is None:
            return v
        if not re.match(r"^\d{2,7}$", str(v)):
            raise ValueError("extension must be 2-7 digits")
        return str(v)
