"""Per-domain domain_settings defaults (theme + domain/email/extension essentials)."""
from __future__ import annotations

DOMAIN_DEFAULTS: list[tuple] = [('theme', '', 'primary_color', '221 83% 53%'), ('theme', '', 'background', '0 0% 100%'), ('theme', '', 'radius', '0.5rem'), ('domain', 'language', 'code', 'en-us'), ('domain', 'time_zone', 'text', 'UTC'), ('domain', 'country', 'code', 'us'), ('domain', 'bridge', 'text', 'loopback'), ('domain', 'paging', 'numeric', '100'), ('domain', 'template', 'text', 'default'), ('domain', 'menu', 'uuid', 'b4750c3f-2a86-b00d-b7d0-345c14eca286'), ('domain', 'country_code', 'numeric', ''), ('extension', 'password_length', 'numeric', '20'), ('extension', 'password_number', 'boolean', 'true'), ('email', 'method', 'text', 'smtp')]

async def seed_domain_settings(session, domain_id) -> int:
    from sqlalchemy import select as _select
    from app.domains.models import DomainSettings as _Ds
    n = 0
    for cat, sub, k, v in DOMAIN_DEFAULTS:
        r = await session.execute(_select(_Ds).where(_Ds.domain_id == domain_id, _Ds.category == cat, _Ds.subcategory == sub, _Ds.key == k))
        if r.scalar_one_or_none(): continue
        session.add(_Ds(domain_id=domain_id, category=cat, subcategory=sub, key=k, value=v)); n += 1
    await session.flush()
    return n
