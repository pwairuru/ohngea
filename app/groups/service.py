from typing import Optional, List
from uuid import UUID, uuid4
import uuid6
from sqlalchemy import select, func, or_, and_, text, delete
from sqlalchemy.ext.asyncio import AsyncSession
from fastapi import HTTPException
from app.auth.models import Group, User, UserGroup, Permission, GroupPermission
from app.freeswitch.cache import invalidate_fs_cache_for_session


def _is_superadmin(user) -> bool:
    perms = getattr(user, "_perms", None)
    if perms is not None:
        return "*" in perms
    return False

def uuid7():
    return uuid6.uuid7()

async def get_user_level(session: AsyncSession, user) -> int:
    # max group_level of user's groups
    q = select(func.max(Group.group_level)).join(UserGroup, UserGroup.group_uuid == Group.group_uuid).where(UserGroup.user_uuid == user.user_uuid)
    r = await session.execute(q)
    lvl = r.scalar_one_or_none()
    if lvl is None:
        # fallback: if permissions contain "*", treat as 80 (superadmin)
        perms = getattr(user, "_perms", []) or []
        if "*" in perms:
            return 80
        return 0
    return int(lvl)

async def get_user_group_uuids(session: AsyncSession, user) -> set:
    q = select(Group.group_uuid).join(UserGroup, UserGroup.group_uuid == Group.group_uuid).where(UserGroup.user_uuid == user.user_uuid)
    r = await session.execute(q)
    return {row[0] for row in r.all()} if r else set()

async def list_groups(session: AsyncSession, domain_id: UUID, user, page: int, per_page: int, q: Optional[str]=None, show_all: bool=False, order_by: str="name", order: str="asc"):
    user_level = await get_user_level(session, user)
    # base filter: group_level <= user_level OR null
    base = and_(or_(Group.group_level <= user_level, Group.group_level == None))
    # domain filter
    if show_all:
        # no domain filter, show all (including global)
        domain_filt = True
    else:
        domain_filt = or_(Group.domain_id == domain_id, Group.domain_id == None)
    # combine
    query = select(Group).where(base, domain_filt)
    count_q = select(func.count()).select_from(Group).where(base, domain_filt)
    if q:
        like = f"%{q}%"
        filt = or_(Group.name.ilike(like), Group.description.ilike(like))
        query = query.where(filt)
        count_q = count_q.where(filt)
    # ordering
    col_map = {"group_name": Group.name, "name": Group.name, "group_level": Group.group_level, "group_protected": Group.group_protected, "description": Group.description}
    col = col_map.get(order_by, Group.name)
    if order == "desc":
        query = query.order_by(col.desc())
    else:
        query = query.order_by(col.asc())
    total = (await session.execute(count_q)).scalar_one()
    query = query.offset((page-1)*per_page).limit(per_page)
    items = (await session.execute(query)).scalars().all()
    return items, total

async def create_group(session: AsyncSession, domain_id: Optional[UUID], name: str, description: Optional[str], group_level: int=10, group_protected: bool=False, creator=None):
    # validate level 10..90 step 10
    if group_level not in [10,20,30,40,50,60,70,80,90]:
        raise HTTPException(status_code=422, detail="group_level must be one of 10,20,30,40,50,60,70,80,90")
    # duplicate check: same name + same domain_id (NULL considered equal)
    if domain_id is None:
        dup_q = select(Group).where(Group.name == name, Group.domain_id == None)
    else:
        dup_q = select(Group).where(Group.name == name, Group.domain_id == domain_id)
    existing = await session.execute(dup_q)
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="Group already exists")
    # hierarchy check: cannot create group with level higher than your own unless superadmin?
    if creator is not None:
        lvl = await get_user_level(session, creator)
        is_super = _is_superadmin(creator)
        if group_level > lvl and not is_super:
            raise HTTPException(status_code=403, detail="Cannot create group with higher level than your own")
    g = Group(domain_id=domain_id, name=name, description=description, group_level=group_level, group_protected=group_protected)
    session.add(g)
    await session.commit()
    try:
        if domain_id:
            await invalidate_fs_cache_for_session(session, domain_id)
        else:
            # global group: invalidate for creator domain
            if creator:
                await invalidate_fs_cache_for_session(session, getattr(creator, "_domain_id", creator.domain_id))
    except Exception:
        pass
    await session.refresh(g)
    return g

async def get_group(session: AsyncSession, domain_id: UUID, group_uuid: UUID, show_all: bool=False):
    # fetch regardless then check visibility
    r = await session.execute(select(Group).where(Group.group_uuid == group_uuid))
    g = r.scalar_one_or_none()
    if not g:
        return None
    if show_all:
        return g
    # visible if domain matches or global
    if g.domain_id is None or g.domain_id == domain_id:
        return g
    return None

async def get_group_with_level_check(session: AsyncSession, domain_id: UUID, group_uuid: UUID):
    return await get_group(session, domain_id, group_uuid)

async def get_group_users(session: AsyncSession, domain_id: UUID, group_uuid: UUID, show_all: bool=False):
    # fetch group to determine domain scoping like legacy group_members.php
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        return []
    # if group has domain, filter by that domain; else use caller domain
    target_domain = g.domain_id if g.domain_id is not None else domain_id
    # legacy also checks user_all permission for cross-domain; we respect show_all
    if show_all:
        q = select(User).join(UserGroup, UserGroup.user_uuid == User.user_uuid).where(UserGroup.group_uuid == group_uuid)
    else:
        q = select(User).join(UserGroup, UserGroup.user_uuid == User.user_uuid).where(UserGroup.group_uuid == group_uuid, UserGroup.domain_id == target_domain)
        # also ensure user domain matches? legacy: ug.domain_uuid = d.domain_uuid and u.domain_uuid = :domain
        q = q.where(User.domain_id == target_domain)
    r = await session.execute(q.order_by(User.username))
    return r.scalars().all()

async def add_user_to_group(session: AsyncSession, domain_id: UUID, group_uuid: UUID, user_uuid: UUID, actor=None):
    grp = await get_group(session, domain_id, group_uuid, show_all=True)
    if not grp:
        raise HTTPException(status_code=404, detail="Group not found")
    # visibility check: if group domain not null and not matching caller, deny unless superadmin
    if grp.domain_id is not None and grp.domain_id != domain_id:
        # allow if actor has group_all? we check level
        pass
    # fetch user (allow same domain or global group can add any domain user? legacy group_members.php adds users from domain_uuid of group)
    target_domain = grp.domain_id if grp.domain_id is not None else domain_id
    r = await session.execute(select(User).where(User.user_uuid == user_uuid))
    user = r.scalar_one_or_none()
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    # ensure user domain matches target_domain unless show_all (simplify enforce)
    if str(user.domain_id) != str(target_domain):
        # allow cross-domain if superadmin? check actor is superadmin via "*"
        if actor:
            if not _is_superadmin(actor):
                raise HTTPException(status_code=403, detail="Cannot add user from different domain")
        else:
            raise HTTPException(status_code=404, detail="User not found in domain")
    existing = await session.execute(select(UserGroup).where(UserGroup.group_uuid == group_uuid, UserGroup.user_uuid == user_uuid))
    if existing.scalar_one_or_none():
        raise HTTPException(status_code=409, detail="User already in group")
    # delegation check: group_level < actor_level OR actor member of group
    if actor is not None:
        actor_level = await get_user_level(session, actor)
        my_groups = await get_user_group_uuids(session, actor)
        if grp.group_level is not None and grp.group_level >= actor_level and grp.group_uuid not in my_groups:
            if not _is_superadmin(actor):
                raise HTTPException(status_code=403, detail="Cannot assign group at your level or higher unless you are member")
        # superadmin group protection: only superadmin can manage superadmin members (legacy group_members.php:37)
        if grp.name == "superadmin":
            # check actor is superadmin
            is_super = False
            q = select(Group.name).join(UserGroup, UserGroup.group_uuid == Group.group_uuid).where(UserGroup.user_uuid == actor.user_uuid)
            rr = await session.execute(q)
            for (n,) in rr.all():
                if n == "superadmin":
                    is_super = True
                    break
            if not is_super and not _is_superadmin(actor):
                raise HTTPException(status_code=403, detail="Only superadmin can manage superadmin group")
    ug = UserGroup(user_uuid=user_uuid, group_uuid=group_uuid, domain_id=target_domain, group_name=grp.name)
    session.add(ug)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, target_domain)
    except Exception:
        pass
    return ug

async def remove_user_from_group(session: AsyncSession, domain_id: UUID, group_uuid: UUID, user_uuid: UUID, actor=None):
    grp = await get_group(session, domain_id, group_uuid, show_all=True)
    target_domain = grp.domain_id if grp and grp.domain_id is not None else domain_id
    r = await session.execute(select(UserGroup).where(UserGroup.group_uuid == group_uuid, UserGroup.user_uuid == user_uuid))
    # also filter by domain if not superadmin
    ug = r.scalar_one_or_none()
    if not ug:
        raise HTTPException(status_code=404, detail="Membership not found")
    # check domain mismatch: if ug domain != caller domain and not superadmin, deny
    if str(ug.domain_id) != str(domain_id):
        if actor:
            if not _is_superadmin(actor):
                raise HTTPException(status_code=403, detail="Cannot remove cross-domain membership")
    # superadmin protection
    if grp and grp.name == "superadmin" and actor:
        is_super = _is_superadmin(actor)
        if not is_super:
            # check membership
            my_groups = await get_user_group_uuids(session, actor)
            if grp.group_uuid not in my_groups:
                raise HTTPException(status_code=403, detail="Only superadmin can manage superadmin group")
    await session.delete(ug)
    await session.commit()
    try:
        await invalidate_fs_cache_for_session(session, target_domain)
    except Exception:
        pass

async def get_user_groups(session: AsyncSession, domain_id: UUID, user_uuid: UUID, show_all: bool=False):
    if show_all:
        q = select(Group).join(UserGroup, UserGroup.group_uuid == Group.group_uuid).where(UserGroup.user_uuid == user_uuid)
    else:
        q = select(Group).join(UserGroup, UserGroup.group_uuid == Group.group_uuid).where(UserGroup.user_uuid == user_uuid, UserGroup.domain_id == domain_id)
    r = await session.execute(q.order_by(Group.name))
    return r.scalars().all()

async def list_permissions(session: AsyncSession):
    r = await session.execute(select(Permission).order_by(Permission.name))
    return r.scalars().all()

async def get_group_permissions(session: AsyncSession, group_uuid: UUID):
    q = select(Permission).join(GroupPermission, GroupPermission.permission_uuid == Permission.permission_uuid).where(GroupPermission.group_uuid == group_uuid)
    r = await session.execute(q)
    return r.scalars().all()

async def get_group_permissions_detailed(session: AsyncSession, group_uuid: UUID):
    # return GroupPermission rows with flags
    r = await session.execute(select(GroupPermission).where(GroupPermission.group_uuid == group_uuid))
    return r.scalars().all()

async def set_group_permissions(session: AsyncSession, group_uuid: UUID, permission_uuids: List[UUID], protected_map: Optional[dict]=None):
    existing = await session.execute(select(GroupPermission).where(GroupPermission.group_uuid == group_uuid))
    for gp in existing.scalars().all():
        await session.delete(gp)
    # need group info for denorm
    rg = await session.execute(select(Group).where(Group.group_uuid == group_uuid))
    grp = rg.scalar_one_or_none()
    for puuid in permission_uuids:
        # lookup permission name
        pr = await session.execute(select(Permission).where(Permission.permission_uuid == puuid))
        perm = pr.scalar_one_or_none()
        pname = perm.name if perm else None
        prot = False
        if protected_map and str(puuid) in protected_map:
            prot = protected_map[str(puuid)]
        elif protected_map and pname in protected_map:
            prot = protected_map[pname]
        gp = GroupPermission(group_uuid=group_uuid, permission_uuid=puuid, domain_id=grp.domain_id if grp else None, group_name=grp.name if grp else None, permission_name=pname, permission_protected=prot, permission_assigned=True)
        session.add(gp)
    await session.commit()
    try:
        if grp and grp.domain_id:
            await invalidate_fs_cache_for_session(session, grp.domain_id)
    except Exception:
        pass

async def copy_group(session: AsyncSession, domain_id: UUID, group_uuid: UUID, actor=None, is_superadmin: bool = False, caller_level: Optional[int] = None):
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    # hierarchy check: cannot copy group with level higher than your own unless superadmin (like create)
    if actor is not None and not is_superadmin:
        lvl = caller_level
        if lvl is None:
            lvl = await get_user_level(session, actor)
        if g.group_level is not None and g.group_level > lvl:
            raise HTTPException(status_code=403, detail="Cannot copy group with higher level than your own")
    new_uuid = uuid7()
    copy_name = g.name + " (copy)"
    # ensure unique name
    base = copy_name
    idx = 1
    while True:
        q = select(Group).where(Group.name == copy_name, Group.domain_id == g.domain_id)
        rr = await session.execute(q)
        if not rr.scalar_one_or_none():
            break
        idx += 1
        copy_name = f"{g.name} (copy {idx})"
    # preserve protected=False for new group but clone permission_protected as-is (legacy)
    new_g = Group(group_uuid=new_uuid, domain_id=g.domain_id, name=copy_name, description=g.description, group_level=g.group_level, group_protected=False)
    session.add(new_g)
    await session.flush()
    # copy permissions
    r = await session.execute(select(GroupPermission).where(GroupPermission.group_uuid == g.group_uuid))
    for gp in r.scalars().all():
        new_gp = GroupPermission(group_uuid=new_uuid, permission_uuid=gp.permission_uuid, domain_id=g.domain_id, group_name=copy_name, permission_name=gp.permission_name, permission_protected=gp.permission_protected, permission_assigned=gp.permission_assigned)
        session.add(new_gp)
    await session.commit()
    await session.refresh(new_g)
    try:
        if g.domain_id:
            await invalidate_fs_cache_for_session(session, g.domain_id)
    except Exception:
        pass
    return new_g

async def toggle_group(session: AsyncSession, domain_id: UUID, group_uuid: UUID, actor=None, is_superadmin: bool = False):
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    # protected guard: only superadmin can toggle protected groups (service-level guard)
    if not is_superadmin and actor is not None:
        is_superadmin = _is_superadmin(actor)
    if g.group_protected and not is_superadmin:
        raise HTTPException(status_code=403, detail="protected group")
    # toggle flips correctly
    g.group_protected = not g.group_protected
    await session.commit()
    await session.refresh(g)
    try:
        if g.domain_id:
            await invalidate_fs_cache_for_session(session, g.domain_id)
    except Exception:
        pass
    return g

async def delete_group(session: AsyncSession, domain_id: UUID, group_uuid: UUID, actor=None, is_superadmin: bool = False):
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    # derive is_superadmin if not supplied but actor given — only via "*" not level
    if not is_superadmin and actor is not None:
        is_superadmin = _is_superadmin(actor)
    if g.group_protected and not is_superadmin:
        raise HTTPException(status_code=403, detail="protected group")
    # permission_protected filter: non-superadmin only removes non-protected assignments
    if is_superadmin:
        await session.execute(delete(GroupPermission).where(GroupPermission.group_uuid == group_uuid))
    else:
        await session.execute(delete(GroupPermission).where(GroupPermission.group_uuid == group_uuid, GroupPermission.permission_protected == False))
    await session.delete(g)
    await session.commit()
    try:
        if g.domain_id:
            await invalidate_fs_cache_for_session(session, g.domain_id)
    except Exception:
        pass
