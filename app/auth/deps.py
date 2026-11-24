from typing import Optional
from uuid import UUID

from fastapi import Depends, Header, HTTPException, Request, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.models import User
from app.auth.service import decode_token, get_user_permissions
from app.db import get_session


async def get_current_user(
    request: Request,
    session: AsyncSession = Depends(get_session),
    authorization: Optional[str] = Header(None),
) -> User:
    # Accept Bearer header OR httpOnly `access_token` cookie (set on login).
    token: str | None = None
    if authorization and authorization.startswith("Bearer "):
        token = authorization.removeprefix("Bearer ").strip()
    if not token:
        token = request.cookies.get("access_token")
    if not token:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Missing token")
    try:
        payload = decode_token(token)
    except Exception:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid token")
    user_id = payload.get("sub")
    if not user_id:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid payload")
    result = await session.execute(select(User).where(User.user_uuid == UUID(user_id)))
    user = result.scalar_one_or_none()
    if not user or not user.enabled:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="User not found or disabled")
    # Load fresh perms from DB (never trust token claim) so downstream
    # _perms checks (groups service, etc.) work securely.
    try:
        user._perms = await get_user_permissions(session, user)  # type: ignore[attr-defined]
    except Exception:
        user._perms = []  # type: ignore[attr-defined]
    user._domain_id = UUID(payload["domain_id"]) if payload.get("domain_id") else user.domain_id
    return user


def require_permission(perm: str):
    async def checker(
        user: User = Depends(get_current_user),
        session: AsyncSession = Depends(get_session),
    ) -> User:
        # _perms already DB-loaded in get_current_user; refresh if missing
        perms = getattr(user, "_perms", None)
        if perms is None:
            perms = await get_user_permissions(session, user)
        if "*" in perms or perm in perms:
            return user
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail=f"Missing permission: {perm}")

    return checker


async def get_tenant_id(user: User = Depends(get_current_user)) -> UUID:
    return getattr(user, "_domain_id", user.domain_id)
