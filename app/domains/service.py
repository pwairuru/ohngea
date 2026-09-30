from uuid import UUID

from fastapi import HTTPException
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.auth.models import Group, GroupPermission, Permission, User, UserGroup
from app.auth.service import hash_password
from app.domains.models import Domain
from app.domains.schemas import DomainCreate
from app.freeswitch.cache import invalidate_fs_cache_for_session


async def create_domain(
    session: AsyncSession,
    data: DomainCreate,
    admin_username: str = "admin",
    admin_password: str = "admin",
) -> Domain:
    # check duplicate
    existing = await session.execute(select(Domain).where(Domain.domain_name == data.domain_name))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Domain already exists")
    domain = Domain(
        domain_name=data.domain_name, enabled=data.enabled, description=data.description
    )
    session.add(domain)
    await session.flush()  # get domain_id

    # seed global permissions (full 1036-name union) + per-domain settings/vars/sip/acl/conf/moh/dialplans
    # so fresh bootstraps seed a working PBX out of box. All idempotent.
    from app.seeds.dialplans import seed_default_dialplans
    from app.seeds.domain_settings import seed_domain_settings
    from app.seeds.operational import (
        seed_access_controls,
        seed_conference_profiles,
        seed_music_on_hold,
    )
    from app.seeds.permissions import ALL_PERMISSIONS
    from app.seeds.sip_profiles import seed_sip_profiles
    from app.seeds.vars import seed_vars

    for perm_name in ALL_PERMISSIONS:
        r = await session.execute(select(Permission).where(Permission.name == perm_name))
        if not r.scalar_one_or_none():
            session.add(Permission(name=perm_name))
    await session.flush()
    await seed_domain_settings(session, domain.domain_id)
    await seed_vars(session, domain.domain_id)

    # seed default groups with levels and protected flags (legacy parity)
    defaults = [
        ("superadmin", 80, True),
        ("admin", 50, True),
        ("user", 30, False),
        ("agent", 20, False),
        ("public", 10, False),
        ("gateway", 10, False),
    ]
    groups_map = {}
    for name, lvl, prot in defaults:
        r = await session.execute(
            select(Group).where(Group.name == name, Group.domain_id == domain.domain_id)
        )
        existing_g = r.scalar_one_or_none()
        if existing_g:
            # update level/protected to correct values if needed
            if existing_g.group_level != lvl or existing_g.group_protected != prot:
                existing_g.group_level = lvl
                existing_g.group_protected = prot
            groups_map[name] = existing_g
        else:
            g = Group(
                domain_id=domain.domain_id,
                name=name,
                description=name,
                group_level=lvl,
                group_protected=prot,
            )
            session.add(g)
            await session.flush()
            await session.refresh(g)
            groups_map[name] = g
    # ensure superadmin group exists for perms assignment
    group = groups_map.get("superadmin")
    if not group:
        group = Group(
            domain_id=domain.domain_id,
            name="superadmin",
            description="Super Admin",
            group_level=80,
            group_protected=True,
        )
        session.add(group)
        await session.flush()
        await session.refresh(group)
        groups_map["superadmin"] = group
    # assign all perms to superadmin with denorm fields
    r = await session.execute(select(Permission))
    for p in r.scalars().all():
        # check existing
        chk = await session.execute(
            select(GroupPermission).where(
                GroupPermission.group_uuid == group.group_uuid,
                GroupPermission.permission_uuid == p.permission_uuid,
            )
        )
        if not chk.scalar_one_or_none():
            session.add(
                GroupPermission(
                    group_uuid=group.group_uuid,
                    permission_uuid=p.permission_uuid,
                    domain_id=domain.domain_id,
                    group_name="superadmin",
                    permission_name=p.name,
                    permission_protected=False,
                    permission_assigned=True,
                )
            )
    # seed UserSetting for language/time_zone defaults if needed (domain_settings already has theme)
    # ensure language/time_zone domain_settings? Keep minimal for now

    user = User(
        domain_id=domain.domain_id,
        username=admin_username,
        password_hash=hash_password(admin_password),
        enabled=True,
    )
    session.add(user)
    await session.flush()
    session.add(
        UserGroup(
            user_uuid=user.user_uuid,
            group_uuid=group.group_uuid,
            domain_id=domain.domain_id,
            group_name="superadmin",
        )
    )

    # full operational seeds (upgrade stubs to full parity on re-seed)
    await seed_sip_profiles(session, domain.domain_id, domain.domain_name)
    await seed_access_controls(session, domain.domain_id)
    await seed_conference_profiles(session, domain.domain_id)
    await seed_music_on_hold(session, domain.domain_id)
    await seed_default_dialplans(session, domain.domain_id, domain.domain_name)

    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, domain.domain_id)
    except Exception:
        pass
    await session.refresh(domain)
    return domain


async def ensure_tenant_partition(session: AsyncSession, domain_id: UUID):
    # For HASH partitioning, no per-tenant table creation needed, but validate.
    # Future LIST strategy would create partition here.
    pass
