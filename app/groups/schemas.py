from datetime import datetime
from typing import Optional, List
from uuid import UUID

from pydantic import BaseModel, ConfigDict


class GroupCreate(BaseModel):
    model_config = ConfigDict(extra="allow", populate_by_name=True)
    name: str
    description: Optional[str] = None
    domain_id: Optional[UUID] = None  # None => global (legacy domain_uuid IS NULL)
    group_level: Optional[int] = 10  # 10..90 step 10
    group_protected: Optional[bool] = False


class GroupRead(BaseModel):
    model_config = ConfigDict(from_attributes=True)
    group_uuid: UUID
    domain_id: Optional[UUID] = None
    name: str
    description: Optional[str] = None
    group_level: int = 10
    group_protected: bool = False
    created_at: datetime


class GroupUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    name: Optional[str] = None
    description: Optional[str] = None
    group_level: Optional[int] = None
    group_protected: Optional[bool] = None
    domain_id: Optional[UUID] = None


class GroupBulkAction(BaseModel):
    model_config = ConfigDict(extra="allow")
    action: str  # copy | toggle | delete
    ids: List[UUID]


class GroupPermissionUpdate(BaseModel):
    model_config = ConfigDict(extra="allow")
    permission_uuids: Optional[List[UUID]] = None
    permission_names: Optional[List[str]] = None
    permissions: Optional[List[str]] = None  # alias for names
    # extended fields for protected parity (plan task 5)
    permission_protected: Optional[dict] = None
    protected_map: Optional[dict] = None
    permissions_data: Optional[List[dict]] = None
    # alternative: list of {permission_uuid/name, assigned, protected}
    items: Optional[List[dict]] = None
