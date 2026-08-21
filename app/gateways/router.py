from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Response
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.gateways.models import Gateway

router = APIRouter(prefix="/api/v1/gateways", tags=["gateways"])


@router.get("")
async def list_gateways(response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    total = (await session.execute(select(func.count()).select_from(select(Gateway).where(Gateway.domain_id == domain_id).subquery()))).scalar() or 0
    r = await session.execute(select(Gateway).where(Gateway.domain_id == domain_id).offset((page - 1) * per_page).limit(per_page))
    data = [{"gateway_uuid": str(g.gateway_uuid), "gateway": g.gateway, "enabled": g.enabled, "_links": {"self": {"href": f"/api/v1/gateways/{g.gateway_uuid}"}}} for g in r.scalars().all()]
    response.headers["X-Total-Count"] = str(total)
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/gateways?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}


@router.post("", status_code=201)
async def create_gateway(payload: dict, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    gw = Gateway(domain_id=domain_id, gateway=payload.get("gateway"), username=payload.get("username"), password=payload.get("password"), proxy=payload.get("proxy"), register=payload.get("register", True), enabled=payload.get("enabled", True))
    session.add(gw)
    await session.commit()
    await session.refresh(gw)
    response.headers["Location"] = f"/api/v1/gateways/{gw.gateway_uuid}"
    return {"gateway_uuid": str(gw.gateway_uuid), "gateway": gw.gateway, "_links": {"self": {"href": f"/api/v1/gateways/{gw.gateway_uuid}"}}}


@router.get("/{gateway_uuid}")
async def get_gateway(gateway_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Gateway).where(Gateway.gateway_uuid == gateway_uuid, Gateway.domain_id == domain_id))
    g = r.scalar_one_or_none()
    if not g:
        raise HTTPException(status_code=404, detail="Gateway not found")
    return {"gateway_uuid": str(g.gateway_uuid), "gateway": g.gateway, "enabled": g.enabled, "_links": {"self": {"href": f"/api/v1/gateways/{g.gateway_uuid}"}}}


@router.delete("/{gateway_uuid}", status_code=204)
async def delete_gateway(gateway_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("gateway_view"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(Gateway).where(Gateway.gateway_uuid == gateway_uuid, Gateway.domain_id == domain_id))
    g = r.scalar_one_or_none()
    if not g:
        raise HTTPException(status_code=404, detail="Gateway not found")
    await session.delete(g)
    await session.commit()
    return Response(status_code=204)
