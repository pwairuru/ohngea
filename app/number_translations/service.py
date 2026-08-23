from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.number_translations.models import NumberTranslation
from app.number_translations.schemas import NumberTranslationCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_nt(session: AsyncSession, data: NumberTranslationCreate, domain_id:UUID):
    name=data.number_translation_name or data.name
    regex=data.number_translation_regex or data.regex
    replace=data.number_translation_replace or data.replace
    if not name or not regex or replace is None:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="name, regex and replace required")
    existing=await session.execute(select(NumberTranslation).where(NumberTranslation.domain_id==domain_id, NumberTranslation.number_translation_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Number translation already exists")
    obj=NumberTranslation(domain_id=domain_id, number_translation_name=name, number_translation_regex=str(regex), number_translation_replace=str(replace), number_translation_enabled=data.number_translation_enabled if data.number_translation_enabled is not None else (data.enabled if data.enabled is not None else True), number_translation_description=data.number_translation_description or data.description)
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_nt(session: AsyncSession, domain_id:UUID, page:int, per_page:int, q:Optional[str]):
    query=select(NumberTranslation).where(NumberTranslation.domain_id==domain_id)
    count_q=select(func.count()).select_from(NumberTranslation).where(NumberTranslation.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(NumberTranslation.number_translation_name.ilike(like), NumberTranslation.number_translation_regex.ilike(like)))
        count_q=count_q.where(or_(NumberTranslation.number_translation_name.ilike(like), NumberTranslation.number_translation_regex.ilike(like)))
    query=query.order_by(NumberTranslation.number_translation_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
