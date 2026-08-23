from typing import Optional
from uuid import UUID
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession
from app.voicemail_greetings.models import VoicemailGreeting
from app.voicemail_greetings.schemas import VoicemailGreetingCreate
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session
async def create_greeting(session: AsyncSession, data: VoicemailGreetingCreate, domain_id: UUID):
    vm_id=data.voicemail_id or data.mailbox
    if not vm_id:
        from fastapi import HTTPException
        raise HTTPException(status_code=400, detail="voicemail_id required")
    obj=VoicemailGreeting(domain_id=domain_id, voicemail_id=str(vm_id), greeting_id=data.greeting_id or 1, greeting_name=data.greeting_name or data.name, greeting_filename=data.greeting_filename, greeting_description=data.greeting_description or data.description)
    session.add(obj)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain_id)
    except Exception:
        pass
    await session.refresh(obj)
    return obj
async def list_greetings(session: AsyncSession, domain_id: UUID, page:int, per_page:int, q:Optional[str]):
    query=select(VoicemailGreeting).where(VoicemailGreeting.domain_id==domain_id)
    count_q=select(func.count()).select_from(VoicemailGreeting).where(VoicemailGreeting.domain_id==domain_id)
    if q:
        like=f"%{q}%"
        query=query.where(or_(VoicemailGreeting.voicemail_id.ilike(like), VoicemailGreeting.greeting_name.ilike(like)))
        count_q=count_q.where(or_(VoicemailGreeting.voicemail_id.ilike(like), VoicemailGreeting.greeting_name.ilike(like)))
    query=query.order_by(VoicemailGreeting.voicemail_id.asc())
    total=(await session.execute(count_q)).scalar_one()
    query=query.offset((page-1)*per_page).limit(per_page)
    items=(await session.execute(query)).scalars().all()
    return items, total
