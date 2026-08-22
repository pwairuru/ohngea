from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class VoicemailGreetingCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    voicemail_id: Optional[str]=None
    mailbox: Optional[str]=None
    greeting_id: Optional[int]=None
    greeting_name: Optional[str]=None
    name: Optional[str]=None
    greeting_filename: Optional[str]=None
    greeting_description: Optional[str]=None
    description: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict):
            if "mailbox" in data and "voicemail_id" not in data: data["voicemail_id"]=str(data.pop("mailbox"))
            if "name" in data and "greeting_name" not in data: data["greeting_name"]=data.pop("name")
            if "description" in data and "greeting_description" not in data: data["greeting_description"]=data.pop("description")
            if "domain_uuid" in data and "domain_id" not in data: data["domain_id"]=data.pop("domain_uuid")
        return data
class VoicemailGreetingRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    voicemail_greeting_uuid: UUID
    domain_id: UUID
    voicemail_id: str
    greeting_id: int
    greeting_name: Optional[str]=None
    greeting_filename: Optional[str]=None
    greeting_description: Optional[str]=None
    created_at: datetime
    updated_at: datetime
class VoicemailGreetingUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    greeting_name: Optional[str]=None
    greeting_filename: Optional[str]=None
    greeting_description: Optional[str]=None
    greeting_id: Optional[int]=None
