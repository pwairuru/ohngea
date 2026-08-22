from typing import List, Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Request, Response, status
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.domains.models import Domain, DomainSettings
from app.domains.schemas import DomainCreate, DomainRead, DomainUpdate, ThemeRead
from app.domains.service import create_domain

router = APIRouter(prefix="/api/v1/domains", tags=["domains"])


@router.get("/bootstrap/status", response_model=dict)
async def bootstrap_status(session: AsyncSession = Depends(get_session)):
    """Public — check if initial setup is needed (no domains exist)."""
    result = await session.execute(select(Domain).limit(1))
    exists = result.scalar_one_or_none() is not None
    return {"needs_setup": not exists, "has_domains": exists}


class BootstrapRequest(BaseModel):
    domain_name: str
    admin_username: str = "admin"
    admin_password: str = "admin"
    admin_email: Optional[str] = None
    description: Optional[str] = None


@router.post("/bootstrap", response_model=DomainRead, status_code=201)
async def bootstrap_domain(payload: BootstrapRequest, response: Response, session: AsyncSession = Depends(get_session)):
    """Public — create first domain without auth if no domains exist. Returns 409 if already bootstrapped."""
    result = await session.execute(select(Domain).limit(1))
    if result.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Already bootstrapped — use authenticated POST /api/v1/domains")
    data = DomainCreate(domain_name=payload.domain_name, enabled=True, description=payload.description)
    domain = await create_domain(session, data, admin_username=payload.admin_username, admin_password=payload.admin_password)
    response.headers["Location"] = f"/api/v1/domains/{domain.domain_id}"
    return domain


@router.get("", response_model=List[DomainRead])
async def list_domains(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    # Only superadmin / domain_add perm can list all — for now allow any authed
    result = await session.execute(select(Domain).order_by(Domain.domain_name))
    domains = result.scalars().all()
    return domains


@router.post("", response_model=DomainRead, status_code=201)
async def create_domain_endpoint(
    data: DomainCreate,
    request: Request,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("domain_add")),
):
    domain = await create_domain(session, data)
    response.headers["Location"] = f"/api/v1/domains/{domain.domain_id}"
    # HATEOAS light
    return domain


@router.get("/{domain_id}", response_model=DomainRead)
async def get_domain(domain_id: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Domain not found")
    return d


@router.patch("/{domain_id}", response_model=DomainRead)
async def update_domain(domain_id: UUID, data: DomainUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("domain_add"))):
    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Domain not found")
    if data.enabled is not None:
        d.enabled = data.enabled
    if data.description is not None:
        d.description = data.description
    await session.commit()
    await session.refresh(d)
    return d


@router.delete("/{domain_id}", status_code=204)
async def delete_domain(domain_id: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("domain_add"))):
    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    d = r.scalar_one_or_none()
    if not d:
        raise HTTPException(status_code=404, detail="Domain not found")
    await session.delete(d)
    await session.commit()
    return Response(status_code=204)


@router.get("/{domain_id}/theme", response_model=ThemeRead)
async def get_theme(domain_id: UUID, session: AsyncSession = Depends(get_session)):
    # public theme fetch for injection (no auth needed for login page)
    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Domain not found")
    # fetch settings
    r2 = await session.execute(select(DomainSettings).where(DomainSettings.domain_id == domain_id, DomainSettings.category == "theme"))
    settings = {s.key: s.value for s in r2.scalars().all()}
    return ThemeRead(
        domain_id=domain_id,
        primary_color=settings.get("primary_color", "221 83% 53%"),
        background=settings.get("background", "0 0% 100%"),
        radius=settings.get("radius", "0.5rem"),
    )


@router.put("/{domain_id}/theme", response_model=ThemeRead)
async def put_theme(domain_id: UUID, theme: ThemeRead, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("domain_add"))):
    for k, v in [("primary_color", theme.primary_color), ("background", theme.background), ("radius", theme.radius)]:
        r = await session.execute(select(DomainSettings).where(DomainSettings.domain_id == domain_id, DomainSettings.key == k))
        s = r.scalar_one_or_none()
        if s:
            s.value = v
        else:
            session.add(DomainSettings(domain_id=domain_id, category="theme", key=k, value=v))
    await session.commit()
    return theme


@router.get("/{domain_id}/apps")
async def list_domain_apps(domain_id: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    from app.domains.models import DomainApp, DomainPlugin
    from app.registry import discover_apps

    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Domain not found")
    perms = getattr(user, "_perms", None) or []
    if str(getattr(user, "_domain_id", user.domain_id)) != str(domain_id) and "domain_add" not in perms and "*" not in perms:
        raise HTTPException(status_code=403, detail="Forbidden")
    await set_tenant(session, domain_id)
    apps = discover_apps()
    app_names = {a["name"] for a in apps}
    r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id))
    rows = r.scalars().all()
    overrides = {p.app_name: p.enabled for p in rows}
    # legacy migration
    r2 = await session.execute(select(DomainPlugin).where(DomainPlugin.domain_id == domain_id))
    plugin_rows = r2.scalars().all()
    if plugin_rows:
        existing = set(overrides.keys())
        to_add = []
        for p in plugin_rows:
            if p.plugin_name not in existing and p.plugin_name in app_names:
                to_add.append(DomainApp(domain_id=domain_id, app_name=p.plugin_name, enabled=p.enabled))
        if to_add:
            for obj in to_add:
                session.add(obj)
            from sqlalchemy.exc import IntegrityError

            try:
                await session.commit()
            except IntegrityError:
                await session.rollback()
            # re-fetch overrides after migration so response reflects migrated values
            r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id))
            overrides = {p.app_name: p.enabled for p in r.scalars().all()}
    data = []
    for a in apps:
        name = a["name"]
        enabled = overrides.get(name, a.get("enabled_by_default", True))
        data.append(
            {
                "app_name": name,
                "enabled": enabled,
                "category": a.get("category", "advanced"),
                "depends_on": a.get("depends_on", []),
                "description": a.get("description", ""),
                "source": a.get("source"),
            }
        )
    return {"data": data, "total": len(data)}


@router.get("/{domain_id}/apps/enabled")
async def list_enabled_apps(domain_id: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    from app.domains.models import DomainApp
    from app.registry import discover_apps

    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Domain not found")
    perms = getattr(user, "_perms", None) or []
    if str(getattr(user, "_domain_id", user.domain_id)) != str(domain_id) and "domain_add" not in perms and "*" not in perms:
        raise HTTPException(status_code=403, detail="Forbidden")
    await set_tenant(session, domain_id)
    apps = discover_apps()
    r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id))
    overrides = {p.app_name: p.enabled for p in r.scalars().all()}
    enabled_list = []
    for a in apps:
        name = a["name"]
        enabled = overrides.get(name, a.get("enabled_by_default", True))
        if enabled:
            enabled_list.append(name)
    return enabled_list


@router.put("/{domain_id}/apps/{app_name}")
async def toggle_app(
    domain_id: UUID,
    app_name: str,
    enabled: bool = True,
    force: bool = False,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(require_permission("domain_add")),
):
    from app.domains.models import DomainApp
    from app.registry import discover_apps

    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Domain not found")
    perms = getattr(user, "_perms", None) or []
    if str(getattr(user, "_domain_id", user.domain_id)) != str(domain_id) and "domain_add" not in perms and "*" not in perms:
        raise HTTPException(status_code=403, detail="Forbidden")
    await set_tenant(session, domain_id)
    apps = {a["name"]: a for a in discover_apps()}
    if app_name not in apps:
        raise HTTPException(status_code=404, detail=f"App {app_name} not found")
    # dependency check: if disabling, prevent if dependents still enabled unless force
    if not enabled and not force:
        dependents = [n for n, m in apps.items() if app_name in m.get("depends_on", [])]
        if dependents:
            r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id))
            overrides = {p.app_name: p.enabled for p in r.scalars().all()}
            active_dependents = [d for d in dependents if overrides.get(d, apps[d].get("enabled_by_default", True))]
            if active_dependents:
                raise HTTPException(
                    status_code=409,
                    detail=f"Cannot disable {app_name}: dependents still enabled {active_dependents}. Disable them first or force with ?force=true",
                )
    r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id, DomainApp.app_name == app_name))
    p = r.scalar_one_or_none()
    if p:
        p.enabled = enabled
    else:
        p = DomainApp(domain_id=domain_id, app_name=app_name, enabled=enabled)
        session.add(p)
    await session.commit()
    return {"app_name": app_name, "enabled": enabled}


@router.get("/{domain_id}/plugins")
async def list_domain_plugins(domain_id: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    from app.domains.models import DomainPlugin

    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Domain not found")
    perms = getattr(user, "_perms", None) or []
    if str(getattr(user, "_domain_id", user.domain_id)) != str(domain_id) and "domain_add" not in perms and "*" not in perms:
        raise HTTPException(status_code=403, detail="Forbidden")
    await set_tenant(session, domain_id)
    r = await session.execute(select(DomainPlugin).where(DomainPlugin.domain_id == domain_id))
    return [{"plugin_name": p.plugin_name, "enabled": p.enabled} for p in r.scalars().all()]


@router.put("/{domain_id}/plugins/{plugin_name}")
async def toggle_plugin(domain_id: UUID, plugin_name: str, enabled: bool = True, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("domain_add"))):
    from app.domains.models import DomainPlugin

    r = await session.execute(select(Domain).where(Domain.domain_id == domain_id))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="Domain not found")
    perms = getattr(user, "_perms", None) or []
    if str(getattr(user, "_domain_id", user.domain_id)) != str(domain_id) and "domain_add" not in perms and "*" not in perms:
        raise HTTPException(status_code=403, detail="Forbidden")
    await set_tenant(session, domain_id)
    r = await session.execute(select(DomainPlugin).where(DomainPlugin.domain_id == domain_id, DomainPlugin.plugin_name == plugin_name))
    p = r.scalar_one_or_none()
    if p:
        p.enabled = enabled
    else:
        p = DomainPlugin(domain_id=domain_id, plugin_name=plugin_name, enabled=enabled)
        session.add(p)
    await session.commit()
    return {"plugin_name": plugin_name, "enabled": enabled}
