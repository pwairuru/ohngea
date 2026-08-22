from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.auth.schemas import UserRead
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1/users", tags=["users"])


@router.get("")
async def list_users(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(User).where(User.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(User).where(User.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    data = [{"user_uuid": str(u.user_uuid), "username": u.username, "email": u.email, "enabled": u.enabled, "_links": {"self": {"href": f"/api/v1/users/{u.user_uuid}"}}} for u in r.scalars().all()]
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/users?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.get("/{user_uuid}", response_model=dict)
async def get_user(user_uuid: UUID, session: AsyncSession = Depends(get_session), current: User = Depends(get_current_user)):
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid == user_uuid, User.domain_id == domain_id))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    return {"user_uuid": str(u.user_uuid), "username": u.username, "email": u.email, "enabled": u.enabled, "_links": {"self": {"href": f"/api/v1/users/{u.user_uuid}"}}}


@router.post("", status_code=201)
async def create_user(payload: dict, response: Response, session: AsyncSession = Depends(get_session), current: User = Depends(require_permission("domain_add"))):
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    # check duplicate
    r = await session.execute(select(User).where(User.username == payload.get("username"), User.domain_id == domain_id))
    if r.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Username exists")
    from app.auth.service import hash_password
    u = User(domain_id=domain_id, username=payload["username"], email=payload.get("email"), password_hash=hash_password(payload.get("password", "password")), enabled=payload.get("enabled", True))
    session.add(u)
    await session.commit()
    await session.refresh(u)
    response.headers["Location"] = f"/api/v1/users/{u.user_uuid}"
    return {"user_uuid": str(u.user_uuid), "username": u.username, "email": u.email, "enabled": u.enabled, "_links": {"self": {"href": f"/api/v1/users/{u.user_uuid}"}}}


@router.patch("/{user_uuid}")
async def patch_user(user_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), current: User = Depends(require_permission("domain_add"))):
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid == user_uuid, User.domain_id == domain_id))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    for k in ["username", "email", "enabled"]:
        if k in payload:
            setattr(u, k, payload[k])
    if "password" in payload and payload["password"]:
        from app.auth.service import hash_password
        u.password_hash = hash_password(payload["password"])
    await session.commit()
    await session.refresh(u)
    return {"user_uuid": str(u.user_uuid), "username": u.username, "email": u.email, "enabled": u.enabled, "_links": {"self": {"href": f"/api/v1/users/{u.user_uuid}"}}}


@router.delete("/{user_uuid}", status_code=204)
async def delete_user(user_uuid: UUID, session: AsyncSession = Depends(get_session), current: User = Depends(require_permission("domain_add"))):
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid == user_uuid, User.domain_id == domain_id))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    await session.delete(u)
    await session.commit()
    return Response(status_code=204)
