from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.vars.models import Var
from app.vars.schemas import VarCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_var(session: AsyncSession, data: VarCreate, domain_id: UUID):
    name=data.var_name or data.name
    value=data.var_value or data.value
    if not name or value is None:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="var_name and var_value required")
    existing=await session.execute(select(Var).where(Var.domain_id==domain_id, Var.var_name==name, Var.var_category==(data.var_category or "Provision")))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Var already exists")
    obj=Var(domain_id=domain_id, var_category=data.var_category or "Provision", var_name=name, var_value=str(value), var_enabled=data.var_enabled if data.var_enabled is not None else (data.enabled if data.enabled is not None else True), var_description=data.var_description or data.description, var_command=data.var_command, var_hostname=data.var_hostname, var_order=data.var_order if data.var_order is not None else 0)
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_vars(session: AsyncSession, domain_id:UUID, page:int, per_page:int, q:Optional[str]):
    query=select(Var).where(Var.domain_id==domain_id)
    count_q=select(func.count()).select_from(Var).where(Var.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(Var.var_name.ilike(like), Var.var_value.ilike(like)))
        count_q=count_q.where(or_(Var.var_name.ilike(like), Var.var_value.ilike(like)))
    query=query.order_by(Var.var_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
