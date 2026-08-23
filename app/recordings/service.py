from typing import Optional
from uuid import UUID, uuid4

from sqlalchemy import select, func, or_
from sqlalchemy.ext.asyncio import AsyncSession

from app.recordings.models import Recording

async def create_recording(session: AsyncSession, data, domain_id: UUID, user_id: Optional[UUID] = None) -> Recording:
    # ensure partitioned key: recordings/{domain_id}/{uuid}.wav if not provided
    from app.storage.s3 import s3_key_for_recording
    rec_uuid = uuid4()
    # allow uuid7 via model default; but compute key if needed
    file_path = getattr(data, "file_path", None) or getattr(data, "file_path", None)
    # data may be dict or BaseModel
    if isinstance(data, dict):
        name = data.get("name") or "recording"
        file_path = data.get("file_path")
        duration = data.get("duration")
        file_size = data.get("file_size")
        s3_bucket = data.get("s3_bucket")
        content_type = data.get("content_type") or "audio/wav"
        storage_backend = data.get("storage_backend") or "s3"
    else:
        name = data.name
        file_path = data.file_path
        duration = data.duration
        file_size = data.file_size
        s3_bucket = data.s3_bucket
        content_type = data.content_type or "audio/wav"
        storage_backend = data.storage_backend or "s3"
    if not file_path:
        # generate partitioned key
        suffix = "wav"
        if name and "." in name:
            suffix = name.rsplit(".", 1)[-1][:10]
        tmp_uuid = rec_uuid
        file_path = s3_key_for_recording(domain_id, tmp_uuid, suffix)
    # enforce partition prefix
    if not file_path.startswith(f"recordings/{domain_id}"):
        # if not partitioned, rewrite to partitioned path preserving filename
        fname = file_path.split("/")[-1] or f"{rec_uuid}.wav"
        file_path = f"recordings/{domain_id}/{fname}"
    obj = Recording(
        domain_id=domain_id,
        name=name,
        file_path=file_path,
        duration=duration,
        file_size=file_size,
        s3_bucket=s3_bucket,
        content_type=content_type,
        storage_backend=storage_backend,
        created_by=user_id,
    )
    # allow manual uuid for deterministic key alignment
    # if we generated file_path with tmp_uuid, align obj uuid
    if not file_path.startswith(f"recordings/{domain_id}/{rec_uuid}"):
        pass
    session.add(obj)
    await session.commit()
    await session.refresh(obj)
    return obj

async def list_recordings(session: AsyncSession, domain_id: UUID, page: int, per_page: int, q: Optional[str] = None, sort: str = "created_at", order: str = "desc"):
    query = select(Recording).where(Recording.domain_id == domain_id)
    count_q = select(func.count()).select_from(Recording).where(Recording.domain_id == domain_id)
    if q:
        like = f"%{q}%"
        query = query.where(or_(Recording.name.ilike(like), Recording.file_path.ilike(like)))
        count_q = count_q.where(or_(Recording.name.ilike(like), Recording.file_path.ilike(like)))
    # sort
    allowed = {"name": Recording.name, "created_at": Recording.created_at, "file_path": Recording.file_path, "duration": Recording.duration}
    col = allowed.get(sort, Recording.created_at)
    if order == "desc":
        query = query.order_by(col.desc())
    else:
        query = query.order_by(col.asc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page - 1) * per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total

def presigned_for(recording: Recording, expires: int = 3600) -> str:
    from app.storage.s3 import presigned_get_url
    bucket = recording.s3_bucket
    return presigned_get_url(recording.file_path, expires=expires, bucket=bucket)
