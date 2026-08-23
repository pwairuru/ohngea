from uuid import UUID
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from fastapi import HTTPException

from app.domains.models import Domain, DomainSettings
from app.domains.schemas import DomainCreate
from app.auth.models import User, Group, Permission, GroupPermission, UserGroup
from app.auth.service import hash_password
from app.freeswitch.cache import invalidate_fs_cache, invalidate_fs_cache_for_session


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

    # seed default permissions if not exist (comprehensive list for parity)
    all_perm_names = [
        "domain_view", "domain_add", "domain_edit", "domain_delete", "domain_all",
        "extension_view", "extension_add", "extension_edit", "extension_delete",
        "dialplan_view", "dialplan_add", "dialplan_edit", "dialplan_delete",
        "gateway_view", "gateway_add", "gateway_edit", "gateway_delete",
        "sip_profile_view", "sip_profile_add", "sip_profile_edit", "sip_profile_delete",
        "ivr_menu_view", "ivr_menu_add", "ivr_menu_edit", "ivr_menu_delete",
        "ring_group_view", "ring_group_add", "ring_group_edit", "ring_group_delete", "ring_group_forward", "ring_group_missed_call",
        "time_condition_view", "time_condition_add", "time_condition_edit", "time_condition_delete",
        "voicemail_view", "voicemail_add", "voicemail_edit", "voicemail_delete",
        "conference_view", "conference_add", "conference_edit", "conference_delete",
        "fax_view", "fax_add", "fax_edit", "fax_delete",
        "call_flow_view", "call_flow_add", "call_flow_edit", "call_flow_delete",
        "group_view", "group_add", "group_edit", "group_delete", "group_domain", "group_all", "group_global", "group_member_view", "group_member_add", "group_member_delete", "group_permissions", "group_permission_view", "group_permission_add", "group_permission_edit", "group_permission_delete", "view_groups",
        "user_view", "user_add", "user_edit", "user_delete", "user_all", "user_domain", "user_group_add", "user_group_delete", "user_group_view", "user_setting_view", "user_password", "user_import", "user_type", "api_key", "message_key",
        "contact_view", "contact_add", "contact_edit", "contact_delete",
        "device_view", "device_add", "device_edit", "device_delete", "device_provision",
        "cdr_view", "xml_cdr_view",
        "music_on_hold_view", "follow_me_view", "call_block_view", "destination_view",
    ]
    for perm_name in all_perm_names:
        r = await session.execute(select(Permission).where(Permission.name == perm_name))
        if not r.scalar_one_or_none():
            session.add(Permission(name=perm_name))
    await session.flush()

    # seed default groups with levels and protected flags (legacy parity)
    defaults = [("superadmin", 80, True), ("admin", 50, True), ("user", 30, False), ("agent", 20, False), ("public", 10, False), ("gateway", 10, False)]
    groups_map = {}
    for name, lvl, prot in defaults:
        r = await session.execute(select(Group).where(Group.name == name, Group.domain_id == domain.domain_id))
        existing_g = r.scalar_one_or_none()
        if existing_g:
            # update level/protected to correct values if needed
            if existing_g.group_level != lvl or existing_g.group_protected != prot:
                existing_g.group_level = lvl
                existing_g.group_protected = prot
            groups_map[name] = existing_g
        else:
            g = Group(domain_id=domain.domain_id, name=name, description=name, group_level=lvl, group_protected=prot)
            session.add(g)
            await session.flush()
            await session.refresh(g)
            groups_map[name] = g
    # ensure superadmin group exists for perms assignment
    group = groups_map.get("superadmin")
    if not group:
        group = Group(domain_id=domain.domain_id, name="superadmin", description="Super Admin", group_level=80, group_protected=True)
        session.add(group)
        await session.flush()
        await session.refresh(group)
        groups_map["superadmin"] = group
    # assign all perms to superadmin with denorm fields
    r = await session.execute(select(Permission))
    for p in r.scalars().all():
        # check existing
        chk = await session.execute(select(GroupPermission).where(GroupPermission.group_uuid == group.group_uuid, GroupPermission.permission_uuid == p.permission_uuid))
        if not chk.scalar_one_or_none():
            session.add(GroupPermission(
                group_uuid=group.group_uuid,
                permission_uuid=p.permission_uuid,
                domain_id=domain.domain_id,
                group_name="superadmin",
                permission_name=p.name,
                permission_protected=False,
                permission_assigned=True,
            ))
    # seed UserSetting for language/time_zone defaults if needed (domain_settings already has theme)
    # ensure language/time_zone domain_settings? Keep minimal for now

    user = User(domain_id=domain.domain_id, username=admin_username, password_hash=hash_password(admin_password), enabled=True)
    session.add(user)
    await session.flush()
    session.add(UserGroup(user_uuid=user.user_uuid, group_uuid=group.group_uuid, domain_id=domain.domain_id, group_name="superadmin"))

    # seed default SIP profiles (tenant-aware, idempotent)
    from app.sip_profiles.models import SipProfile

    for name, port in [("internal", 5060), ("external", 5080)]:
        r = await session.execute(select(SipProfile).where(SipProfile.domain_id == domain.domain_id, SipProfile.name == name))
        if not r.scalar_one_or_none():
            session.add(SipProfile(domain_id=domain.domain_id, name=name, enabled=True, sip_port=port))

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
