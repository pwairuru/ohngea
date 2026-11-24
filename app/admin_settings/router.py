"""Core admin editors: default/domain settings, email templates, menu items, user logs, notifications, services.

One module, seven small CRUD resources sharing a generic factory. Domain-scoped
tables reuse the tenant RLS pattern; global tables skip RLS like other globals.
"""
from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import or_, select
from sqlalchemy.ext.asyncio import AsyncSession
from app.admin_settings.models import (
    DefaultSetting,
    EmailTemplate,
    MenuItem,
    Notification,
    Service,
    UserLog,
)
from app.domains.models import DomainSettings
from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant, uuid7
router = APIRouter(tags=["admin_settings"])


def _to_dict(obj) -> dict:
    d = {}
    for col in obj.__table__.columns:
        v = getattr(obj, col.name)
        if isinstance(v, UUID):
            v = str(v)
        elif hasattr(v, "isoformat") and v is not None:
            try:
                v = str(v.isoformat())
            except Exception:
                v = str(v)
        d[col.name] = v
    return d


def _coerce_uuids(model, data: dict) -> dict:
    """SQLite/PG need real UUID objects — coerce UUID-column strings (SQLModel may leave them raw)."""
    import uuid as _uuid_mod

    cols = {c.name for c in model.__table__.columns if "uuid" in type(c.type).__name__.lower()}
    for k in list(data.keys()):
        if k in cols and isinstance(data[k], str):
            try:
                data[k] = _uuid_mod.UUID(data[k])
            except Exception:
                pass
    return data


def _crud(
    prefix: str,
    model,
    pk: str,
    *,
    domain_scoped: bool,
    domain_nullable: bool = False,
    perms: tuple[str, str, str, str],
    search_cols: tuple[str, ...] = (),
    required: tuple[str, ...] = (),
    read_only: bool = False,
    owner_field: Optional[str] = None,
):
    """Register list/get/post/patch/delete for a simple table. Returns the sub-router."""
    sub = APIRouter(prefix=prefix)
    view_perm, add_perm, edit_perm, delete_perm = perms

    def _base_query(domain_id):
        q = select(model)
        if domain_scoped and not domain_nullable:
            q = q.where(model.domain_id == domain_id)
        elif domain_scoped and domain_nullable:
            q = q.where(or_(model.domain_id == domain_id, model.domain_id.is_(None)))
        return q

    @sub.get("", response_model=dict)
    async def list_ep(
        request: Request,
        response: Response,
        page: int = Query(1, ge=1),
        per_page: int = Query(20, ge=1, le=100),
        q: Optional[str] = Query(None),
        session: AsyncSession = Depends(get_session),
        user: User = Depends(require_permission(view_perm)),
    ):
        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        base = _base_query(domain_id)
        if q and search_cols:
            like = f"%{q}%"
            base = base.where(or_(*[getattr(model, c).ilike(like) for c in search_cols]))
        if owner_field and request.query_params.get("mine") == "1":
            me = getattr(user, "user_uuid", None)
            base = base.where(or_(getattr(model, owner_field) == me, getattr(model, owner_field).is_(None)))
        items = (await session.execute(base.offset((page - 1) * per_page).limit(per_page))).scalars().all()
        return {"data": [_to_dict(x) for x in items], "total": len(items), "page": page, "per_page": per_page}

    @sub.get("/{uid}", response_model=dict)
    async def get_ep(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission(view_perm))):
        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        r = await session.execute(select(model).where(getattr(model, pk) == uid))
        obj = r.scalar_one_or_none()
        if not obj:
            raise HTTPException(status_code=404, detail="Not found")
        if domain_scoped and not domain_nullable and str(obj.domain_id) != str(domain_id):
            raise HTTPException(status_code=404, detail="Not found")
        return _to_dict(obj)

    if not read_only:
        @sub.post("", response_model=dict, status_code=201)
        async def create_ep(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission(add_perm))):
            domain_id = getattr(user, "_domain_id", user.domain_id)
            await set_tenant(session, domain_id)
            for f in required:
                if f not in payload or payload[f] in (None, ""):
                    raise HTTPException(status_code=422, detail=f"{f} required")
            allowed = {c.name for c in model.__table__.columns} - {pk, "created_at", "updated_at", "domain_id"}
            data = _coerce_uuids(model, {k: v for k, v in payload.items() if k in allowed})
            if domain_scoped and not domain_nullable:
                data["domain_id"] = domain_id
            elif domain_scoped and domain_nullable:
                data["domain_id"] = payload.get("domain_id") or domain_id
            data[pk] = uuid7()
            obj = model(**data)
            session.add(obj)
            await session.commit()
            await session.refresh(obj)
            return _to_dict(obj)

        @sub.patch("/{uid}", response_model=dict)
        async def patch_ep(uid: UUID, payload: dict, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission(edit_perm))):
            domain_id = getattr(user, "_domain_id", user.domain_id)
            await set_tenant(session, domain_id)
            r = await session.execute(select(model).where(getattr(model, pk) == uid))
            obj = r.scalar_one_or_none()
            if not obj:
                raise HTTPException(status_code=404, detail="Not found")
            if domain_scoped and not domain_nullable and str(obj.domain_id) != str(domain_id):
                raise HTTPException(status_code=404, detail="Not found")
            allowed = {c.name for c in model.__table__.columns} - {pk, "created_at", "updated_at", "domain_id"}
            for k in allowed:
                if k in payload:
                    v = payload[k]
                    colnames = {c.name: c for c in model.__table__.columns}
                    if isinstance(v, str) and "uuid" in type(colnames[k].type).__name__.lower():
                        try:
                            v = UUID(v)
                        except Exception:
                            pass
                    setattr(obj, k, v)
            await session.commit()
            await session.refresh(obj)
            return _to_dict(obj)

    @sub.delete("/{uid}", status_code=204)
    async def delete_ep(uid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission(delete_perm))):
        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        r = await session.execute(select(model).where(getattr(model, pk) == uid))
        obj = r.scalar_one_or_none()
        if not obj:
            raise HTTPException(status_code=404, detail="Not found")
        if domain_scoped and not domain_nullable and str(obj.domain_id) != str(domain_id):
            raise HTTPException(status_code=404, detail="Not found")
        await session.delete(obj)
        await session.commit()
        return Response(status_code=204)

    router.include_router(sub)
    return sub


_crud("/api/v1/default_settings", DefaultSetting, "default_setting_uuid", domain_scoped=False,
       perms=("default_setting_view", "default_setting_add", "default_setting_edit", "default_setting_delete"),
       search_cols=("default_setting_category", "default_setting_name", "default_setting_value"), required=("default_setting_category", "default_setting_name"))
_crud("/api/v1/domain_settings", DomainSettings, "id", domain_scoped=True,
       perms=("domain_setting_view", "domain_setting_add", "domain_setting_edit", "domain_setting_delete"),
       search_cols=("category", "key", "value"), required=("category", "key"))
_crud("/api/v1/email_templates", EmailTemplate, "email_template_uuid", domain_scoped=True, domain_nullable=True,
       perms=("email_template_view", "email_template_add", "email_template_edit", "email_template_delete"),
       search_cols=("template_name", "template_subject"), required=("template_name",))
_crud("/api/v1/menu_items", MenuItem, "menu_item_uuid", domain_scoped=False,
       perms=("menu_view", "menu_add", "menu_edit", "menu_delete"),
       search_cols=("title", "path"), required=("title",))
_crud("/api/v1/user_logs", UserLog, "user_log_uuid", domain_scoped=True, domain_nullable=True,
       perms=("user_log_view", "user_log_add", "user_log_edit", "user_log_delete"),
       search_cols=("username", "action", "detail"), read_only=True)
_crud("/api/v1/notifications", Notification, "notification_uuid", domain_scoped=True, domain_nullable=True,
       perms=("notification_view", "notification_add", "notification_edit", "notification_delete"),
       search_cols=("title", "body"), required=("title",), owner_field="user_uuid")
_crud("/api/v1/services", Service, "service_uuid", domain_scoped=False,
       perms=("service_view", "service_add", "service_edit", "service_delete"),
       search_cols=("name", "host"), required=("name",))


def _render_text(tpl: Optional[str], context: dict) -> str:
    out = tpl or ""
    for k, v in (context or {}).items():
        out = out.replace("{{" + str(k) + "}}", str(v))
    return out


async def render_email_template(session: AsyncSession, domain_id, name: str, context: Optional[dict] = None) -> Optional[dict]:
    """Render an enabled email template by name (domain row wins, global row fallback)."""
    for scope in ({"domain_id": domain_id}, {"domain_id": None}):
        r = await session.execute(
            select(EmailTemplate).where(
                EmailTemplate.template_name == name,
                EmailTemplate.enabled == True,  # noqa: E712
                EmailTemplate.domain_id == scope["domain_id"] if scope["domain_id"] is not None else EmailTemplate.domain_id.is_(None),
            )
        )
        tpl = r.scalars().first()
        if tpl:
            return {
                "subject": _render_text(tpl.template_subject, context or {}),
                "body": _render_text(tpl.template_body, context or {}),
            }
    return None


async def log_user_action(session: AsyncSession, domain_id, user_uuid, username: Optional[str], action: str, detail: Optional[str] = None, ip: Optional[str] = None):
    """Best-effort audit writer for login/logout and admin actions."""
    try:
        await set_tenant(session, domain_id)
        session.add(UserLog(domain_id=domain_id, user_uuid=user_uuid, username=username, action=action, detail=(detail or "")[:1024], ip_address=ip))
        await session.commit()
    except Exception:
        try:
            await session.rollback()
        except Exception:
            pass
