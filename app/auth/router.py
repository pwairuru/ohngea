from fastapi import APIRouter, Depends, HTTPException, Request, Response, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.schemas import LoginRequest, TokenResponse, UserRead
from app.auth.service import authenticate, create_token, get_user_permissions
from app.auth.deps import get_current_user
from app.auth.models import User
from app.config import settings
from app.db import get_session

router = APIRouter(prefix="/auth", tags=["auth"])

_COOKIE_NAME = "access_token"


def _set_auth_cookie(resp: Response, token: str) -> None:
    secure = settings.env.lower() in ("prod", "production")
    resp.set_cookie(
        _COOKIE_NAME,
        token,
        max_age=settings.jwt_expire_minutes * 60,
        httponly=True,
        secure=secure,
        samesite="lax",
        path="/",
    )


@router.post("/login", response_model=TokenResponse)
async def login(payload: LoginRequest, request: Request, response: Response, session: AsyncSession = Depends(get_session)):
    from app.middleware.rate_limit import is_locked, record_failure, record_success
    import logging
    log = logging.getLogger("app.auth")
    if is_locked(payload.username):
        raise HTTPException(status_code=429, detail="Account locked — try later")
    domain = payload.domain or payload.domain_name
    user = await authenticate(session, payload.username, payload.password, domain)
    if not user:
        record_failure(payload.username)
        log.warning("failed login username=%s domain=%s ip=%s", payload.username, domain, request.client.host if request.client else None)
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid credentials")
    record_success(payload.username)
    perms = await get_user_permissions(session, user)
    token = create_token(user, perms)
    _set_auth_cookie(response, token)
    try:
        from app.admin_settings.router import log_user_action

        ip = request.client.host if request.client else None
        await log_user_action(session, user.domain_id, user.user_uuid, user.username, "login", domain, ip)
    except Exception as e:
        log.warning("log_user_action failed: %s", e, exc_info=True)
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
async def refresh(response: Response, user: User = Depends(get_current_user), session: AsyncSession = Depends(get_session)):
    perms = await get_user_permissions(session, user)
    token = create_token(user, perms)
    _set_auth_cookie(response, token)
    return TokenResponse(access_token=token)


@router.post("/logout", response_model=dict)
async def logout(request: Request, response: Response, user: User = Depends(get_current_user), session: AsyncSession = Depends(get_session)):
    import logging
    try:
        from app.admin_settings.router import log_user_action

        ip = request.client.host if request.client else None
        await log_user_action(session, user.domain_id, user.user_uuid, user.username, "logout", None, ip)
    except Exception as e:
        logging.getLogger("app.auth").warning("logout log failed: %s", e, exc_info=True)
    response.delete_cookie(_COOKIE_NAME, path="/")
    return {"status": "logged out"}
