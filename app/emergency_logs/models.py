from datetime import datetime
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class EmergencyLogBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    extension: Optional[str]=Field(default=None, max_length=32)
    event: Optional[str]=Field(default=None, max_length=512)
class EmergencyLog(EmergencyLogBase, table=True):
    __tablename__="emergency_logs"
    emergency_log_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
