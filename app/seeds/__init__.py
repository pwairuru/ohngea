"""Out-of-box seed fixtures (extracted from legacy fusionpbx)."""

from __future__ import annotations


async def seed_global(session) -> dict:
    """Seed global (non-tenant) tables. Idempotent. Runs on `alembic upgrade head`."""
    from app.seeds.default_settings import seed_default_settings
    from app.seeds.permissions import seed_permissions
    from app.seeds.sofia import seed_sofia

    return {
        "default_settings": await seed_default_settings(session),
        "sofia": await seed_sofia(session),
        "permissions": await seed_permissions(session),
    }


async def seed_domain(session, domain_id, domain_name: str) -> dict:
    """Seed per-domain tables. Idempotent. Runs inside create_domain()."""
    from app.seeds.dialplans import seed_default_dialplans
    from app.seeds.domain_settings import seed_domain_settings
    from app.seeds.operational import (
        seed_access_controls,
        seed_conference_profiles,
        seed_music_on_hold,
    )
    from app.seeds.sip_profiles import seed_sip_profiles
    from app.seeds.vars import seed_vars

    return {
        "domain_settings": await seed_domain_settings(session, domain_id),
        "vars": await seed_vars(session, domain_id),
        "sip": await seed_sip_profiles(session, domain_id, domain_name),
        "acl": await seed_access_controls(session, domain_id),
        "conf": await seed_conference_profiles(session, domain_id),
        "moh": await seed_music_on_hold(session, domain_id),
        "dialplans": await seed_default_dialplans(session, domain_id, domain_name),
    }
