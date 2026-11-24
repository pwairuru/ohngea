from typing import Optional
from uuid import UUID
import uuid6
from fastapi import APIRouter, Depends, HTTPException, Query, Response, Request
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user
from app.auth.models import User
from app.auth.service import get_user_permissions
from app.contacts.models import (
    Contact,
    ContactAddress,
    ContactPhone,
    ContactEmail,
    ContactUrl,
    ContactNote,
    ContactAttachment,
    ContactGroup,
    ContactUser,
    ContactSetting,
    ContactRelation,
    ContactTime,
)
from app.db import get_session, set_tenant

router = APIRouter(prefix="/api/v1/contacts", tags=["contacts"])

# helpers
def _contact_to_dict(c: Contact) -> dict:
    base = {
        "contact_uuid": str(c.contact_uuid),
        "domain_id": str(c.domain_id),
        "domain_uuid": str(c.domain_id),
        "name": c.name,
        "number": c.number,
        "email": c.email,
        "organization": c.organization,
        "contact_organization": c.organization,
        "created_at": c.created_at.isoformat() if c.created_at else None,
    }
    # legacy extended fields
    for f in [
        "contact_type",
        "contact_parent_uuid",
        "contact_name_prefix",
        "contact_name_given",
        "contact_name_middle",
        "contact_name_family",
        "contact_name_suffix",
        "contact_nickname",
        "contact_title",
        "contact_role",
        "contact_category",
        "contact_url",
        "contact_time_zone",
        "contact_note",
    ]:
        v = getattr(c, f, None)
        if isinstance(v, UUID):
            v = str(v)
        elif hasattr(v, "isoformat"):
            try:
                v = v.isoformat()
            except Exception:
                pass
        base[f] = v
    return base

async def _get_contact_or_404(session: AsyncSession, contact_uuid: UUID) -> Contact:
    r = await session.execute(select(Contact).where(Contact.contact_uuid == contact_uuid))
    c = r.scalar_one_or_none()
    if not c:
        raise HTTPException(status_code=404, detail="Contact not found")
    return c

def _has_perm(perms, *names) -> bool:
    return "*" in perms or any(n in perms for n in names)

CONTACT_FIELDS = {c.name for c in Contact.__table__.columns} - {"contact_uuid", "created_at", "domain_id"}

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
    if not _has_perm(perms, "contact_view", "contact_add", "user_view"):
        raise HTTPException(status_code=403, detail="Missing permission: contact_view")
    show_all = (show == "all" and _has_perm(perms, "contact_all", "user_all"))
    q_search = q or search
    base = select(Contact)
    count_q = select(func.count()).select_from(Contact)
    if not show_all:
        base = base.where(Contact.domain_id == domain_id)
        count_q = count_q.where(Contact.domain_id == domain_id)
    if q_search:
        like = f"%{q_search}%"
        filt = or_(
            Contact.name.ilike(like),
            Contact.organization.ilike(like),
            Contact.email.ilike(like),
            Contact.number.ilike(like),
            Contact.contact_name_given.ilike(like),
            Contact.contact_name_family.ilike(like),
            Contact.contact_nickname.ilike(like),
        )
        base = base.where(filt)
        count_q = count_q.where(filt)
    total = (await session.execute(count_q)).scalar_one()
    base = base.order_by(Contact.name).offset((page-1)*per_page).limit(per_page)
    items = (await session.execute(base)).scalars().all()
    data = [_contact_to_dict(c) for c in items]
    response.headers["X-Total-Count"] = str(total)
    return {"data": data, "total": total, "page": page, "per_page": per_page}

@router.post("", response_model=dict, status_code=201)
async def create_contact(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not _has_perm(perms, "contact_add", "user_add"):
        raise HTTPException(status_code=403, detail="Missing permission: contact_add")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    target_domain = payload.get("domain_id") or payload.get("domain_uuid") or str(domain_id)
    try:
        target_uuid = UUID(str(target_domain))
    except Exception:
        raise HTTPException(status_code=422, detail="Invalid domain_id")
    # base fields
    name = payload.get("name") or payload.get("contact_name") or payload.get("contact_name_given") or "unknown"
    # if given+family provided, combine for name fallback
    if (payload.get("contact_name_given") or payload.get("contact_name_family")) and not payload.get("name"):
        given = payload.get("contact_name_given") or ""
        family = payload.get("contact_name_family") or ""
        combined = f"{given} {family}".strip()
        if combined:
            name = combined
    number = payload.get("number") or payload.get("contact_number") or ""
    email = payload.get("email") or payload.get("contact_email") or payload.get("email_address")
    org = payload.get("organization") or payload.get("contact_organization")
    c = Contact(contact_uuid=uuid6.uuid7(), domain_id=target_uuid, name=name, number=number, email=email, organization=org)
    # set extended fields if present
    for f in CONTACT_FIELDS:
        if f in payload:
            val = payload[f]
            # handle UUID field
            if f == "contact_parent_uuid" and val is not None:
                try:
                    val = UUID(str(val))
                except Exception:
                    raise HTTPException(status_code=422, detail="Invalid contact_parent_uuid")
            setattr(c, f, val)
        # also allow alias without prefix for some? already handled
    # handle contact_organization alias to organization if not already set
    if "contact_organization" in payload and not org:
        c.organization = payload["contact_organization"]
    session.add(c)
    await session.commit()
    await session.refresh(c)
    response.headers["Location"] = f"/api/v1/contacts/{c.contact_uuid}"
    return _contact_to_dict(c)

@router.get("/{contact_uuid}", response_model=dict)
async def get_contact(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    c = await _get_contact_or_404(session, contact_uuid)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not _has_perm(perms, "contact_view") and str(c.domain_id) != str(domain_id):
        raise HTTPException(status_code=404, detail="Contact not found")
    return _contact_to_dict(c)

@router.patch("/{contact_uuid}", response_model=dict)
async def patch_contact(contact_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not _has_perm(perms, "contact_edit", "user_edit"):
        raise HTTPException(status_code=403, detail="Missing permission: contact_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    c = await _get_contact_or_404(session, contact_uuid)
    for field in ["name", "number", "email", "organization"]:
        if field in payload:
            setattr(c, field, payload[field])
        if f"contact_{field}" in payload:
            setattr(c, field, payload[f"contact_{field}"])
    # extended fields
    for f in CONTACT_FIELDS:
        if f in payload:
            val = payload[f]
            if f == "contact_parent_uuid" and val is not None:
                try:
                    val = UUID(str(val))
                except Exception:
                    raise HTTPException(status_code=422, detail="Invalid contact_parent_uuid")
            setattr(c, f, val)
    # alias sync: if contact_organization changed, keep organization sync
    if "contact_organization" in payload:
        c.organization = payload["contact_organization"]
    await session.commit()
    await session.refresh(c)
    return _contact_to_dict(c)

@router.delete("/{contact_uuid}", status_code=204)
async def delete_contact(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not _has_perm(perms, "contact_delete"):
        raise HTTPException(status_code=403, detail="Missing permission: contact_delete")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    c = await _get_contact_or_404(session, contact_uuid)
    await session.delete(c)
    await session.commit()
    return Response(status_code=204)

# --- child CRUD helpers ---

def _child_to_dict(obj) -> dict:
    d = {}
    for col in obj.__table__.columns:
        v = getattr(obj, col.name)
        if isinstance(v, UUID):
            v = str(v)
        elif hasattr(v, "isoformat") and v is not None:
            try:
                v = v.isoformat()
            except Exception:
                pass
        d[col.name] = v
        # alias domain_uuid
        if col.name == "domain_id":
            d["domain_uuid"] = str(v) if v else None
    return d

async def _list_child(session, domain_id, contact_uuid, model, perms, view_perms):
    await set_tenant(session, domain_id)
    if not _has_perm(perms, *view_perms):
        raise HTTPException(status_code=403, detail=f"Missing permission: {view_perms[0]}")
    c = await _get_contact_or_404(session, contact_uuid)
    # domain check (RLS already but also check)
    if str(c.domain_id) != str(domain_id) and not _has_perm(perms, "contact_all"):
        raise HTTPException(status_code=404, detail="Contact not found")
    r = await session.execute(select(model).where(model.contact_uuid == contact_uuid, model.domain_id == c.domain_id).order_by(model.created_at)) if hasattr(model, "created_at") else await session.execute(select(model).where(model.contact_uuid == contact_uuid, model.domain_id == c.domain_id))
    items = r.scalars().all()
    return [_child_to_dict(x) for x in items]

async def _create_child(session, domain_id, contact_uuid, model, payload: dict, perms, add_perms, required_fields=None):
    await set_tenant(session, domain_id)
    if not _has_perm(perms, *add_perms):
        raise HTTPException(status_code=403, detail=f"Missing permission: {add_perms[0]}")
    c = await _get_contact_or_404(session, contact_uuid)
    if str(c.domain_id) != str(domain_id) and not _has_perm(perms, "contact_all"):
        raise HTTPException(status_code=404, detail="Contact not found")
    # validate required
    if required_fields:
        for f in required_fields:
            if f not in payload or payload[f] in (None, ""):
                raise HTTPException(status_code=422, detail=f"{f} required")
    # build obj
    allowed = {col.name for col in model.__table__.columns} - {"created_at"}
    data = {}
    for k, v in payload.items():
        if k in allowed:
            data[k] = v
    # enforce FKs
    data["domain_id"] = c.domain_id
    data["contact_uuid"] = c.contact_uuid
    # generate pk if missing
    pk_col = [col.name for col in model.__table__.columns if col.primary_key][0]
    if pk_col not in data or not data[pk_col]:
        data[pk_col] = uuid6.uuid7()
    else:
        try:
            data[pk_col] = UUID(str(data[pk_col]))
        except Exception:
            data[pk_col] = uuid6.uuid7()
    obj = model(**data)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return _child_to_dict(obj)

# Addresses
@router.get("/{contact_uuid}/addresses", response_model=dict)
async def list_addresses(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    data = await _list_child(session, domain_id, contact_uuid, ContactAddress, perms, ["contact_address_view", "contact_view"])
    return {"data": data, "total": len(data)}

@router.post("/{contact_uuid}/addresses", response_model=dict, status_code=201)
async def create_address(contact_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    obj = await _create_child(session, domain_id, contact_uuid, ContactAddress, payload, perms, ["contact_address_add", "contact_add"])
    return obj

# Phones
@router.get("/{contact_uuid}/phones", response_model=dict)
async def list_phones(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    data = await _list_child(session, domain_id, contact_uuid, ContactPhone, perms, ["contact_phone_view", "contact_view"])
    return {"data": data, "total": len(data)}

@router.post("/{contact_uuid}/phones", response_model=dict, status_code=201)
async def create_phone(contact_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    obj = await _create_child(session, domain_id, contact_uuid, ContactPhone, payload, perms, ["contact_phone_add", "contact_add"], required_fields=["phone_number"])
    return obj

# Emails
@router.get("/{contact_uuid}/emails", response_model=dict)
async def list_emails(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    data = await _list_child(session, domain_id, contact_uuid, ContactEmail, perms, ["contact_email_view", "contact_view"])
    return {"data": data, "total": len(data)}

@router.post("/{contact_uuid}/emails", response_model=dict, status_code=201)
async def create_email(contact_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    # allow email_address or email
    if "email" in payload and "email_address" not in payload:
        payload["email_address"] = payload["email"]
    obj = await _create_child(session, domain_id, contact_uuid, ContactEmail, payload, perms, ["contact_email_add", "contact_add"], required_fields=["email_address"])
    return obj

# URLs
@router.get("/{contact_uuid}/urls", response_model=dict)
async def list_urls(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    data = await _list_child(session, domain_id, contact_uuid, ContactUrl, perms, ["contact_url_view", "contact_view"])
    return {"data": data, "total": len(data)}

@router.post("/{contact_uuid}/urls", response_model=dict, status_code=201)
async def create_url(contact_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if "url" in payload and "url_address" not in payload:
        payload["url_address"] = payload["url"]
    obj = await _create_child(session, domain_id, contact_uuid, ContactUrl, payload, perms, ["contact_url_add", "contact_add"], required_fields=["url_address"])
    return obj

# Notes
@router.get("/{contact_uuid}/notes", response_model=dict)
async def list_notes(contact_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    data = await _list_child(session, domain_id, contact_uuid, ContactNote, perms, ["contact_note_view", "contact_view"])
    return {"data": data, "total": len(data)}

@router.post("/{contact_uuid}/notes", response_model=dict, status_code=201)
async def create_note(contact_uuid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    # allow note / contact_note alias
    if "note" in payload and "contact_note" not in payload:
        payload["contact_note"] = payload["note"]
    if "notes" in payload and "contact_note" not in payload:
        payload["contact_note"] = payload["notes"]
    obj = await _create_child(session, domain_id, contact_uuid, ContactNote, payload, perms, ["contact_note_add", "contact_add"], required_fields=["contact_note"])
    return obj


# --- generic child delete (phones/addresses/emails/urls/notes) ---
_CHILD_MODELS = {
    "phones": (ContactPhone, "contact_phone_uuid", ["contact_phone_delete", "contact_delete"]),
    "addresses": (ContactAddress, "contact_address_uuid", ["contact_address_delete", "contact_delete"]),
    "emails": (ContactEmail, "contact_email_uuid", ["contact_email_delete", "contact_delete"]),
    "urls": (ContactUrl, "contact_url_uuid", ["contact_url_delete", "contact_delete"]),
    "notes": (ContactNote, "contact_note_uuid", ["contact_note_delete", "contact_delete"]),
}


@router.delete("/{contact_uuid}/{child}/{child_id}", status_code=204)
async def delete_child(contact_uuid: UUID, child: str, child_id: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    if child not in _CHILD_MODELS:
        raise HTTPException(status_code=404, detail="Unknown child")
    model, pk, del_perms = _CHILD_MODELS[child]
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    await set_tenant(session, domain_id)
    if not _has_perm(perms, *del_perms):
        raise HTTPException(status_code=403, detail=f"Missing permission: {del_perms[0]}")
    c = await _get_contact_or_404(session, contact_uuid)
    if str(c.domain_id) != str(domain_id) and not _has_perm(perms, "contact_all"):
        raise HTTPException(status_code=404, detail="Contact not found")
    r = await session.execute(select(model).where(getattr(model, pk) == child_id, model.contact_uuid == contact_uuid))
    obj = r.scalar_one_or_none()
    if not obj:
        raise HTTPException(status_code=404, detail="Not found")
    await session.delete(obj)
    await session.commit()
    return Response(status_code=204)


# --- extra children: attachments, groups, users, settings, relations, times ---
_EXTRA_CHILDREN = {
    "attachments": (ContactAttachment, "contact_attachment_uuid", ["contact_attachment_view", "contact_view"], ["contact_attachment_add", "contact_add"], ["contact_attachment_delete", "contact_delete"], ["attachment_filename"]),
    "groups": (ContactGroup, "contact_group_uuid", ["contact_group_view", "contact_view"], ["contact_group_add", "contact_add"], ["contact_group_delete", "contact_delete"], ["group_uuid"]),
    "users": (ContactUser, "contact_user_uuid", ["contact_user_view", "contact_view"], ["contact_user_add", "contact_add"], ["contact_user_delete", "contact_delete"], ["user_uuid"]),
    "settings": (ContactSetting, "contact_setting_uuid", ["contact_setting_view", "contact_view"], ["contact_setting_add", "contact_add"], ["contact_setting_delete", "contact_delete"], ["contact_setting_name"]),
    "relations": (ContactRelation, "contact_relation_uuid", ["contact_relation_view", "contact_view"], ["contact_relation_add", "contact_add"], ["contact_relation_delete", "contact_delete"], ["relation_contact_uuid"]),
    "times": (ContactTime, "contact_time_uuid", ["contact_time_view", "contact_view"], ["contact_time_add", "contact_add"], ["contact_time_delete", "contact_delete"], []),
}


@router.get("/{contact_uuid}/{child}", response_model=dict)
async def list_extra_child(contact_uuid: UUID, child: str, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    if child not in _EXTRA_CHILDREN:
        raise HTTPException(status_code=404, detail="Unknown child")
    model, pk, view_perms, _, _, _ = _EXTRA_CHILDREN[child]
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    data = await _list_child(session, domain_id, contact_uuid, model, perms, view_perms)
    return {"data": data, "total": len(data)}


@router.post("/{contact_uuid}/{child}", response_model=dict, status_code=201)
async def create_extra_child(contact_uuid: UUID, child: str, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    if child not in _EXTRA_CHILDREN:
        raise HTTPException(status_code=404, detail="Unknown child")
    model, pk, _, add_perms, _, required = _EXTRA_CHILDREN[child]
    domain_id = getattr(user, "_domain_id", user.domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    obj = await _create_child(session, domain_id, contact_uuid, model, payload, perms, add_perms, required_fields=required or None)
    return obj


@router.delete("/{contact_uuid}/{child}/{child_id}", status_code=204)
async def delete_extra_child(contact_uuid: UUID, child: str, child_id: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    if child not in _EXTRA_CHILDREN:
        raise HTTPException(status_code=404, detail="Unknown child")
    model, pk, _, _, del_perms, _ = _EXTRA_CHILDREN[child]
    # reuse generic delete path
    _CHILD_MODELS[child] = (model, pk, del_perms)
    try:
        return await delete_child(contact_uuid, child, child_id, session, user)
    finally:
        del _CHILD_MODELS[child]
