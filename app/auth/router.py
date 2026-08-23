from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.schemas import LoginRequest, TokenResponse, UserRead
from app.auth.service import authenticate, create_token, get_user_permissions
from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session

router = APIRouter(prefix="/auth", tags=["auth"])


@router.post("/login", response_model=TokenResponse)
async def login(payload: LoginRequest, session: AsyncSession = Depends(get_session)):
    domain = payload.domain or payload.domain_name
    user = await authenticate(session, payload.username, payload.password, domain)
    if not user:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid credentials")
    perms = await get_user_permissions(session, user)
    token = create_token(user, perms)
    return TokenResponse(access_token=token, token_type="Bearer")


@router.get("/me", response_model=UserRead)
async def me(user: User = Depends(get_current_user)):
    return UserRead(
        user_uuid=user.user_uuid,
        username=user.username,
        email=user.email,
        domain_id=user.domain_id,
        enabled=user.enabled,
    )


@router.post("/refresh", response_model=TokenResponse)
async def refresh(user: User = Depends(get_current_user), session: AsyncSession = Depends(get_session)):
    # refresh on group change: re-query permissions so JWT reflects current group assignments
    perms = await get_user_permissions(session, user)
    token = create_token(user, perms)
    return TokenResponse(access_token=token)
