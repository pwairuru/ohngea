from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class FeatureCodeBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    feature_code_name: str = Field(max_length=128)
    feature_code_value: str = Field(max_length=32)
    feature_code_enabled: bool = Field(default=True)
    feature_code_description: Optional[str]=Field(default=None, max_length=512)
    insert_user: Optional[UUID]=Field(default=None)
    update_user: Optional[UUID]=Field(default=None)
class FeatureCode(FeatureCodeBase, table=True):
    __tablename__="feature_codes"
    feature_code_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
