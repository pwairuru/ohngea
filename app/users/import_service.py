import csv
import io
import re
from typing import List, Tuple, Dict, Any
from uuid import UUID
import uuid6
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.models import User, Group, UserGroup
from app.auth.service import hash_password
from app.contacts.models import Contact
from app.db import uuid7


def _validate_password_strength(password: str, length_req=12, need_number=True, need_lower=True, need_upper=True, need_special=False):
    errors = []
    if len(password) < length_req:
        errors.append(f"password must be at least {length_req} characters")
    if need_number and not re.search(r"\d", password):
        errors.append("password must contain at least one number")
    if need_lower and not re.search(r"[a-z]", password):
        errors.append("password must contain at least one lowercase")
    if need_upper and not re.search(r"[A-Z]", password):
        errors.append("password must contain at least one uppercase")
    if need_special and not re.search(r"[\W_]", password):
        errors.append("password must contain at least one special character")
    return errors


async def _validate_and_create_user(
    session: AsyncSession,
    domain_id: UUID,
    row: Dict[str, str],
    perms=None,
) -> Tuple[User | None, str | None]:
    """
    Helper to create single user from CSV row dict.
    Expected keys: username (required), password (required), email (optional),
    contact_organization, contact_name, number, enabled
    Returns (user, error)
    """
    username = (row.get("username") or row.get("user_name") or row.get("user") or "").strip()
    password = (row.get("password") or row.get("user_password") or "").strip()
    if not username:
        return None, "username required"
    if not password:
        return None, "password required"
    if len(username) < 2 or len(username) > 64:
        return None, "username must be 2-64 characters"
    # unique check per domain
    existing = await session.execute(select(User).where(User.username == username, User.domain_id == domain_id))
    if existing.scalar_one_or_none():
        return None, f"Username {username} exists"
    email = (row.get("email") or row.get("user_email") or row.get("contact_email") or "").strip() or None
    # basic email validation if provided
    if email and "@" not in email:
        return None, "invalid email"
    # password strength: reuse same validation as router (defaults 12/true/true/true/false) — no weak bypass
    pw_errors = _validate_password_strength(password)
    if pw_errors:
        return None, "; ".join(pw_errors)
    # create user
    enabled_val = row.get("enabled", row.get("user_enabled", "true"))
    if isinstance(enabled_val, str):
        enabled = enabled_val.lower() in ("true", "1", "yes", "t", "enabled")
    else:
        enabled = bool(enabled_val) if enabled_val is not None else True
    u = User(
        domain_id=domain_id,
        username=username,
        email=email,
        password_hash=hash_password(password),
        enabled=enabled,
    )
    # contact handling
    org = row.get("contact_organization") or row.get("organization")
    cname = row.get("contact_name") or row.get("name") or username
    cemail = row.get("contact_email") or email
    if org or cname != username:
        # create contact
        c_uuid = uuid6.uuid7()
        number = row.get("number") or row.get("contact_number") or ""
        c = Contact(contact_uuid=c_uuid, domain_id=domain_id, name=cname or username, number=number or "", email=cemail, organization=org)
        session.add(c)
        await session.flush()
        u.contact_uuid = c_uuid
    session.add(u)
    await session.flush()
    # group assignment if provided
    group_name = row.get("group_name") or row.get("group") or row.get("group_uuid_name")
    if group_name:
        # try lookup by name
        from sqlalchemy import or_
        # if UUID
        try:
            g_uuid = UUID(str(group_name).split("|")[0])
            # check if valid UUID lookup
            from sqlalchemy import select as sel
            r = await session.execute(sel(Group).where(Group.group_uuid == g_uuid))
            grp = r.scalar_one_or_none()
            if grp:
                ug = UserGroup(user_uuid=u.user_uuid, group_uuid=grp.group_uuid, domain_id=domain_id, group_name=grp.name)
                session.add(ug)
        except Exception:
            # treat as name
            r = await session.execute(select(Group).where(Group.name == group_name, Group.domain_id == domain_id))
            grp = r.scalar_one_or_none()
            if grp:
                ug = UserGroup(user_uuid=u.user_uuid, group_uuid=grp.group_uuid, domain_id=domain_id, group_name=grp.name)
                session.add(ug)
    return u, None


def parse_csv_content(csv_text: str) -> List[Dict[str, str]]:
    """Parse CSV text, return list of dicts. Handles header row."""
    f = io.StringIO(csv_text.strip())
    # sniff dialect? default excel
    try:
        reader = csv.DictReader(f)
        if reader.fieldnames is None:
            return []
        # normalize fieldnames lower
        normalized = []
        for row in reader:
            norm = {k.strip().lower(): (v.strip() if isinstance(v, str) else v) for k, v in row.items() if k}
            # also keep original case variants
            normalized.append(norm)
        return normalized
    except Exception:
        return []


async def import_users_from_csv(
    session: AsyncSession,
    domain_id: UUID,
    csv_text: str,
) -> Dict[str, Any]:
    """
    Parse CSV and import users.
    Returns dict with imported count, errors, created users.
    """
    rows = parse_csv_content(csv_text)
    if not rows:
        return {"imported": 0, "errors": [{"row": 0, "error": "empty or invalid CSV"}], "data": []}
    imported = 0
    errors: List[Dict[str, Any]] = []
    created: List[Dict[str, str]] = []
    for idx, row in enumerate(rows, start=2):  # header is row 1
        # skip empty rows
        if not any(v for v in row.values() if v):
            continue
        try:
            user, err = await _validate_and_create_user(session, domain_id, row)
            if err:
                errors.append({"row": idx, "error": err, "data": row})
            else:
                imported += 1
                created.append({"user_uuid": str(user.user_uuid), "username": user.username, "email": user.email})
        except Exception as e:
            errors.append({"row": idx, "error": str(e), "data": row})
    await session.commit()
    return {"imported": imported, "errors": errors, "data": created, "total": imported}
