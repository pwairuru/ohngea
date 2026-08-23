from typing import Optional
from uuid import UUID
import uuid6
from fastapi import APIRouter, Depends, HTTPException, Query, Response, Request
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user
from app.auth.models import User
from app.auth.service import get_user_permissions
from app.contacts.models import Contact
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1/contacts", tags=["contacts"])

@router.get("", response_model=dict)
async def list_contacts(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(50, ge=1, le=100),
    search: Optional[str] = Query(None),
    q: Optional[str] = Query(None),
    show: Optional[str] = Query(None),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "contact_view" in perms or "contact_add" in perms or "user_view" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: contact_view")
    show_all = (show == "all" and ("*" in perms or "contact_all" in perms or "user_all" in perms))
    q_search = q or search
    base = select(Contact)
    count_q = select(func.count()).select_from(Contact)
    if not show_all:
        base = base.where(Contact.domain_id == domain_id)
        count_q = count_q.where(Contact.domain_id == domain_id)
    if q_search:
        like = f"%{q_search}%"
        filt = or_(Contact.name.ilike(like), Contact.organization.ilike(like), Contact.email.ilike(like), Contact.number.ilike(like))
        base = base.where(filt)
        count_q = count_q.where(filt)
    total = (await session.execute(count_q)).scalar_one()
    base = base.order_by(Contact.name).offset((page-1)*per_page).limit(per_page)
    items = (await session.execute(base)).scalars().all()
    data = [{"contact_uuid": str(c.contact_uuid), "domain_id": str(c.domain_id), "name": c.name, "number": c.number, "email": c.email, "organization": c.organization, "created_at": c.created_at.isoformat() if c.created_at else None} for c in items]
    response.headers["X-Total-Count"] = str(total)
    return {"data": data, "total": total, "page": page, "per_page": per_page}

@router.post("", response_model=dict, status_code=201)
async def create_contact(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "contact_add" in perms or "user_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: contact_add")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    target_domain = payload.get("domain_id") or payload.get("domain_uuid") or str(domain_id)
    try:
        target_uuid = UUID(str(target_domain))
    except:
        raise HTTPException(status_code=422, detail="Invalid domain_id")
    name = payload.get("name") or payload.get("contact_name") or "unknown"
    number = payload.get("number") or payload.get("contact_number") or ""
    email = payload.get("email")
    org = payload.get("organization") or payload.get("contact_organization")
    c = Contact(contact_uuid=uuid6.uuid7(), domain_id=target_uuid, name=name, number=number, email=email, organization=org)
    session.add(c)
    await session.commit()
    await session.refresh(c)
    response.headers["Location"] = f"/api/v1/contacts/{c.contact_uuid}"
    return {"contact_uuid": str(c.contact_uuid), "name": c.name, "number": c.number, "email": c.email, "organization": c.organization}

@router.get("/{contact_uuid}", response_model=dict)
async def get_contact(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Contact).where(Contact.contact_uuid == contact_uuid))
    c = r.scalar_one_or_none()
    if not c:
        raise HTTPException(status_code=404, detail="Contact not found")
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "contact_view" in perms) and str(c.domain_id) != str(domain_id):
        raise HTTPException(status_code=404, detail="Contact not found")
    return {"contact_uuid": str(c.contact_uuid), "domain_id": str(c.domain_id), "name": c.name, "number": c.number, "email": c.email, "organization": c.organization}

@router.patch("/{contact_uuid}", response_model=dict)
async def patch_contact(contact_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "contact_edit" in perms or "user_edit" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: contact_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Contact).where(Contact.contact_uuid == contact_uuid))
    c = r.scalar_one_or_none()
    if not c:
        raise HTTPException(status_code=404, detail="Contact not found")
    for field in ["name", "number", "email", "organization"]:
        if field in payload:
            setattr(c, field, payload[field])
        # also support prefixed
        if f"contact_{field}" in payload:
            setattr(c, field, payload[f"contact_{field}"])
    await session.commit()
    await session.refresh(c)
    return {"contact_uuid": str(c.contact_uuid), "name": c.name, "number": c.number, "email": c.email, "organization": c.organization}

@router.delete("/{contact_uuid}", status_code=204)
async def delete_contact(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "contact_delete" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: contact_delete")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Contact).where(Contact.contact_uuid == contact_uuid))
    c = r.scalar_one_or_none()
    if not c:
        raise HTTPException(status_code=404, detail="Contact not found")
    await session.delete(c)
    await session.commit()
    return Response(status_code=204)
