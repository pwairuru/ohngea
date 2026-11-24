from datetime import datetime, timezone
from typing import Optional
from uuid import UUID
from sqlmodel import Field, SQLModel
from app.db import uuid7
class DatabaseTransactionBase(SQLModel):
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    user_uuid: Optional[UUID]=Field(default=None, index=True)
    app_name: Optional[str]=Field(default=None, max_length=128)
    app_uuid: Optional[UUID]=Field(default=None, index=True)
    transaction_code: Optional[str]=Field(default=None, max_length=512)
    transaction_address: Optional[str]=Field(default=None, max_length=64)
    transaction_type: Optional[str]=Field(default=None, max_length=16)
    transaction_old: Optional[str]=Field(default=None)
    transaction_new: Optional[str]=Field(default=None)
    transaction_result: Optional[str]=Field(default=None)
class DatabaseTransaction(DatabaseTransactionBase, table=True):
    __tablename__="database_transactions"
    database_transaction_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    transaction_date: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
