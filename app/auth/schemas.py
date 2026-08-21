from typing import Optional
from uuid import UUID

from pydantic import BaseModel


class LoginRequest(BaseModel):
    username: str
    password: str
    domain: Optional[str] = None  # domain_name or domain_id
    domain_name: Optional[str] = None  # alias


class TokenResponse(BaseModel):
    access_token: str
    refresh_token: Optional[str] = None
    token_type: str = "Bearer"
    expires_in: int = 86400


class UserCreate(BaseModel):
    username: str
    password: str
    email: Optional[str] = None
    domain_id: UUID


class UserRead(BaseModel):
    user_uuid: UUID
    username: str
    email: Optional[str] = None
    domain_id: UUID
    enabled: bool
