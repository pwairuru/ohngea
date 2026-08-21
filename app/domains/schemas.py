from datetime import datetime
from typing import Optional
from uuid import UUID

from pydantic import BaseModel


class DomainCreate(BaseModel):
    domain_name: str
    enabled: bool = True
    description: Optional[str] = None


class DomainUpdate(BaseModel):
    enabled: Optional[bool] = None
    description: Optional[str] = None


class DomainRead(BaseModel):
    domain_id: UUID
    domain_name: str
    enabled: bool
    description: Optional[str] = None
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True


class ThemeRead(BaseModel):
    domain_id: UUID
    primary_color: str = "221 83% 53%"
    background: str = "0 0% 100%"
    radius: str = "0.5rem"
