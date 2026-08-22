from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.extension_settings.models import ExtensionSetting
from app.extension_settings.schemas import ExtensionSettingCreate
async def create_es(session: AsyncSession, data: ExtensionSettingCreate, domain_id: UUID):
    if not data.extension_uuid:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="extension_uuid required")
    if not (data.extension_setting_name or data.name):
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="extension_setting_name required")
    obj=ExtensionSetting(domain_id=domain_id, extension_uuid=data.extension_uuid, extension_setting_type=data.extension_setting_type, extension_setting_name=data.extension_setting_name or data.name, extension_setting_value=str(data.extension_setting_value or data.value or ""), extension_setting_enabled=data.extension_setting_enabled if data.extension_setting_enabled is not None else (data.enabled if data.enabled is not None else True), extension_setting_description=data.extension_setting_description or data.description)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_es(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str], extension_uuid: Optional[UUID]=None):
    query=select(ExtensionSetting).where(ExtensionSetting.domain_id==domain_id)
    count_q=select(func.count()).select_from(ExtensionSetting).where(ExtensionSetting.domain_id==domain_id)
    if extension_uuid:
        query=query.where(ExtensionSetting.extension_uuid==extension_uuid)
        count_q=count_q.where(ExtensionSetting.extension_uuid==extension_uuid)
    if q:
        like=f"%{q}%"
        query=query.where(or_(ExtensionSetting.extension_setting_name.ilike(like), ExtensionSetting.extension_setting_value.ilike(like)))
        count_q=count_q.where(or_(ExtensionSetting.extension_setting_name.ilike(like), ExtensionSetting.extension_setting_value.ilike(like)))
    query=query.order_by(ExtensionSetting.extension_setting_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
