from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.feature_codes.models import FeatureCode
from app.feature_codes.schemas import FeatureCodeCreate
async def create_fc(session: AsyncSession, data: FeatureCodeCreate, domain_id: UUID):
    name=data.feature_code_name or data.name
    val=data.feature_code_value or data.value
    if not name or not val:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="feature_code_name and value required")
    existing=await session.execute(select(FeatureCode).where(FeatureCode.domain_id==domain_id, FeatureCode.feature_code_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Feature code already exists")
    obj=FeatureCode(domain_id=domain_id, feature_code_name=name, feature_code_value=str(val), feature_code_enabled=data.feature_code_enabled if data.feature_code_enabled is not None else (data.enabled if data.enabled is not None else True), feature_code_description=data.feature_code_description or data.description)
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj
async def list_fc(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(FeatureCode).where(FeatureCode.domain_id==domain_id)
    count_q=select(func.count()).select_from(FeatureCode).where(FeatureCode.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(FeatureCode.feature_code_name.ilike(like), FeatureCode.feature_code_value.ilike(like)))
        count_q=count_q.where(or_(FeatureCode.feature_code_name.ilike(like), FeatureCode.feature_code_value.ilike(like)))
    query=query.order_by(FeatureCode.feature_code_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
