from datetime import datetime
from typing import Optional
from uuid import UUID
from pydantic import BaseModel, ConfigDict, model_validator
class DatabaseTransactionCreate(BaseModel):
    model_config=ConfigDict(extra="allow", populate_by_name=True)
    domain_id: Optional[UUID]=None
    app_name: Optional[str]=None
    transaction_code: Optional[str]=None
    transaction_type: Optional[str]=None
    transaction_address: Optional[str]=None
    @model_validator(mode="before")
    @classmethod
    def map_aliases(cls,data):
        if isinstance(data, dict) and "domain_uuid" in data and "domain_id" not in data:
            data["domain_id"]=data.pop("domain_uuid")
        return data
class DatabaseTransactionRead(BaseModel):
    model_config=ConfigDict(from_attributes=True)
    database_transaction_uuid: UUID
    domain_id: UUID
    app_name: Optional[str]=None
    transaction_code: Optional[str]=None
    transaction_type: Optional[str]=None
    transaction_date: datetime
class DatabaseTransactionUpdate(BaseModel):
    model_config=ConfigDict(extra="allow")
    transaction_type: Optional[str]=None
