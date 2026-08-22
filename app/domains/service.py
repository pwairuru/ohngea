from uuid import UUID
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from fastapi import HTTPException

from app.domains.models import Domain, DomainSettings
from app.domains.schemas import DomainCreate
from app.auth.models import User, Group, Permission, GroupPermission, UserGroup
from app.auth.service import hash_password


async def create_domain(session: AsyncSession, data: DomainCreate, admin_username: str = "admin", admin_password: str = "admin") -> Domain:
    # check duplicate
    existing = await session.execute(select(Domain).where(Domain.domain_name == data.domain_name))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Domain already exists")
    domain = Domain(domain_name=data.domain_name, enabled=data.enabled, description=data.description)
    session.add(domain)
    await session.flush()  # get domain_id

    # seed default domain_settings (minimal)
    defaults = [
        ("theme", "", "primary_color", "221 83% 53%"),
        ("theme", "", "background", "0 0% 100%"),
        ("theme", "", "radius", "0.5rem"),
    ]
    for cat, sub, k, v in defaults:
        session.add(DomainSettings(domain_id=domain.domain_id, category=cat, subcategory=sub, key=k, value=v))

    # seed default permissions if not exist
    for perm_name in [
        "domain_view", "domain_add", "extension_view", "extension_add", "dialplan_view", "gateway_view",
        "ring_group_view", "ring_group_add",
        "time_condition_view", "time_condition_add", "time_condition_edit", "time_condition_delete",
    ]:
        r = await session.execute(select(Permission).where(Permission.name == perm_name))
        if not r.scalar_one_or_none():
            session.add(Permission(name=perm_name))

    await session.flush()
    # create admin group + user
    group = Group(domain_id=domain.domain_id, name="superadmin", description="Super Admin")
    session.add(group)
    await session.flush()
    # assign all perms to superadmin
    r = await session.execute(select(Permission))
    for p in r.scalars().all():
        session.add(GroupPermission(group_uuid=group.group_uuid, permission_uuid=p.permission_uuid))

    user = User(domain_id=domain.domain_id, username=admin_username, password_hash=hash_password(admin_password), enabled=True)
    session.add(user)
    await session.flush()
    session.add(UserGroup(user_uuid=user.user_uuid, group_uuid=group.group_uuid, domain_id=domain.domain_id))

    # seed default SIP profiles (tenant-aware, idempotent)
    from app.sip_profiles.models import SipProfile

    for name, port in [("internal", 5060), ("external", 5080)]:
        r = await session.execute(select(SipProfile).where(SipProfile.domain_id == domain.domain_id, SipProfile.name == name))
        if not r.scalar_one_or_none():
            session.add(SipProfile(domain_id=domain.domain_id, name=name, enabled=True, sip_port=port))

    await session.commit()
    await session.refresh(domain)
    return domain


async def ensure_tenant_partition(session: AsyncSession, domain_id: UUID):
    # For HASH partitioning, no per-tenant table creation needed, but validate.
    # Future LIST strategy would create partition here.
    pass
