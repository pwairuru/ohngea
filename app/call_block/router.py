from typing import Optional
from uuid import UUID

from fastapi import APIRouter, Depends, HTTPException, Query, Request, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User
from app.db import get_session, set_tenant
from app.call_block.models import CallBlock
from app.call_block.schemas import CallBlockCreate, CallBlockRead, CallBlockUpdate
from app.call_block.service import create_call_block, list_call_block

router = APIRouter(prefix="/api/v1/call_block", tags=["call_block"])


def _links(cb: CallBlock):
    base = f"/api/v1/call_block/{cb.call_block_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/call_block"}}


def _serialize(cb: CallBlock):
    d = CallBlockRead.model_validate(cb, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(cb)
    d["call_block_uuid"] = str(cb.call_block_uuid)
    return d


@router.get("", response_model=dict)
async def list_cb(request: Request, response: Response, page: int = Query(1, ge=1), per_page: int = Query(20, ge=1, le=100), q: Optional[str] = Query(None), session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    items, total = await list_call_block(session, domain_id, page, per_page, q)
    response.headers["X-Total-Count"] = str(total)
    return {"data": [_serialize(x) for x in items], "total": total, "page": page, "per_page": per_page}


@router.post("", response_model=dict, status_code=201)
async def create_cb(payload: CallBlockCreate, response: Response, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_block_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    if payload.domain_id and str(payload.domain_id) != str(domain_id):
        raise HTTPException(status_code=403, detail="Domain mismatch")
    await set_tenant(session, domain_id)
    cb = await create_call_block(session, payload, domain_id)
    response.headers["Location"] = f"/api/v1/call_block/{cb.call_block_uuid}"
    return _serialize(cb)


@router.get("/{cb_uuid}", response_model=dict)
async def get_cb(cb_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallBlock).where(CallBlock.call_block_uuid == cb_uuid, CallBlock.domain_id == domain_id))
    cb = r.scalar_one_or_none()
    if not cb:
        raise HTTPException(status_code=404, detail="Call block not found")
    return _serialize(cb)


@router.patch("/{cb_uuid}", response_model=dict)
async def patch_cb(cb_uuid: UUID, payload: CallBlockUpdate, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_block_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallBlock).where(CallBlock.call_block_uuid == cb_uuid, CallBlock.domain_id == domain_id))
    cb = r.scalar_one_or_none()
    if not cb:
        raise HTTPException(status_code=404, detail="Call block not found")
    update = payload.model_dump(exclude_unset=True)
    for k, v in update.items():
        if k == "name":
            cb.call_block_name = v
        elif k == "number":
            cb.call_block_number = v
        elif k == "enabled":
            cb.call_block_enabled = v
        elif k == "description":
            cb.call_block_description = v
        else:
            setattr(cb, k, v)
    await session.commit()
    await session.refresh(cb)
    return _serialize(cb)


@router.delete("/{cb_uuid}", status_code=204)
async def delete_cb(cb_uuid: UUID, session: AsyncSession = Depends(get_session), user: User = Depends(require_permission("call_block_add"))):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(CallBlock).where(CallBlock.call_block_uuid == cb_uuid, CallBlock.domain_id == domain_id))
    cb = r.scalar_one_or_none()
    if not cb:
        raise HTTPException(status_code=404, detail="Call block not found")
    await session.delete(cb)
    await session.commit()
    return Response(status_code=204)
