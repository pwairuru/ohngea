from typing import Optional, List, Any
from uuid import UUID

from fastapi import HTTPException
from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.voicemails.models import Voicemail, VoicemailDestination


async def create_voicemail(
    session: AsyncSession,
    data,
    domain_id: UUID,
) -> Voicemail:
    if hasattr(data, "model_dump"):
        payload = data.model_dump(exclude_unset=True)
    elif isinstance(data, dict):
        payload = dict(data)
    else:
        payload = {}

    voicemail_id = payload.get("voicemail_id")
    voicemail_password = payload.get("voicemail_password")
    if not voicemail_id:
        raise HTTPException(status_code=422, detail="voicemail_id is required")
    if not voicemail_password:
        raise HTTPException(status_code=422, detail="voicemail_password is required")

    # duplicate check per domain
    existing = await session.execute(
        select(Voicemail).where(Voicemail.domain_id == domain_id, Voicemail.voicemail_id == str(voicemail_id))
    )
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Voicemail mailbox already exists")

    # extract forward destinations before model creation
    forward_raw = payload.pop("forward_destinations", None)
    # also handle alias destinations if still present
    if "destinations" in payload:
        alt = payload.pop("destinations", None)
        if alt and not forward_raw:
            forward_raw = alt
        elif alt and forward_raw:
            # merge
            if isinstance(alt, list) and isinstance(forward_raw, list):
                forward_raw = forward_raw + alt

    # ensure domain_id matches tenant
    payload["domain_id"] = domain_id
    # sanitize empty strings to None for optional fields
    for k in ("toll_allow", "voicemail_mail_to", "voicemail_file", "alternate_greet_id", "description"):
        if payload.get(k) == "":
            payload[k] = None
    if payload.get("greeting_id") == "":
        payload["greeting_id"] = None

    vm = Voicemail(**{k: v for k, v in payload.items() if k in Voicemail.model_fields})
    # handle extra fields like greeting_id etc correctly via model_fields check? but Voicemail has those so ok
    # However need to handle that some payload keys may not be in model_fields (e.g. extra allow) - filter
    session.add(vm)
    await session.flush()

    # create destinations if provided
    if forward_raw:
        for item in forward_raw:
            copy_uuid = None
            if isinstance(item, str):
                s = item.strip()
                if not s:
                    continue
                # try UUID parse
                try:
                    copy_uuid = UUID(s)
                except Exception:
                    # try lookup by voicemail_id within same domain
                    # if it's a mailbox number like "1002", find its voicemail_uuid
                    r = await session.execute(
                        select(Voicemail).where(Voicemail.domain_id == domain_id, Voicemail.voicemail_id == s)
                    )
                    found = r.scalar_one_or_none()
                    if found:
                        copy_uuid = found.voicemail_uuid
                    else:
                        continue
            elif isinstance(item, dict):
                # dict with voicemail_uuid_copy or copy id
                val = item.get("voicemail_uuid_copy") or item.get("voicemail_copy_uuid") or item.get("copy_uuid") or item.get("uuid") or item.get("voicemail_id")
                if not val:
                    continue
                try:
                    copy_uuid = UUID(str(val))
                except Exception:
                    # maybe voicemail_id
                    r = await session.execute(
                        select(Voicemail).where(Voicemail.domain_id == domain_id, Voicemail.voicemail_id == str(val))
                    )
                    found = r.scalar_one_or_none()
                    if found:
                        copy_uuid = found.voicemail_uuid
                    else:
                        continue
            else:
                # try UUID directly
                try:
                    copy_uuid = UUID(str(item))
                except Exception:
                    # pydantic model?
                    try:
                        d = item.model_dump() if hasattr(item, "model_dump") else dict(item)
                        val = d.get("voicemail_uuid_copy") or d.get("copy_uuid")
                        if val:
                            copy_uuid = UUID(str(val))
                    except Exception:
                        continue
            if copy_uuid:
                # avoid self-copy?
                if copy_uuid == vm.voicemail_uuid:
                    continue
                dest = VoicemailDestination(
                    domain_id=domain_id,
                    voicemail_uuid=vm.voicemail_uuid,
                    voicemail_uuid_copy=copy_uuid,
                )
                session.add(dest)

    await session.commit()
    await session.refresh(vm)
    return vm


async def list_voicemails(
    session: AsyncSession,
    domain_id: UUID,
    page: int = 1,
    per_page: int = 20,
    q: Optional[str] = None,
    sort: str = "voicemail_id",
    order: str = "asc",
    filter_voicemail_id: Optional[str] = None,
    filter_enabled: Optional[bool] = None,
    filter_description: Optional[str] = None,
) -> tuple[List[Voicemail], int]:
    per_page = min(max(per_page, 1), 100)
    offset = (page - 1) * per_page
    base = select(Voicemail).where(Voicemail.domain_id == domain_id)

    if q:
        ilike = f"%{q}%"
        base = base.where(
            or_(
                Voicemail.voicemail_id.ilike(ilike),
                Voicemail.description.ilike(ilike),
                Voicemail.voicemail_mail_to.ilike(ilike),
                Voicemail.toll_allow.ilike(ilike),
            )
        )
    if filter_voicemail_id:
        base = base.where(Voicemail.voicemail_id.ilike(f"%{filter_voicemail_id}%"))
    if filter_enabled is not None:
        base = base.where(Voicemail.enabled == filter_enabled)
    if filter_description:
        base = base.where(Voicemail.description.ilike(f"%{filter_description}%"))

    count_q = select(func.count()).select_from(base.subquery())
    total = (await session.execute(count_q)).scalar() or 0

    allowed_sort = {
        "voicemail_id": Voicemail.voicemail_id,
        "mailbox": Voicemail.voicemail_id,
        "extension": Voicemail.voicemail_id,
        "voicemail_password": Voicemail.voicemail_password,
        "enabled": Voicemail.enabled,
        "description": Voicemail.description,
        "created_at": Voicemail.created_at,
        "updated_at": Voicemail.updated_at,
        "greeting_id": Voicemail.greeting_id,
    }
    col = allowed_sort.get(sort, Voicemail.voicemail_id)
    if order == "desc":
        base = base.order_by(col.desc())
    else:
        base = base.order_by(col.asc())

    base = base.offset(offset).limit(per_page)
    result = await session.execute(base)
    return result.scalars().all(), total


async def get_voicemail(session: AsyncSession, voicemail_uuid: UUID, domain_id: UUID) -> Optional[Voicemail]:
    r = await session.execute(select(Voicemail).where(Voicemail.voicemail_uuid == voicemail_uuid, Voicemail.domain_id == domain_id))
    return r.scalar_one_or_none()


async def get_voicemail_with_destinations(session: AsyncSession, voicemail_uuid: UUID, domain_id: UUID):
    vm = await get_voicemail(session, voicemail_uuid, domain_id)
    if not vm:
        return None, []
    d = await session.execute(
        select(VoicemailDestination).where(
            VoicemailDestination.voicemail_uuid == voicemail_uuid,
            VoicemailDestination.domain_id == domain_id,
        )
    )
    dests = d.scalars().all()
    return vm, dests
