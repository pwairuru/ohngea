# TODO: gate all apps centrally via middleware, currently 5 routers as MVP
from fastapi import Depends, HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.auth.deps import get_current_user
from app.auth.models import User
from app.db import get_session, set_tenant
from app.registry import discover_apps

def require_app_enabled(app_name: str):
    async def _check(session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
        from app.domains.models import DomainApp
        domain_id = getattr(user, "_domain_id", user.domain_id)
        await set_tenant(session, domain_id)
        r = await session.execute(select(DomainApp).where(DomainApp.domain_id == domain_id, DomainApp.app_name == app_name))
        row = r.scalar_one_or_none()
        if row is not None:
            enabled = row.enabled
        else:
            apps = {a["name"]: a for a in discover_apps()}
            enabled = apps.get(app_name, {}).get("enabled_by_default", True)
        if not enabled:
            raise HTTPException(status_code=403, detail=f"App {app_name} is disabled for this domain")
        return True
    return Depends(_check)
