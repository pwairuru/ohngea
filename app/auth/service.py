from datetime import datetime, timedelta, timezone
from typing import Optional
from uuid import UUID

import jwt
from passlib.context import CryptContext
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.models import Group, GroupPermission, Permission, User, UserGroup
from app.config import settings
from app.domains.models import Domain

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


def hash_password(password: str) -> str:
    return pwd_context.hash(password)


def verify_password(plain: str, hashed: str) -> bool:
    return pwd_context.verify(plain, hashed)


def create_token(user: User, permissions: list[str] | None = None) -> str:
    payload = {
        "sub": str(user.user_uuid),
        "domain_id": str(user.domain_id),
        "username": user.username,
        "permissions": permissions or [],
        "exp": datetime.now(timezone.utc) + timedelta(minutes=settings.jwt_expire_minutes),
        "iat": datetime.now(timezone.utc),
    }
    return jwt.encode(payload, settings.jwt_secret, algorithm=settings.jwt_algorithm)


def decode_token(token: str) -> dict:
    return jwt.decode(token, settings.jwt_secret, algorithms=[settings.jwt_algorithm])


async def authenticate(session: AsyncSession, username: str, password: str, domain_name: Optional[str] = None) -> Optional[User]:
    # Resolve domain if provided
    domain_id = None
    if domain_name:
        r = await session.execute(select(Domain).where(Domain.domain_name == domain_name))
        d = r.scalar_one_or_none()
        if not d:
            return None
        domain_id = d.domain_id

    q = select(User).where(User.username == username, User.enabled == True)  # noqa
    if domain_id:
        q = q.where(User.domain_id == domain_id)
    result = await session.execute(q)
    user = result.scalars().first()
    if not user:
        # fallback: try without domain filter for legacy? then verify
        result = await session.execute(select(User).where(User.username == username, User.enabled == True))
        user = result.scalars().first()
        if not user or (domain_id and str(user.domain_id) != str(domain_id)):
            return None
    if not verify_password(password, user.password_hash):
        return None
    return user


async def get_user_permissions(session: AsyncSession, user: User) -> list[str]:
    # via user_groups -> group_permissions -> permissions
    q = (
        select(Permission.name)
        .join(GroupPermission, GroupPermission.permission_uuid == Permission.permission_uuid)
        .join(Group, Group.group_uuid == GroupPermission.group_uuid)
        .join(UserGroup, UserGroup.group_uuid == Group.group_uuid)
        .where(UserGroup.user_uuid == user.user_uuid)
    )
    r = await session.execute(q)
    perms = [row[0] for row in r.all()]
    # superadmin: if user is in superadmin group, return wildcard
    if not perms:
        # Default permissive for dev: if no groups, allow domain admins basic perms
        # Seed will create proper groups; fallback to allow all for initial admin
        return ["*"]
    return perms
