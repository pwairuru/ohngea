from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.phrases.models import Phrase, PhraseDetail
from app.phrases.schemas import PhraseCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session

async def create_phrase(session: AsyncSession, data: PhraseCreate, domain_id: UUID) -> Phrase:
    name = data.phrase_name or data.name
    if not name:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="phrase_name required")
    existing = await session.execute(select(Phrase).where(Phrase.domain_id==domain_id, Phrase.phrase_name==name))
    if existing.scalar_one_or_none():
        from fastapi import HTTPException
        raise HTTPException(status_code=409, detail="Phrase already exists")
    obj = Phrase(domain_id=domain_id, phrase_name=name, phrase_language=data.phrase_language or "en-us", phrase_enabled=data.phrase_enabled if data.phrase_enabled is not None else (data.enabled if data.enabled is not None else True), phrase_description=data.phrase_description or data.description)
    session.add(obj)
    await session.flush()
    for d in (data.details or []):
        if isinstance(d, dict):
            det = PhraseDetail(domain_id=domain_id, phrase_uuid=obj.phrase_uuid, phrase_detail_function=d.get("function") or d.get("phrase_detail_function"), phrase_detail_data=d.get("data") or d.get("phrase_detail_data"), phrase_detail_order=int(d.get("order",0) or 0))
            session.add(det)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj

async def list_phrases(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(Phrase).where(Phrase.domain_id==domain_id)
    count_q=select(func.count()).select_from(Phrase).where(Phrase.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(Phrase.phrase_name.ilike(like), Phrase.phrase_description.ilike(like)))
        count_q=count_q.where(or_(Phrase.phrase_name.ilike(like), Phrase.phrase_description.ilike(like)))
    query=query.order_by(Phrase.phrase_name.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total

async def get_phrase_with_details(session: AsyncSession, pid: UUID, domain_id: UUID):
    r=await session.execute(select(Phrase).where(Phrase.phrase_uuid==pid, Phrase.domain_id==domain_id))
    ph=r.scalar_one_or_none()
    if not ph: return None, []
    dests=(await session.execute(select(PhraseDetail).where(PhraseDetail.phrase_uuid==pid))).scalars().all()
    return ph, dests
