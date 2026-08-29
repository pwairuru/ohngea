from datetime import datetime, timezone
from typing import Optional
from uuid import UUID

from sqlmodel import Field, SQLModel

from app.db import uuid7


class DialplanBase(SQLModel):
    name: str = Field(max_length=128, index=True)
    context: str = Field(max_length=128, description="e.g. tenant.example.com")
    enabled: bool = Field(default=True)
    domain_id: UUID = Field(foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    description: Optional[str] = Field(default=None)
    # full parity
    dialplan_continue: bool = Field(default=False, description="continue flag")
    dialplan_order: Optional[int] = Field(default=100)
    hostname: Optional[str] = Field(default=None, max_length=128)
    app_uuid: Optional[str] = Field(default=None, max_length=64)
    # legacy parity — nullable additions
    dialplan_number: Optional[str] = Field(default=None, max_length=128, description="dialplan_number for destinations")
    dialplan_destination: Optional[bool] = Field(default=None, description="whether exposed as destination")
    dialplan_xml: Optional[str] = Field(default=None, description="cached dialplan xml")

    # alias for legacy column name dialplan_enabled <-> enabled
    @property
    def dialplan_enabled(self) -> bool:
        return self.enabled

    @dialplan_enabled.setter
    def dialplan_enabled(self, value: bool):
        self.enabled = bool(value) if value is not None else True

    @property
    def dialplan_name(self) -> str:
        return self.name

    @dialplan_name.setter
    def dialplan_name(self, value: str):
        self.name = value

    @property
    def dialplan_context(self) -> str:
        return self.context

    @dialplan_context.setter
    def dialplan_context(self, value: str):
        self.context = value

    @property
    def dialplan_description(self) -> Optional[str]:
        return self.description

    @dialplan_description.setter
    def dialplan_description(self, value: Optional[str]):
        self.description = value


class Dialplan(DialplanBase, table=True):
    __tablename__ = "dialplans"
    dialplan_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    created_at: datetime = Field(default_factory=lambda: datetime.utcnow())
    updated_at: datetime = Field(default_factory=lambda: datetime.utcnow())


class DialplanDetail(SQLModel, table=True):
    __tablename__ = "dialplan_details"
    detail_uuid: UUID = Field(default_factory=uuid7, primary_key=True)
    dialplan_uuid: UUID = Field(foreign_key="dialplans.dialplan_uuid", ondelete="CASCADE", index=True)
    tag: str = Field(max_length=32)  # condition, action, anti-action
    type: str = Field(max_length=64)  # destination_number, etc.
    data: str = Field(max_length=512)
    order: int = Field(default=0)
    # parity additions — nullable, do not rename existing columns
    domain_id: Optional[UUID] = Field(default=None, foreign_key="domains.domain_id", index=True, ondelete="CASCADE")
    dialplan_detail_break: Optional[str] = Field(default=None, max_length=32, description="break: on-true/on-false/never/always")
    dialplan_detail_inline: Optional[str] = Field(default=None, max_length=32, description="inline: true/false")
    dialplan_detail_group: Optional[int] = Field(default=None, description="group ordering")
    # task requires detail_enabled column; legacy alias dialplan_detail_enabled also supported
    detail_enabled: Optional[bool] = Field(default=True, description="detail enabled flag")
    # alias dialplan_detail_enabled <-> detail_enabled
    @property
    def dialplan_detail_enabled(self) -> Optional[bool]:
        return self.detail_enabled

    @dialplan_detail_enabled.setter
    def dialplan_detail_enabled(self, value: Optional[bool]):
        self.detail_enabled = bool(value) if value is not None else True

    @property
    def dialplan_detail_tag(self) -> str:
        return self.tag

    @dialplan_detail_tag.setter
    def dialplan_detail_tag(self, value: str):
        self.tag = value

    @property
    def dialplan_detail_type(self) -> str:
        return self.type

    @dialplan_detail_type.setter
    def dialplan_detail_type(self, value: str):
        self.type = value

    @property
    def dialplan_detail_data(self) -> str:
        return self.data

    @dialplan_detail_data.setter
    def dialplan_detail_data(self, value: str):
        self.data = value

    @property
    def dialplan_detail_order(self) -> int:
        return self.order

    @dialplan_detail_order.setter
    def dialplan_detail_order(self, value: int):
        self.order = value
