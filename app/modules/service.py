from typing import Optional
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.modules.models import Module
from app.modules.schemas import ModuleCreate
async def create_mod(session: AsyncSession, data: ModuleCreate):
    label=data.module_label or data.label
    name=data.module_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="module_name required")
    existing=await session.execute(select(Module).where(Module.module_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Module already exists")
    obj=Module(module_label=label or name, module_name=name, module_category=data.module_category, module_enabled=data.module_enabled if data.module_enabled is not None else (data.enabled if data.enabled is not None else True), module_description=data.module_description or data.description)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_mods(session: AsyncSession, page:int, per_page:int, q:Optional[str]):
    query=select(Module)
    count_q=select(func.count()).select_from(Module)
    if q:
        like=f"%{q}%"
        query=query.where(or_(Module.module_name.ilike(like), Module.module_label.ilike(like)))
        count_q=count_q.where(or_(Module.module_name.ilike(like), Module.module_label.ilike(like)))
    query=query.order_by(Module.module_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
