from typing import Optional
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.sofia_global_settings.models import SofiaGlobalSetting
from app.sofia_global_settings.schemas import SofiaGlobalSettingCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_sgs(session: AsyncSession, data: SofiaGlobalSettingCreate):
    name=data.global_setting_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="global_setting_name required")
    existing=await session.execute(select(SofiaGlobalSetting).where(SofiaGlobalSetting.global_setting_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Setting already exists")
    obj=SofiaGlobalSetting(global_setting_name=name, global_setting_value=str(data.global_setting_value or data.value or ""), global_setting_enabled=data.global_setting_enabled if data.global_setting_enabled is not None else (data.enabled if data.enabled is not None else True), global_setting_description=data.global_setting_description or data.description)
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache()
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_sgs(session: AsyncSession, page:int, per_page:int, q:Optional[str]):
    query=select(SofiaGlobalSetting)
    count_q=select(func.count()).select_from(SofiaGlobalSetting)
    if q:
        like=f"%{q}%"
        query=query.where(or_(SofiaGlobalSetting.global_setting_name.ilike(like), SofiaGlobalSetting.global_setting_value.ilike(like)))
        count_q=count_q.where(or_(SofiaGlobalSetting.global_setting_name.ilike(like), SofiaGlobalSetting.global_setting_value.ilike(like)))
    query=query.order_by(SofiaGlobalSetting.global_setting_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
