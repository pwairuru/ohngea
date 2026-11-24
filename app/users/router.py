from uuid import UUID
import logging
import re
import uuid6
from typing import Optional, List
from fastapi import APIRouter, Depends, HTTPException, Query, Response, UploadFile, File, Request
from sqlalchemy import select, func, or_, and_, text, update, delete
from sqlalchemy.exc import SQLAlchemyError
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user
from app.auth.models import User, Group, UserGroup, UserSetting, Permission, GroupPermission
from app.auth.service import hash_password, get_user_permissions
from app.contacts.models import Contact
from app.db import get_session, set_tenant
from app.domains.models import Domain, DomainSettings
from app.freeswitch.cache import invalidate_fs_cache_for_session
from app.groups.service import _is_superadmin

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/api/v1/users", tags=["users"])

async def get_user_level(session, user) -> int:
    from app.groups.service import get_user_level as gul
    return await gul(session, user)

async def get_user_group_uuids(session, user):
    from app.groups.service import get_user_group_uuids as gugu
    return await gugu(session, user)

async def _get_domain_setting(session: AsyncSession, domain_id: UUID, category: str, subcategory: str, default=None):
    try:
        # build variants for hyphen/underscore interchange (brief uses password-length, legacy may use password_length)
        variants = {subcategory}
        if "-" in subcategory:
            variants.add(subcategory.replace("-", "_"))
        if "_" in subcategory:
            variants.add(subcategory.replace("_", "-"))
        for var in variants:
            r = await session.execute(select(DomainSettings).where(DomainSettings.domain_id == domain_id, DomainSettings.category == category, DomainSettings.subcategory == var))
            row = r.scalar_one_or_none()
            if row:
                return row.value
        for var in variants:
            r2 = await session.execute(select(DomainSettings).where(DomainSettings.domain_id == domain_id, DomainSettings.category == category, DomainSettings.key == var))
            row2 = r2.scalar_one_or_none()
            if row2:
                return row2.value
        # fall back to global default_settings (legacy default → domain override chain)
        try:
            from app.admin_settings.models import DefaultSetting

            for var in variants:
                r3 = await session.execute(select(DefaultSetting).where(DefaultSetting.default_setting_category == category, DefaultSetting.default_setting_name == var))
                row3 = r3.scalar_one_or_none()
                if row3 and row3.default_setting_enabled:
                    return row3.default_setting_value
        except Exception:
            pass
    except SQLAlchemyError as e:
        logger.warning(f"_get_domain_setting failed {category}/{subcategory} for {domain_id}: {e}")
        pass
    return default

def _is_valid_email(email: str) -> bool:
    if not email or not isinstance(email, str):
        return False
    email = email.strip().lower()
    if len(email) > 254 or email.count("@") != 1:
        return False
    try:
        from email_validator import validate_email
        validate_email(email, check_deliverability=False)
        return True
    except Exception:
        return bool(re.match(r"^[^@\s]+@[^@\s]+\.[^@\s]+$", email))

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

_USERNAME_REGEX = re.compile(r"^[a-z0-9_-]{2,64}$")

def user_links(uid: UUID):
    base = f"/api/v1/users/{uid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/users"}, "groups": {"href": f"{base}/groups"}, "settings": {"href": f"{base}/settings"}}

async def enrich_user(session, u: User, domain_name_map=None):
    # group_names
    r = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==u.user_uuid))
    group_names = ",".join([row[0] for row in r.all()]) if r else ""
    # max group_level
    r2 = await session.execute(select(func.max(Group.group_level)).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==u.user_uuid))
    group_level = r2.scalar_one_or_none()
    # contact info
    contact_org = None
    contact_name = None
    contact_note = None
    if u.contact_uuid:
        cr = await session.execute(select(Contact).where(Contact.contact_uuid==u.contact_uuid))
        c = cr.scalar_one_or_none()
        if c:
            contact_org = c.organization
            contact_name = c.name
            contact_note = getattr(c, "email", None)
    # domain name
    domain_name = None
    if domain_name_map and str(u.domain_id) in domain_name_map:
        domain_name = domain_name_map[str(u.domain_id)]
    else:
        dr = await session.execute(select(Domain.domain_name).where(Domain.domain_id==u.domain_id))
        domain_name = dr.scalar_one_or_none()
    # user_settings (language/time_zone)
    sr = await session.execute(select(UserSetting).where(UserSetting.user_uuid==u.user_uuid, UserSetting.user_setting_enabled==True))
    settings_list = sr.scalars().all()
    lang = None
    tz = None
    for s in settings_list:
        if s.user_setting_category=="domain" and s.user_setting_subcategory=="language":
            lang = s.user_setting_value
        if s.user_setting_category=="domain" and s.user_setting_subcategory=="time_zone":
            tz = s.user_setting_value
    return {
        "user_uuid": str(u.user_uuid),
        "username": u.username,
        "email": u.email,
        "enabled": u.enabled,
        "user_enabled": "true" if u.enabled else "false",
        "domain_id": str(u.domain_id),
        "domain_uuid": str(u.domain_id),
        "domain_name": domain_name,
        "group_names": group_names,
        "group_level": group_level,
        "contact_uuid": str(u.contact_uuid) if u.contact_uuid else None,
        "contact_organization": contact_org,
        "contact_name": contact_name,
        "contact_note": contact_note,
        "user_status": u.user_status,
        "user_type": u.user_type,
        "api_key": u.api_key,
        "user_totp_secret": u.user_totp_secret,
        "user_language": lang,
        "user_time_zone": tz,
        "_links": user_links(u.user_uuid),
        "created_at": u.created_at.isoformat() if u.created_at else None,
    }

@router.get("")
async def list_users(
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(50, ge=1, le=100),
    search: Optional[str] = Query(None),
    q: Optional[str] = Query(None),
    show: Optional[str] = Query(None),
    order_by: str = Query("username", pattern="^(username|group_names|contact_organization|contact_name|contact_note|user_enabled|domain_name)$"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user)
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    perms = getattr(user, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "user_view" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_view")
    show_all = (show=="all" and ("*" in perms or "user_all" in perms))
    user_level = await get_user_level(session, user)
    query_q = q or search
    # base query: filter by group_level <= user_level OR null
    # need to join? For level filtering, we need to consider max group level per user
    # Instead filter users where max group_level <= user_level
    # Simplified: fetch all then filter in python? For performance ok small.
    # We'll fetch candidates by domain filter first then filter level
    base_where = []
    if not show_all:
        base_where.append(or_(User.domain_id==domain_id, User.domain_id==None))
    # search handling deferred after join?
    # get counts first with filters applied excluding level
    # For paginated counts we need to filter after level check, so we do two-step
    # First fetch IDs that satisfy search + domain
    # Build subquery for users
    user_q = select(User)
    if not show_all:
        user_q = user_q.where(or_(User.domain_id==domain_id, User.domain_id==None))
    # we will apply search after enrichment? but for DB search we need contact and group names
    # For now apply username/email search directly on users table
    count_q = select(func.count()).select_from(User)
    if not show_all:
        count_q = count_q.where(or_(User.domain_id==domain_id, User.domain_id==None))
    # search is handled via post-filtering for username/email/group_names/contact to support org/name like legacy users.php:128
    # keep DB query unfiltered for search, then filter in python for full parity (username, group_names, organization, name, note)
    # (legacy searches all those fields, so DB pre-filter would miss org/name)
    # fetch all matching users (without pagination) to apply level filter then paginate
    # For efficiency, fetch all then filter level, then paginate slice
    all_r = await session.execute(user_q.order_by(User.username))
    all_users = all_r.scalars().all()
    # filter by group_level
    filtered = []
    for u in all_users:
        # get max level
        r = await session.execute(select(func.max(Group.group_level)).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==u.user_uuid))
        max_lvl = r.scalar_one_or_none()
        if max_lvl is None or max_lvl <= user_level:
            filtered.append(u)
        else:
            # if user has no groups, allow? legacy shows users where group_level is null also visible
            # need to check if user has no groups, then visible
            # already max_lvl None means no groups -> visible
            # else high level -> hide
            pass
    # apply additional search on group_names/contact if query_q and not already filtered via username
    if query_q:
        # further filter filtered list by group_names/contact_organization/name
        final = []
        qq = query_q.lower()
        for u in filtered:
            # fetch enriched quickly
            gr = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==u.user_uuid))
            gnames = ",".join([r[0] for r in gr.all()]).lower()
            corg = ""
            cname = ""
            if u.contact_uuid:
                cr = await session.execute(select(Contact).where(Contact.contact_uuid==u.contact_uuid))
                c = cr.scalar_one_or_none()
                if c:
                    corg = (c.organization or "").lower()
                    cname = (c.name or "").lower()
            if qq in (u.username or "").lower() or qq in (u.email or "").lower() or qq in gnames or qq in corg or qq in cname:
                final.append(u)
        filtered = final
    total = len(filtered)
    # order
    reverse = (order=="desc")
    if order_by=="username":
        filtered.sort(key=lambda x: (x.username or "").lower(), reverse=reverse)
    elif order_by=="domain_name":
        # need domain names
        dmap = {}
        for u in filtered:
            if str(u.domain_id) not in dmap:
                dr = await session.execute(select(Domain.domain_name).where(Domain.domain_id==u.domain_id))
                dmap[str(u.domain_id)] = dr.scalar_one_or_none() or ""
        filtered.sort(key=lambda x: dmap.get(str(x.domain_id),"").lower(), reverse=reverse)
    elif order_by=="user_enabled":
        filtered.sort(key=lambda x: x.enabled, reverse=reverse)
    # paging
    offset = (page-1)*per_page
    paged = filtered[offset: offset+per_page]
    # enrich
    # prefetch domain names
    dmap = {}
    for u in paged:
        if str(u.domain_id) not in dmap:
            dr = await session.execute(select(Domain.domain_name).where(Domain.domain_id==u.domain_id))
            dmap[str(u.domain_id)] = dr.scalar_one_or_none()
    data = [await enrich_user(session, u, dmap) for u in paged]
    response.headers["X-Total-Count"] = str(total)
    if offset + per_page < total:
        response.headers["Link"] = f'</api/v1/users?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}

@router.post("/bulk", response_model=dict)
async def bulk_users(payload: dict, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    perms = getattr(current, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, current)
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    action = payload.get("action")
    ids = payload.get("ids") or payload.get("users") or []
    # normalize ids: list of {uuid, checked} or list of uuids
    uuids = []
    for item in ids:
        if isinstance(item, dict):
            if item.get("checked")=="true" or item.get("checked") is True:
                uuids.append(item.get("uuid") or item.get("user_uuid"))
            elif "uuid" in item and "checked" not in item:
                uuids.append(item["uuid"])
        else:
            uuids.append(item)
    results=[]
    for uid_str in uuids:
        try:
            uid = UUID(str(uid_str))
        except Exception:
            results.append({"user_uuid": str(uid_str), "error": "invalid uuid"})
            continue
        r = await session.execute(select(User).where(User.user_uuid==uid))
        u = r.scalar_one_or_none()
        if not u:
            results.append({"user_uuid": str(uid_str), "error": "not found"})
            continue
        # domain check
        if not ("*" in perms or "user_all" in perms) and str(u.domain_id)!=str(domain_id):
            results.append({"user_uuid": str(uid_str), "error": "domain mismatch"})
            continue
        if action=="copy":
            if not ("*" in perms or "user_add" in perms or "domain_add" in perms):
                results.append({"user_uuid": str(uid_str), "error": "Missing permission: user_add"})
                continue
            # check limit
            # copy user with new uuid, username + " (copy)"
            new_uuid = uuid6.uuid7()
            base_name = u.username + " (copy)"
            # ensure unique
            check = await session.execute(select(User).where(User.username==base_name, User.domain_id==u.domain_id))
            cnt=1
            new_name = base_name
            while check.scalar_one_or_none():
                cnt+=1
                new_name = f"{u.username} (copy {cnt})"
                check = await session.execute(select(User).where(User.username==new_name, User.domain_id==u.domain_id))
            new_u = User(user_uuid=new_uuid, domain_id=u.domain_id, username=new_name, email=u.email, password_hash=u.password_hash, enabled=u.enabled, contact_uuid=u.contact_uuid, user_status=u.user_status, user_type=u.user_type, api_key=None, user_totp_secret=None)
            session.add(new_u)
            await session.flush()
            # copy groups
            gr = await session.execute(select(UserGroup).where(UserGroup.user_uuid==uid))
            for ug in gr.scalars().all():
                new_ug = UserGroup(user_uuid=new_uuid, group_uuid=ug.group_uuid, domain_id=ug.domain_id, group_name=ug.group_name)
                session.add(new_ug)
            # copy settings
            sr = await session.execute(select(UserSetting).where(UserSetting.user_uuid==uid))
            for s in sr.scalars().all():
                ns = UserSetting(user_uuid=new_uuid, domain_id=s.domain_id, user_setting_category=s.user_setting_category, user_setting_subcategory=s.user_setting_subcategory, user_setting_name=s.user_setting_name, user_setting_value=s.user_setting_value, user_setting_enabled=s.user_setting_enabled, user_setting_description=s.user_setting_description)
                session.add(ns)
            await session.commit()
            results.append({"user_uuid": str(new_uuid), "action": "copied", "username": new_name})
        elif action=="toggle":
            if not ("*" in perms or "user_edit" in perms or "domain_add" in perms):
                results.append({"user_uuid": str(uid_str), "error": "Missing permission: user_edit"})
                continue
            u.enabled = not u.enabled
            await session.commit()
            results.append({"user_uuid": str(uid_str), "action": "toggled", "enabled": u.enabled})
        elif action=="delete":
            if not ("*" in perms or "user_delete" in perms or "domain_add" in perms):
                results.append({"user_uuid": str(uid_str), "error": "Missing permission: user_delete"})
                continue
            # superadmin protection: * means superadmin, no lvl fallback
            # check if user is superadmin member
            gr = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==uid))
            is_super = any(row[0]=="superadmin" for row in gr.all())
            if is_super:
                # check if current is superadmin
                cr = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==current.user_uuid))
                is_cur_super = any(row[0]=="superadmin" for row in cr.all()) or "*" in perms
                if not is_cur_super:
                    results.append({"user_uuid": str(uid_str), "error": "Only superadmin can delete superadmin"})
                    continue
            await session.delete(u)
            await session.commit()
            results.append({"user_uuid": str(uid_str), "action": "deleted"})
        else:
            results.append({"user_uuid": str(uid_str), "error": "Invalid action"})
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return {"data": results, "total": len(results)}

@router.post("/import", response_model=dict)
async def import_users_endpoint(
    request: Request,
    session: AsyncSession = Depends(get_session),
    current: User = Depends(get_current_user),
):
    perms = getattr(current, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, current)
    if not ("*" in perms or "user_add" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_add")
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    # support both JSON {"csv": "..."} and multipart file upload
    content_type = request.headers.get("content-type", "")
    csv_text = None
    # try multipart
    if "multipart/form-data" in content_type:
        try:
            form = await request.form()
            # look for file field
            for key in ["file", "csv", "import", "users"]:
                if key in form:
                    val = form[key]
                    if hasattr(val, "read"):
                        data = await val.read()
                        csv_text = data.decode("utf-8", errors="ignore")
                        break
                    elif isinstance(val, str):
                        csv_text = val
                        break
            if not csv_text:
                # try any file part
                for v in form.values():
                    if hasattr(v, "read"):
                        try:
                            data = await v.read()
                            csv_text = data.decode("utf-8", errors="ignore")
                            break
                        except Exception:
                            continue
        except Exception as e:
            raise HTTPException(status_code=422, detail=f"Failed to parse multipart: {e}")
    else:
        # JSON body
        try:
            payload = await request.json()
        except Exception:
            payload = {}
        if isinstance(payload, dict):
            csv_text = payload.get("csv") or payload.get("data") or payload.get("content")
            # also support direct users array
            if not csv_text and "users" in payload and isinstance(payload["users"], list):
                # convert users array to CSV-like rows via import_service helper
                from app.users.import_service import _validate_and_create_user
                imported = 0
                errors = []
                created = []
                for idx, row in enumerate(payload["users"], start=1):
                    try:
                        user, err = await _validate_and_create_user(session, domain_id, row)
                        if err:
                            errors.append({"row": idx, "error": err, "data": row})
                        else:
                            imported += 1
                            created.append({"user_uuid": str(user.user_uuid), "username": user.username})
                    except Exception as e:
                        errors.append({"row": idx, "error": str(e), "data": row})
                await session.commit()
                try:
                    await invalidate_fs_cache_for_session(session, domain_id)
                except Exception:
                    pass
                return {"imported": imported, "errors": errors, "data": created, "total": imported}
        # if csv_text still None, try raw body as csv
        if not csv_text:
            try:
                body = await request.body()
                if body:
                    csv_text = body.decode("utf-8", errors="ignore")
                    # heuristic: if not containing comma or newline, treat as not csv
                    if "," not in csv_text and "\n" not in csv_text:
                        csv_text = None
            except Exception:
                pass
    if not csv_text:
        raise HTTPException(status_code=422, detail="CSV content required: provide JSON {'csv': 'username,password,email\\n...'} or multipart file")
    from app.users.import_service import import_users_from_csv
    result = await import_users_from_csv(session, domain_id, csv_text)
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    return result

@router.get("/{user_uuid}", response_model=dict)
async def get_user(user_uuid: UUID, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid==user_uuid))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    perms = getattr(current, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, current)
    show_all = ("*" in perms or "user_all" in perms)
    if not show_all and str(u.domain_id)!=str(domain_id):
        raise HTTPException(status_code=404, detail="User not found")
    enriched = await enrich_user(session, u)
    # also include groups, settings
    gr = await session.execute(select(Group).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==user_uuid))
    groups = [{"group_uuid": str(g.group_uuid), "name": g.name, "description": g.description, "group_level": g.group_level, "domain_id": str(g.domain_id) if g.domain_id else None} for g in gr.scalars().all()]
    sr = await session.execute(select(UserSetting).where(UserSetting.user_uuid==user_uuid))
    settings = [{"user_setting_uuid": str(s.user_setting_uuid), "category": s.user_setting_category, "subcategory": s.user_setting_subcategory, "name": s.user_setting_name, "value": s.user_setting_value, "enabled": s.user_setting_enabled} for s in sr.scalars().all()]
    enriched["groups"] = groups
    enriched["user_settings"] = settings
    return enriched

@router.post("", status_code=201)
async def create_user(payload: dict, response: Response, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    perms = getattr(current, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, current)
    if not ("*" in perms or "user_add" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_add")
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    # domain handling
    target_domain = payload.get("domain_uuid") or payload.get("domain_id") or str(domain_id)
    try:
        target_domain_uuid = UUID(str(target_domain))
    except Exception:
        raise HTTPException(status_code=422, detail="Invalid domain_uuid")
    if str(target_domain_uuid)!=str(domain_id) and not ("*" in perms or "user_domain" in perms or "user_all" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_domain")
    username = payload.get("username") or payload.get("user_name")
    if not username:
        raise HTTPException(status_code=422, detail="username required")
    # username format check from domain_settings users/username_format (any/email/no_email)
    username_format = await _get_domain_setting(session, target_domain_uuid, "users", "username_format", default="any")
    # normalize: value may be "any"/"email"/"no_email"
    if username_format and username_format != "any":
        is_email = _is_valid_email(username)
        if username_format == "email" and not is_email:
            raise HTTPException(status_code=422, detail="username must be email")
        if username_format == "no_email" and is_email:
            raise HTTPException(status_code=422, detail="username must not be email")
    # username length / chars? legacy not strict, but enforce max 64
    if len(username) > 64 or len(username) < 2:
        raise HTTPException(status_code=422, detail="username must be 2-64 characters")
    # unique check
    # check domain_settings users/unique global vs domain
    unique_global = False
    unique_val = await _get_domain_setting(session, target_domain_uuid, "users", "unique", default=None)
    if unique_val == "global":
        unique_global = True
    if unique_global:
        r = await session.execute(select(User).where(User.username==username))
        if r.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Username exists globally")
    else:
        r = await session.execute(select(User).where(User.username==username, User.domain_id==target_domain_uuid))
        if r.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Username exists")
    password = payload.get("password")
    if not password:
        raise HTTPException(status_code=422, detail="password required")
    # password requirements: fetch domain_settings or default 12/true/true/true/false
    pw_len_val = await _get_domain_setting(session, target_domain_uuid, "users", "password_length", default="12")
    try:
        pw_len = int(pw_len_val) if pw_len_val is not None else 12
    except Exception:
        pw_len = 12
    pw_number = await _get_domain_setting(session, target_domain_uuid, "users", "password_number", default="true")
    pw_lower = await _get_domain_setting(session, target_domain_uuid, "users", "password_lowercase", default="true")
    pw_upper = await _get_domain_setting(session, target_domain_uuid, "users", "password_uppercase", default="true")
    pw_special = await _get_domain_setting(session, target_domain_uuid, "users", "password_special", default="false")
    def _to_bool(v):
        if isinstance(v, bool):
            return v
        if isinstance(v, str):
            return v.lower() in ("true", "1", "yes", "t")
        return bool(v)
    need_number = _to_bool(pw_number)
    need_lower = _to_bool(pw_lower)
    need_upper = _to_bool(pw_upper)
    need_special = _to_bool(pw_special)
    pw_errors = _validate_password_strength(password, length_req=pw_len, need_number=need_number, need_lower=need_lower, need_upper=need_upper, need_special=need_special)
    if pw_errors:
        raise HTTPException(status_code=422, detail="; ".join(pw_errors))
    if payload.get("password_confirm") and password != payload.get("password_confirm"):
        raise HTTPException(status_code=422, detail="Passwords mismatch")
    # limit check
    ls = await session.execute(select(DomainSettings).where(DomainSettings.domain_id==target_domain_uuid, DomainSettings.category=="limit", DomainSettings.subcategory=="users"))
    lim_row = ls.scalar_one_or_none()
    if lim_row and lim_row.enabled:
        try:
            lim = int(lim_row.value)
            cnt = (await session.execute(select(func.count()).select_from(User).where(User.domain_id==target_domain_uuid))).scalar_one()
            if cnt >= lim:
                raise HTTPException(status_code=409, detail=f"Maximum users {lim} reached")
        except HTTPException:
            raise
        except Exception:
            pass
    email = payload.get("user_email") or payload.get("email")
    if email and not _is_valid_email(email):
        raise HTTPException(status_code=422, detail="invalid email")
    # create user
    u = User(domain_id=target_domain_uuid, username=username, email=email, password_hash=hash_password(password), enabled=payload.get("user_enabled", payload.get("enabled", True)), user_status=payload.get("user_status"), user_type=payload.get("user_type", "default"), api_key=payload.get("api_key"), user_totp_secret=payload.get("user_totp_secret"))
    # contact handling: if contact_uuid provided, link, else create contact if organization/name provided
    contact_uuid = payload.get("contact_uuid")
    if contact_uuid:
        try:
            u.contact_uuid = UUID(str(contact_uuid))
        except Exception:
            pass
    else:
        # create contact if fields present and contact_add permitted
        org = payload.get("contact_organization") or payload.get("organization")
        given = payload.get("contact_name_given") or payload.get("first_name")
        family = payload.get("contact_name_family") or payload.get("last_name")
        if org or given or family:
            c_uuid = uuid6.uuid7()
            c = Contact(contact_uuid=c_uuid, domain_id=target_domain_uuid, name=f"{given or ''} {family or ''}".strip() or username, number="", email=email, organization=org)
            session.add(c)
            await session.flush()
            u.contact_uuid = c_uuid
    session.add(u)
    await session.flush()
    # user_settings: language, time_zone, message_key
    lang = payload.get("user_language") or payload.get("language")
    if lang:
        us = UserSetting(user_uuid=u.user_uuid, domain_id=target_domain_uuid, user_setting_category="domain", user_setting_subcategory="language", user_setting_name="code", user_setting_value=lang, user_setting_enabled=True)
        session.add(us)
    tz = payload.get("user_time_zone") or payload.get("time_zone")
    if tz:
        us2 = UserSetting(user_uuid=u.user_uuid, domain_id=target_domain_uuid, user_setting_category="domain", user_setting_subcategory="time_zone", user_setting_name="name", user_setting_value=tz, user_setting_enabled=True)
        session.add(us2)
    mkey = payload.get("message_key")
    if mkey and ("*" in perms or "message_key" in perms):
        us3 = UserSetting(user_uuid=u.user_uuid, domain_id=target_domain_uuid, user_setting_category="message", user_setting_subcategory="key", user_setting_name="text", user_setting_value=mkey, user_setting_enabled=True)
        session.add(us3)
    # group assignment
    group_uuid_name = payload.get("group_uuid_name") or payload.get("group_uuid") or payload.get("group_name")
    if group_uuid_name:
        # format "uuid|name" or just uuid
        if "|" in str(group_uuid_name):
            g_uuid_str, g_name = str(group_uuid_name).split("|",1)
        else:
            g_uuid_str = str(group_uuid_name)
            g_name = None
        try:
            g_uuid = UUID(g_uuid_str)
            gr = await session.execute(select(Group).where(Group.group_uuid==g_uuid))
            grp = gr.scalar_one_or_none()
            if grp:
                # delegation check: * means superadmin (no lvl fallback)
                lvl = await get_user_level(session, current)
                my_uuids = await get_user_group_uuids(session, current)
                if grp.group_level is not None and grp.group_level >= lvl and g_uuid not in my_uuids and not _is_superadmin(current):
                    raise HTTPException(status_code=403, detail="Cannot assign group at your level or higher")
                ug = UserGroup(user_uuid=u.user_uuid, group_uuid=g_uuid, domain_id=target_domain_uuid, group_name=grp.name)
                session.add(ug)
            elif g_name:
                # try lookup by name
                gr2 = await session.execute(select(Group).where(Group.name==g_name, or_(Group.domain_id==target_domain_uuid, Group.domain_id==None)))
                grp2 = gr2.scalar_one_or_none()
                if grp2:
                    ug = UserGroup(user_uuid=u.user_uuid, group_uuid=grp2.group_uuid, domain_id=target_domain_uuid, group_name=grp2.name)
                    session.add(ug)
        except HTTPException:
            raise
        except Exception as e:
            raise HTTPException(status_code=422, detail=str(e))
    else:
        # handle groups array?
        groups_arr = payload.get("groups") or payload.get("group_uuids")
        if groups_arr and isinstance(groups_arr, list):
            for g_uuid_str in groups_arr:
                try:
                    g_uuid = UUID(str(g_uuid_str))
                    gr = await session.execute(select(Group).where(Group.group_uuid==g_uuid))
                    grp = gr.scalar_one_or_none()
                    if grp:
                        lvl = await get_user_level(session, current)
                        my_uuids = await get_user_group_uuids(session, current)
                        if grp.group_level is not None and grp.group_level >= lvl and g_uuid not in my_uuids and not _is_superadmin(current):
                            continue
                        ug = UserGroup(user_uuid=u.user_uuid, group_uuid=g_uuid, domain_id=target_domain_uuid, group_name=grp.name)
                        session.add(ug)
                except Exception: continue
    await session.commit()
    await session.refresh(u)
    try:
        await invalidate_fs_cache_for_session(session, target_domain_uuid)
    except Exception:
        pass
    response.headers["Location"] = f"/api/v1/users/{u.user_uuid}"
    enriched = await enrich_user(session, u)
    return enriched

@router.patch("/{user_uuid}")
async def patch_user(user_uuid: UUID, payload: dict, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    perms = getattr(current, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, current)
    if not ("*" in perms or "user_edit" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_edit")
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid==user_uuid))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    show_all = ("*" in perms or "user_all" in perms)
    if not show_all and str(u.domain_id)!=str(domain_id):
        raise HTTPException(status_code=404, detail="User not found")
    # superadmin protection: * means superadmin, no lvl fallback
    gr = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==user_uuid))
    is_super = any(row[0]=="superadmin" for row in gr.all())
    if is_super:
        cr = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==current.user_uuid))
        is_cur_super = any(row[0]=="superadmin" for row in cr.all()) or "*" in perms
        if not is_cur_super:
            raise HTTPException(status_code=403, detail="Only superadmin can edit superadmin")
    # update fields
    if "username" in payload and payload["username"]!=u.username:
        new_name = payload["username"]
        # username format validation
        username_format = await _get_domain_setting(session, u.domain_id, "users", "username_format", default="any")
        if username_format and username_format != "any":
            is_email = _is_valid_email(new_name)
            if username_format == "email" and not is_email:
                raise HTTPException(status_code=422, detail="username must be email")
            if username_format == "no_email" and is_email:
                raise HTTPException(status_code=422, detail="username must not be email")
        if len(new_name) > 64 or len(new_name) < 2:
            raise HTTPException(status_code=422, detail="username must be 2-64 characters")
        # unique check
        unique_global=False
        unique_val = await _get_domain_setting(session, u.domain_id, "users", "unique", default=None)
        if unique_val == "global":
            unique_global=True
        if unique_global:
            chk = await session.execute(select(User).where(User.username==new_name, User.user_uuid!=user_uuid))
            if chk.scalar_one_or_none():
                raise HTTPException(status_code=409, detail="Username exists globally")
        else:
            chk = await session.execute(select(User).where(User.username==new_name, User.domain_id==u.domain_id, User.user_uuid!=user_uuid))
            if chk.scalar_one_or_none():
                raise HTTPException(status_code=409, detail="Username exists")
        u.username = new_name
    for k in ["email","user_email"]:
        if k in payload:
            new_email = payload[k]
            if new_email and not _is_valid_email(new_email):
                raise HTTPException(status_code=422, detail="invalid email")
            u.email = new_email
            break
    if "enabled" in payload or "user_enabled" in payload:
        u.enabled = payload.get("enabled", payload.get("user_enabled"))
    if "user_status" in payload:
        # validate status
        valid_status = ["", "Available", "Available (On Demand)", "On Break", "Do Not Disturb", "Logged Out"]
        if payload["user_status"] in valid_status:
            u.user_status = payload["user_status"] or None
    if "user_type" in payload and ("*" in perms or "user_type" in perms):
        u.user_type = payload["user_type"]
    if "api_key" in payload and ("*" in perms or "api_key" in perms):
        u.api_key = payload["api_key"] or None
    if "user_totp_secret" in payload:
        # check TOTP method enabled
        u.user_totp_secret = payload["user_totp_secret"] or None
    if "password" in payload and payload["password"]:
        if not ("*" in perms or "user_password" in perms or "domain_add" in perms):
            raise HTTPException(status_code=403, detail="Missing permission: user_password")
        pw = payload["password"]
        if payload.get("password_confirm") and pw != payload["password_confirm"]:
            raise HTTPException(status_code=422, detail="Passwords mismatch")
        pw_len_val = await _get_domain_setting(session, u.domain_id, "users", "password_length", default="12")
        try:
            pw_len = int(pw_len_val) if pw_len_val is not None else 12
        except Exception:
            pw_len = 12
        pw_number = await _get_domain_setting(session, u.domain_id, "users", "password_number", default="true")
        pw_lower = await _get_domain_setting(session, u.domain_id, "users", "password_lowercase", default="true")
        pw_upper = await _get_domain_setting(session, u.domain_id, "users", "password_uppercase", default="true")
        pw_special = await _get_domain_setting(session, u.domain_id, "users", "password_special", default="false")
        def _to_bool2(v):
            if isinstance(v, bool):
                return v
            if isinstance(v, str):
                return v.lower() in ("true", "1", "yes", "t")
            return bool(v)
        need_number = _to_bool2(pw_number)
        need_lower = _to_bool2(pw_lower)
        need_upper = _to_bool2(pw_upper)
        need_special = _to_bool2(pw_special)
        pw_errors = _validate_password_strength(pw, length_req=pw_len, need_number=need_number, need_lower=need_lower, need_upper=need_upper, need_special=need_special)
        if pw_errors:
            raise HTTPException(status_code=422, detail="; ".join(pw_errors))
        u.password_hash = hash_password(pw)
        # delete remember tokens? skip
        # send email? skip
    if "domain_uuid" in payload or "domain_id" in payload:
        if not ("*" in perms or "user_domain" in perms):
            raise HTTPException(status_code=403, detail="Missing permission: user_domain")
        new_dom_str = payload.get("domain_uuid") or payload.get("domain_id")
        try:
            new_dom = UUID(str(new_dom_str))
            # update user and related group/settings domain
            old_dom = u.domain_id
            u.domain_id = new_dom
            # update user_groups domain
            await session.execute(update(UserGroup).where(UserGroup.user_uuid==user_uuid).values(domain_id=new_dom))
            await session.execute(update(UserSetting).where(UserSetting.user_uuid==user_uuid).values(domain_id=new_dom))
            # remove groups not in new domain (legacy)
            await session.execute(delete(UserGroup).where(UserGroup.user_uuid==user_uuid, UserGroup.domain_id==old_dom, UserGroup.group_uuid.notin_(select(Group.group_uuid).where(or_(Group.domain_id==new_dom, Group.domain_id==None)))))
        except Exception as e:
            raise HTTPException(status_code=422, detail=str(e))
    # contact
    if "contact_uuid" in payload:
        try:
            if payload["contact_uuid"]:
                u.contact_uuid = UUID(str(payload["contact_uuid"]))
            else:
                u.contact_uuid = None
        except Exception:
            raise HTTPException(status_code=422, detail="Invalid contact_uuid")
    # contact organization/name/email linkage update parity
    contact_fields = {}
    # support nested contact dict like {"contact": {"organization": "NewOrg", "name": "..."}}
    if "contact" in payload and isinstance(payload["contact"], dict):
        cdict = payload["contact"]
        if "organization" in cdict:
            contact_fields["organization"] = cdict["organization"]
        if "name" in cdict:
            contact_fields["name"] = cdict["name"]
        if "email" in cdict:
            contact_fields["email"] = cdict["email"]
    if "contact_organization" in payload:
        contact_fields["organization"] = payload["contact_organization"]
    elif "organization" in payload:
        contact_fields["organization"] = payload["organization"]
    if "contact_name" in payload:
        contact_fields["name"] = payload["contact_name"]
    elif "contact_name_given" in payload or "contact_name_family" in payload:
        given = payload.get("contact_name_given") or ""
        family = payload.get("contact_name_family") or ""
        combined = f"{given} {family}".strip()
        if combined:
            contact_fields["name"] = combined
    if "contact_email" in payload:
        contact_fields["email"] = payload["contact_email"]
    elif "contact_note" in payload:
        contact_fields["email"] = payload["contact_note"]
    if contact_fields:
        # fetch or create contact
        contact_obj = None
        if u.contact_uuid:
            cr = await session.execute(select(Contact).where(Contact.contact_uuid == u.contact_uuid))
            contact_obj = cr.scalar_one_or_none()
        if contact_obj:
            for k, v in contact_fields.items():
                setattr(contact_obj, k, v)
        else:
            # create new contact linked to user
            c_uuid = uuid6.uuid7()
            name_val = contact_fields.get("name") or u.username
            org_val = contact_fields.get("organization")
            email_val = contact_fields.get("email") or u.email
            new_c = Contact(contact_uuid=c_uuid, domain_id=u.domain_id, name=name_val, number="", email=email_val, organization=org_val)
            session.add(new_c)
            await session.flush()
            u.contact_uuid = c_uuid
    # user_settings updates
    if "user_language" in payload or "language" in payload:
        lang = payload.get("user_language") or payload.get("language")
        # upsert
        sr = await session.execute(select(UserSetting).where(UserSetting.user_uuid==user_uuid, UserSetting.user_setting_category=="domain", UserSetting.user_setting_subcategory=="language"))
        s = sr.scalar_one_or_none()
        if lang:
            if not s:
                ns = UserSetting(user_uuid=user_uuid, domain_id=u.domain_id, user_setting_category="domain", user_setting_subcategory="language", user_setting_name="code", user_setting_value=lang, user_setting_enabled=True)
                session.add(ns)
            else:
                s.user_setting_value = lang
        else:
            if s:
                await session.delete(s)
    if "user_time_zone" in payload or "time_zone" in payload:
        tz = payload.get("user_time_zone") or payload.get("time_zone")
        sr = await session.execute(select(UserSetting).where(UserSetting.user_uuid==user_uuid, UserSetting.user_setting_category=="domain", UserSetting.user_setting_subcategory=="time_zone"))
        s = sr.scalar_one_or_none()
        if tz:
            if not s:
                ns = UserSetting(user_uuid=user_uuid, domain_id=u.domain_id, user_setting_category="domain", user_setting_subcategory="time_zone", user_setting_name="name", user_setting_value=tz, user_setting_enabled=True)
                session.add(ns)
            else:
                s.user_setting_value = tz
        else:
            if s:
                await session.delete(s)
    if "message_key" in payload and ("*" in perms or "message_key" in perms):
        mk = payload["message_key"]
        sr = await session.execute(select(UserSetting).where(UserSetting.user_uuid==user_uuid, UserSetting.user_setting_category=="message", UserSetting.user_setting_subcategory=="key"))
        s = sr.scalar_one_or_none()
        if mk:
            if not s:
                ns = UserSetting(user_uuid=user_uuid, domain_id=u.domain_id, user_setting_category="message", user_setting_subcategory="key", user_setting_name="text", user_setting_value=mk, user_setting_enabled=True)
                session.add(ns)
            else:
                s.user_setting_value = mk
        else:
            if s:
                await session.delete(s)
    # group assignment via group_uuid_name (add one group)
    if "group_uuid_name" in payload and payload["group_uuid_name"]:
        ggn = payload["group_uuid_name"]
        if "|" in str(ggn):
            g_uuid_str, g_name = str(ggn).split("|",1)
        else:
            g_uuid_str = str(ggn); g_name=None
        try:
            g_uuid = UUID(g_uuid_str)
            gr = await session.execute(select(Group).where(Group.group_uuid==g_uuid))
            grp = gr.scalar_one_or_none()
            if grp:
                lvl = await get_user_level(session, current)
                my_uuids = await get_user_group_uuids(session, current)
                if grp.group_level is not None and grp.group_level >= lvl and g_uuid not in my_uuids and not _is_superadmin(current):
                    raise HTTPException(status_code=403, detail="Cannot assign group at your level or higher")
                # check duplicate
                chk = await session.execute(select(UserGroup).where(UserGroup.user_uuid==user_uuid, UserGroup.group_uuid==g_uuid))
                if not chk.scalar_one_or_none():
                    ug = UserGroup(user_uuid=user_uuid, group_uuid=g_uuid, domain_id=u.domain_id, group_name=grp.name)
                    session.add(ug)
        except HTTPException:
            raise
        except Exception:
            pass
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, u.domain_id)
    except Exception:
        pass
    await session.refresh(u)
    enriched = await enrich_user(session, u)
    return enriched

@router.delete("/{user_uuid}", status_code=204)
async def delete_user(user_uuid: UUID, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    perms = getattr(current, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, current)
    if not ("*" in perms or "user_delete" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_delete")
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid==user_uuid))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    show_all = ("*" in perms or "user_all" in perms)
    if not show_all and str(u.domain_id)!=str(domain_id):
        raise HTTPException(status_code=404, detail="User not found")
    # superadmin protection: * means superadmin, no lvl fallback
    gr = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==user_uuid))
    is_super = any(row[0]=="superadmin" for row in gr.all())
    if is_super:
        cr = await session.execute(select(Group.name).join(UserGroup, UserGroup.group_uuid==Group.group_uuid).where(UserGroup.user_uuid==current.user_uuid))
        is_cur_super = any(row[0]=="superadmin" for row in cr.all()) or "*" in perms
        if not is_cur_super:
            raise HTTPException(status_code=403, detail="Only superadmin can delete superadmin")
    await session.delete(u)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, u.domain_id)
    except Exception:
        pass
    return Response(status_code=204)

# user_settings endpoints

@router.get("/{user_uuid}/settings", response_model=dict)
async def get_user_settings(user_uuid: UUID, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid==user_uuid))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="User not found")
    sr = await session.execute(select(UserSetting).where(UserSetting.user_uuid==user_uuid))
    data = [{"user_setting_uuid": str(s.user_setting_uuid), "category": s.user_setting_category, "subcategory": s.user_setting_subcategory, "name": s.user_setting_name, "value": s.user_setting_value, "enabled": s.user_setting_enabled} for s in sr.scalars().all()]
    return {"data": data, "total": len(data)}

@router.put("/{user_uuid}/settings", response_model=dict)
async def put_user_settings(user_uuid: UUID, payload: dict, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    is_self = str(getattr(current, "user_uuid", "")) == str(user_uuid)
    perms = getattr(current, "_perms", None)
    if perms is None:
        perms = await get_user_permissions(session, current)
    if not is_self and not ("*" in perms or "user_setting_view" in perms or "user_edit" in perms):
        raise HTTPException(status_code=403, detail="Missing permission")
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid==user_uuid))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    # payload expected: list of settings or dict
    # simplified: payload = {"settings": [{"category","subcategory","name","value","enabled"}]}
    settings = payload.get("settings") or payload.get("data") or []
    # delete existing where needed? Replace all if provided
    if isinstance(settings, list):
        # delete all existing then recreate
        await session.execute(delete(UserSetting).where(UserSetting.user_uuid==user_uuid))
        for s in settings:
            ns = UserSetting(user_uuid=user_uuid, domain_id=u.domain_id, user_setting_category=s.get("category"), user_setting_subcategory=s.get("subcategory"), user_setting_name=s.get("name"), user_setting_value=s.get("value"), user_setting_enabled=s.get("enabled", True))
            session.add(ns)
        await session.commit()
    return {"status": "updated"}

