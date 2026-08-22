from typing import Optional
from uuid import UUID

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.follow_me.models import FollowMe, FollowMeDestination
from app.follow_me.schemas import FollowMeCreate


async def create_follow_me(session: AsyncSession, data: FollowMeCreate, domain_id: UUID):
    fm = FollowMe(
        domain_id=domain_id,
        follow_me_enabled=data.follow_me_enabled if data.follow_me_enabled is not None else (data.enabled if data.enabled is not None else True),
        cid_name_prefix=data.cid_name_prefix,
        cid_number_prefix=data.cid_number_prefix,
        call_prompt=data.call_prompt,
        dial_string=data.dial_string,
        description=data.description,
    )
    session.add(fm)
    await session.flush()
    # destinations
    for dest in (data.destinations or []):
        if isinstance(dest, str):
            d = FollowMeDestination(domain_id=domain_id, follow_me_uuid=fm.follow_me_uuid, follow_me_destination=dest)
            session.add(d)
        elif isinstance(dest, dict):
            num = dest.get("follow_me_destination") or dest.get("destination") or dest.get("number")
            if num:
                d = FollowMeDestination(
                    domain_id=domain_id,
                    follow_me_uuid=fm.follow_me_uuid,
                    follow_me_destination=str(num),
                    follow_me_delay=int(dest.get("follow_me_delay", 0) or 0),
                    follow_me_timeout=int(dest.get("follow_me_timeout", 30) or 30),
                    follow_me_order=int(dest.get("follow_me_order", 0) or 0),
                )
                session.add(d)
    await session.commit()
    await session.refresh(fm)
    return fm


async def list_follow_me(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str]):
    query = select(FollowMe).where(FollowMe.domain_id == domain_id)
    count_q = select(func.count()).select_from(FollowMe).where(FollowMe.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(FollowMe.description.ilike(like), FollowMe.cid_name_prefix.ilike(like)))
        count_q = count_q.where(or_(FollowMe.description.ilike(like), FollowMe.cid_name_prefix.ilike(like)))
    query = query.order_by(FollowMe.created_at.desc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total


async def get_follow_me_with_dests(session: AsyncSession, fm_uuid: UUID, domain_id: UUID):
    r = await session.execute(select(FollowMe).where(FollowMe.follow_me_uuid == fm_uuid, FollowMe.domain_id == domain_id))
    fm = r.scalar_one_or_none()
    if not fm:
        return None, []
    dests = (await session.execute(select(FollowMeDestination).where(FollowMeDestination.follow_me_uuid == fm_uuid))).scalars().all()
    return fm, dests
