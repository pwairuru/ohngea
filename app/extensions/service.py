from uuid import UUID
import secrets
import string
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from fastapi import HTTPException

from app.extensions.models import Extension
from app.extensions.schemas import ExtensionCreate, ExtensionUpdate


def generate_password(length: int = 20) -> str:
    alphabet = string.ascii_letters + string.digits + "!@#$%^&*"
    # ensure strength: include at least one of each category if length >=4
    if length >= 4:
        pwd = [
            secrets.choice(string.ascii_lowercase),
            secrets.choice(string.ascii_uppercase),
            secrets.choice(string.digits),
            secrets.choice("!@#$%^&*"),
        ]
        pwd += [secrets.choice(alphabet) for _ in range(length - 4)]
        secrets.SystemRandom().shuffle(pwd)
        return "".join(pwd)
    return "".join(secrets.choice(alphabet) for _ in range(length))


async def _invalidate_fs_cache(domain_id: UUID, domain_name: str | None = None):
    # Best-effort invalidation of all FS xml keys for this domain
    try:
        try:
            import valkey.asyncio as redis  # type: ignore[import-not-found]
        except ImportError:
            import redis.asyncio as redis  # type: ignore[no-redef]
        from app.config import settings
        url = getattr(settings, "valkey_url", settings.redis_url)
        r = redis.from_url(url, decode_responses=True)
        # Legacy key
        await r.delete(f"xml:{domain_id}:directory")
        # New keys: xml:directory:<domain> and xml:directory:<ext>@<domain> and dialplan
        if domain_name:
            # Use SCAN to avoid blocking
            for pattern in [f"xml:directory:*@{domain_name}", f"xml:directory:{domain_name}", f"xml:dialplan:{domain_name}", f"xml:directory:{domain_name}:*", f"xml:*:{domain_name}"]:
                try:
                    cursor = "0"
                    while True:
                        cursor, keys = await r.scan(cursor=cursor, match=pattern, count=100)
                        if keys:
                            await r.delete(*keys)
                        if cursor == "0" or cursor == 0:
                            break
                except Exception:
                    # fallback direct delete
                    try:
                        await r.delete(pattern)
                    except Exception:
                        pass
            # also generic directory/dialplan keys for domain_name
            try:
                await r.delete(f"xml:directory:{domain_name}", f"xml:dialplan:{domain_name}")
            except Exception:
                pass
        else:
            # fallback scan all xml keys for this domain_id string
            try:
                await r.delete(f"xml:directory:{domain_id}", f"xml:dialplan:{domain_id}")
            except Exception:
                pass
        try:
            await r.aclose()
        except Exception:
            try:
                await r.close()
            except Exception:
                pass
    except Exception:
        pass
    # ESL reloadxml best-effort
    try:
        from app.freeswitch.esl_client import esl_client
        await esl_client.reloadxml()
    except Exception:
        pass


async def create_extension(session: AsyncSession, data: ExtensionCreate) -> Extension:
    # check duplicate extension per domain
    existing = await session.execute(select(Extension).where(Extension.domain_id == data.domain_id, Extension.extension == data.extension))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Extension already exists")
    # also check number_alias duplicate if provided
    if data.number_alias:
        existing_alias = await session.execute(select(Extension).where(Extension.domain_id == data.domain_id, Extension.number_alias == data.number_alias))
        if existing_alias.scalar_one_or_none():
            raise HTTPException(status_code=409, detail="Number alias already exists")
    dump = data.model_dump()
    # generate password if missing
    if not dump.get("password"):
        dump["password"] = generate_password(20)
    # fill user_context default to domain_name if missing — need lookup
    if not dump.get("user_context") and dump.get("domain_id"):
        try:
            from app.domains.models import Domain
            r = await session.execute(select(Domain).where(Domain.domain_id == dump["domain_id"]))
            dom = r.scalar_one_or_none()
            if dom:
                dump["user_context"] = dom.domain_name
        except Exception:
            pass
    # normalize CIDR comma handling etc could be added
    ext = Extension(**dump)
    session.add(ext)
    await session.commit()
    await session.refresh(ext)
    # invalidate cache + ESL
    domain_name = dump.get("user_context")
    if not domain_name:
        try:
            from app.domains.models import Domain
            r = await session.execute(select(Domain).where(Domain.domain_id == ext.domain_id))
            dom = r.scalar_one_or_none()
            domain_name = dom.domain_name if dom else None
        except Exception:
            domain_name = None
    await _invalidate_fs_cache(ext.domain_id, domain_name)
    return ext


async def list_extensions(
    session: AsyncSession, domain_id: UUID, page: int = 1, per_page: int = 20, q: str | None = None, sort: str = "extension", order: str = "asc"
) -> tuple[list[Extension], int]:
    per_page = min(max(per_page, 1), 100)
    offset = (page - 1) * per_page
    base = select(Extension).where(Extension.domain_id == domain_id)
    if q:
        base = base.where((Extension.extension.ilike(f"%{q}%")) | (Extension.effective_caller_id_name.ilike(f"%{q}%")) | (Extension.number_alias.ilike(f"%{q}%")) | (Extension.description.ilike(f"%{q}%")))
    count_q = select(func.count()).select_from(base.subquery())
    total = (await session.execute(count_q)).scalar() or 0
    # sorting — allowlist to avoid injection
    allowed = {"extension","effective_caller_id_name","created_at","updated_at","enabled","number_alias","call_timeout"}
    if sort not in allowed:
        sort = "extension"
    col = getattr(Extension, sort, Extension.extension)
    if order == "desc":
        base = base.order_by(col.desc())
    else:
        base = base.order_by(col.asc())
    base = base.offset(offset).limit(per_page)
    result = await session.execute(base)
    return result.scalars().all(), total
