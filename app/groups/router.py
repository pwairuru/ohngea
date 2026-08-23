import re
from typing import Optional, List
from uuid import UUID
from fastapi import APIRouter, Depends, HTTPException, Query, Response, Request
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession

_GROUP_NAME_RE = re.compile(r"^[a-z0-9_-]{2,64}$")

from app.auth.deps import get_current_user, require_permission
from app.auth.models import User, Group, Permission, GroupPermission, UserGroup
from app.db import get_session, set_tenant
from app.freeswitch.cache import invalidate_fs_cache_for_session
from app.groups.schemas import GroupCreate, GroupRead, GroupUpdate, GroupPermissionUpdate, GroupBulkAction
from app.groups.service import (
    list_groups, create_group, get_group, get_group_users, add_user_to_group, remove_user_from_group,
    get_user_groups, list_permissions, get_group_permissions, set_group_permissions,
    copy_group, toggle_group, get_user_level
)
from app.domains.models import Domain

router = APIRouter(prefix="/api/v1", tags=["groups"])

def _links(g: Group):
    base = f"/api/v1/groups/{g.group_uuid}"
    return {"self": {"href": base}, "collection": {"href": "/api/v1/groups"}, "users": {"href": f"{base}/users"}, "permissions": {"href": f"{base}/permissions"}, "copy": {"href": f"{base}/copy"}, "toggle": {"href": f"{base}/toggle"}}

def _ser(g: Group, member_count: Optional[int]=None, perm_count: Optional[int]=None, domain_name: Optional[str]=None, permissions=None):
    d = GroupRead.model_validate(g, from_attributes=True).model_dump(mode="json")
    d["_links"] = _links(g)
    d["group_uuid"] = str(g.group_uuid)
    d["domain_id"] = str(g.domain_id) if g.domain_id else None
    d["domain_name"] = domain_name
    if member_count is not None:
        d["member_count"] = member_count
        d["group_members"] = member_count
    if perm_count is not None:
        d["group_permissions"] = perm_count
        d["permission_count"] = perm_count
    if permissions is not None:
        d["permissions"] = permissions
    return d

@router.get("/groups", response_model=dict)
async def list_groups_ep(
    request: Request,
    response: Response,
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    q: Optional[str] = Query(None, alias="search"),
    search: Optional[str] = Query(None),
    show: Optional[str] = Query(None),
    order_by: str = Query("group_name", pattern="^(group_name|name|group_level|group_protected|description)$"),
    order: str = Query("asc", pattern="^(asc|desc)$"),
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    query_q = q or search
    show_all = (show == "all")
    # permission check: need group_view or superadmin wildcard
    perms = getattr(user, "_perms", None)
    # if perms is None fetch via service
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    has_view = "*" in perms or "group_view" in perms or "domain_add" in perms
    if not has_view:
        raise HTTPException(status_code=403, detail="Missing permission: group_view")
    # if show_all requested but no permission, fallback to false
    if show_all and not ("group_all" in perms or "*" in perms or "domain_add" in perms):
        show_all = False
    # map order_by
    ob_map = {"group_name": "group_name", "name": "group_name"}
    items, total = await list_groups(session, domain_id, user, page, per_page, query_q, show_all, ob_map.get(order_by, "group_name"), order)
    response.headers["X-Total-Count"] = str(total)
    data = []
    # batch fetch counts and domain names
    for g in items:
        c = (await session.execute(select(func.count()).select_from(UserGroup).where(UserGroup.group_uuid == g.group_uuid))).scalar_one()
        pc = (await session.execute(select(func.count()).select_from(GroupPermission).where(GroupPermission.group_uuid == g.group_uuid))).scalar_one()
        domain_name = None
        if g.domain_id:
            # try cache? fetch domain
            r = await session.execute(select(Domain.domain_name).where(Domain.domain_id == g.domain_id))
            row = r.scalar_one_or_none()
            domain_name = row
        else:
            domain_name = "global"
        data.append(_ser(g, member_count=c, perm_count=pc, domain_name=domain_name))
    if page * per_page < total:
        response.headers["Link"] = f'</api/v1/groups?page={page+1}&per_page={per_page}>; rel="next"'
    return {"data": data, "total": total, "page": page, "per_page": per_page}

@router.post("/groups", response_model=dict, status_code=201)
async def create_group_ep(
    payload: GroupCreate,
    response: Response,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    # permission check group_add
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_add" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_add")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    if not payload.name or not payload.name.strip():
        raise HTTPException(status_code=422, detail="name required")
    name = payload.name.strip()
    if len(name) > 64:
        raise HTTPException(status_code=422, detail="name too long")
    # group_name regex ^[a-z0-9_-]{2,64}$ lower like legacy
    if not _GROUP_NAME_RE.match(name.lower()):
        raise HTTPException(status_code=422, detail="group_name must match ^[a-z0-9_-]{2,64}$")
    # normalize to lower? legacy preserves case but tests use lower; keep as provided lowercased?
    # keep original but ensure duplicate check uses exact name
    name = name.lower()
    # determine target domain: None for global
    target_domain = payload.domain_id
    # if payload.domain_id is explicitly None and user wants global, check permission group_global
    if target_domain is None and payload.model_fields_set and "domain_id" in payload.model_fields_set:
        # they explicitly sent null => want global
        if not ("*" in perms or "group_global" in perms or "domain_add" in perms):
            raise HTTPException(status_code=403, detail="Missing permission: group_global")
        # global domain_id None
    elif target_domain is not None:
        if str(target_domain) != str(domain_id):
            # check group_domain or group_all
            if not ("*" in perms or "group_domain" in perms or "group_all" in perms):
                raise HTTPException(status_code=403, detail="Domain mismatch")
        # target_domain stays as payload
    else:
        # not set => default to user's domain
        target_domain = domain_id
    level = payload.group_level if payload.group_level is not None else 10
    protected = payload.group_protected if payload.group_protected is not None else False
    g = await create_group(session, target_domain, name, payload.description, level, protected, creator=user)
    response.headers["Location"] = f"/api/v1/groups/{g.group_uuid}"
    return _ser(g, member_count=0, perm_count=0, domain_name="global" if g.domain_id is None else None)

@router.post("/groups/bulk", response_model=dict)
async def bulk_groups_ep(
    payload: GroupBulkAction,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    is_superadmin = "*" in perms
    caller_level = await get_user_level(session, user)
    action = payload.action
    ids = payload.ids
    if action not in ("copy","toggle","delete"):
        raise HTTPException(status_code=422, detail="Invalid action")
    results = []
    for gid in ids:
        try:
            if action == "copy":
                if not ("*" in perms or "group_add" in perms or "domain_add" in perms):
                    raise HTTPException(status_code=403, detail="Missing permission: group_add")
                # hierarchy check
                g = await get_group(session, domain_id, gid, show_all=True)
                if not g:
                    raise HTTPException(status_code=404, detail="Group not found")
                if g.group_level is not None and g.group_level > caller_level and not is_superadmin:
                    raise HTTPException(status_code=403, detail="Cannot copy group with higher level than your own")
                ng = await copy_group(session, domain_id, gid, actor=user, is_superadmin=is_superadmin, caller_level=caller_level)
                results.append({"group_uuid": str(ng.group_uuid), "action": "copied", "name": ng.name})
            elif action == "toggle":
                if not ("*" in perms or "group_edit" in perms or "domain_add" in perms):
                    raise HTTPException(status_code=403, detail="Missing permission: group_edit")
                g = await get_group(session, domain_id, gid, show_all=True)
                if not g:
                    raise HTTPException(status_code=404, detail="Group not found")
                if g.group_protected and not is_superadmin:
                    raise HTTPException(status_code=403, detail="protected group")
                ng = await toggle_group(session, domain_id, gid, actor=user, is_superadmin=is_superadmin)
                results.append({"group_uuid": str(ng.group_uuid), "action": "toggled", "protected": ng.group_protected})
            elif action == "delete":
                if not ("*" in perms or "group_delete" in perms or "domain_add" in perms):
                    raise HTTPException(status_code=403, detail="Missing permission: group_delete")
                from app.groups.service import delete_group
                await delete_group(session, domain_id, gid, actor=user, is_superadmin=is_superadmin)
                results.append({"group_uuid": str(gid), "action": "deleted"})
        except HTTPException as e:
            results.append({"group_uuid": str(gid), "error": e.detail})
    return {"data": results, "total": len(results)}

@router.get("/groups/{group_uuid}", response_model=dict)
async def get_group_ep(group_uuid: UUID, show: Optional[str]=Query(None), session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    show_all = (show == "all")
    g = await get_group(session, domain_id, group_uuid, show_all=show_all)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    # level visibility
    lvl = await get_user_level(session, user)
    if g.group_level is not None and g.group_level > lvl and lvl < 80:
        raise HTTPException(status_code=403, detail="Group level too high")
    c = (await session.execute(select(func.count()).select_from(UserGroup).where(UserGroup.group_uuid == group_uuid))).scalar_one()
    pc = (await session.execute(select(func.count()).select_from(GroupPermission).where(GroupPermission.group_uuid == group_uuid))).scalar_one()
    perms = await get_group_permissions(session, group_uuid)
    perm_names = [p.name for p in perms]
    domain_name = None
    if g.domain_id:
        r = await session.execute(select(Domain.domain_name).where(Domain.domain_id == g.domain_id))
        domain_name = r.scalar_one_or_none()
    else:
        domain_name = "global"
    return _ser(g, member_count=c, perm_count=pc, domain_name=domain_name, permissions=perm_names)

@router.patch("/groups/{group_uuid}", response_model=dict)
async def patch_group_ep(group_uuid: UUID, payload: GroupUpdate, session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_edit" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    # check level hierarchy for edit
    lvl = await get_user_level(session, user)
    is_superadmin = "*" in perms
    if g.group_level is not None and g.group_level > lvl and not is_superadmin:
        raise HTTPException(status_code=403, detail="Group level too high to edit")
    # protected guard: only superadmin can patch protected groups' level/protected fields
    if g.group_protected and not is_superadmin:
        if payload.group_protected is not None or payload.group_level is not None:
            raise HTTPException(status_code=403, detail="protected group")
    # only superadmin can change protected flag
    if payload.group_protected is not None and not is_superadmin:
        raise HTTPException(status_code=403, detail="protected group")
    if payload.name is not None:
        name = payload.name.strip()
        if not name:
            raise HTTPException(status_code=422, detail="name cannot be empty")
        if not _GROUP_NAME_RE.match(name.lower()):
            raise HTTPException(status_code=422, detail="group_name must match ^[a-z0-9_-]{2,64}$")
        name = name.lower()
        if name != g.name:
            # duplicate check
            if g.domain_id is None:
                dq = select(Group).where(Group.name==name, Group.domain_id==None, Group.group_uuid!=g.group_uuid)
            else:
                dq = select(Group).where(Group.name==name, Group.domain_id==g.domain_id, Group.group_uuid!=g.group_uuid)
            r = await session.execute(dq)
            if r.scalar_one_or_none():
                raise HTTPException(status_code=409, detail="Group name exists")
            # update denorm group_name in permissions and user_groups
            g.name = name
            # update related tables
            await session.execute(select(GroupPermission).where(GroupPermission.group_uuid==g.group_uuid)) # dummy?
            # will update after commit via following updates
            from sqlalchemy import update
            await session.execute(update(GroupPermission).where(GroupPermission.group_uuid==g.group_uuid).values(group_name=name))
            await session.execute(update(UserGroup).where(UserGroup.group_uuid==g.group_uuid).values(group_name=name))
    if payload.description is not None:
        g.description = payload.description
    if payload.group_level is not None:
        if payload.group_level not in [10,20,30,40,50,60,70,80,90]:
            raise HTTPException(status_code=422, detail="Invalid group_level")
        if payload.group_level > lvl and not is_superadmin:
            raise HTTPException(status_code=403, detail="Cannot set level higher than your own")
        g.group_level = payload.group_level
    if payload.group_protected is not None:
        g.group_protected = payload.group_protected
    if payload.domain_id is not None or ("domain_id" in payload.model_fields_set and payload.domain_id is None):
        # domain change
        if not ("*" in perms or "group_domain" in perms or "group_all" in perms):
            raise HTTPException(status_code=403, detail="Missing permission: group_domain")
        g.domain_id = payload.domain_id
        # update related denorm domain_id in permissions
        from sqlalchemy import update
        await session.execute(update(GroupPermission).where(GroupPermission.group_uuid==g.group_uuid).values(domain_id=payload.domain_id))
        # user_groups domain update? legacy also updates user_groups domain when user domain changes, but for group we update?
    # handle explicit global null via empty string? already
    await session.commit()
    await session.refresh(g)
    c = (await session.execute(select(func.count()).select_from(UserGroup).where(UserGroup.group_uuid==group_uuid))).scalar_one()
    pc = (await session.execute(select(func.count()).select_from(GroupPermission).where(GroupPermission.group_uuid==group_uuid))).scalar_one()
    perms_list = await get_group_permissions(session, group_uuid)
    return _ser(g, member_count=c, perm_count=pc, permissions=[p.name for p in perms_list])

@router.post("/groups/{group_uuid}/copy", response_model=dict, status_code=201)
async def copy_group_ep(group_uuid: UUID, session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_add" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_add")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    is_superadmin = "*" in perms
    caller_level = await get_user_level(session, user)
    # hierarchy check like create: if new_level > caller_level and not is_superadmin =>403
    # fetch group first to check level
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    if g.group_level is not None and g.group_level > caller_level and not is_superadmin:
        raise HTTPException(status_code=403, detail="Cannot copy group with higher level than your own")
    ng = await copy_group(session, domain_id, group_uuid, actor=user, is_superadmin=is_superadmin, caller_level=caller_level)
    return _ser(ng, member_count=0)

@router.post("/groups/{group_uuid}/toggle", response_model=dict)
async def toggle_group_ep(group_uuid: UUID, session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_edit" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    # protected guard before flip: if group_protected and not superadmin =>403
    # Note: toggle flips protected flag, so only superadmin should toggle protected groups
    # For now, if group is protected and caller not superadmin, block
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    is_superadmin = "*" in perms
    if g.group_protected and not is_superadmin:
        raise HTTPException(status_code=403, detail="protected group")
    ng = await toggle_group(session, domain_id, group_uuid, actor=user, is_superadmin=is_superadmin)
    return _ser(ng)

@router.delete("/groups/{group_uuid}", status_code=204)
async def delete_group_ep(group_uuid: UUID, session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_delete" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_delete")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    from app.groups.service import delete_group
    is_superadmin = "*" in perms
    await delete_group(session, domain_id, group_uuid, actor=user, is_superadmin=is_superadmin)
    return Response(status_code=204)

# membership

@router.get("/groups/{group_uuid}/users", response_model=dict)
async def list_group_users_ep(group_uuid: UUID, show: Optional[str]=Query(None), session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    show_all = (show=="all")
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_member_view" in perms or "group_view" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_member_view")
    g = await get_group(session, domain_id, group_uuid, show_all=show_all)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    # superadmin guard
    if g.name=="superadmin":
        lvl = await get_user_level(session, user)
        is_super = lvl>=80 or any(1 for _ in [] )
        # check if user is superadmin member
        my_uuids = await get_user_groups(session, domain_id, user.user_uuid)
        is_super = any(gr.name=="superadmin" for gr in my_uuids) or "*" in perms
        if not is_super:
            raise HTTPException(status_code=403, detail="Only superadmin can view superadmin members")
    users = await get_group_users(session, domain_id, group_uuid, show_all=show_all)
    data = [{"user_uuid": str(u.user_uuid), "username": u.username, "email": u.email, "enabled": u.enabled, "domain_id": str(u.domain_id)} for u in users]
    return {"data": data, "total": len(data)}

@router.post("/groups/{group_uuid}/users", response_model=dict, status_code=201)
async def add_group_user_ep(group_uuid: UUID, payload: dict, session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_member_add" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_member_add")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    user_uuid = payload.get("user_uuid") or payload.get("user_id") or payload.get("id")
    if not user_uuid:
        raise HTTPException(status_code=422, detail="user_uuid required")
    try:
        u_uuid = UUID(str(user_uuid))
    except Exception:
        raise HTTPException(status_code=422, detail="Invalid user_uuid")
    ug = await add_user_to_group(session, domain_id, group_uuid, u_uuid, actor=user)
    return {"id": str(ug.id), "group_uuid": str(group_uuid), "user_uuid": str(u_uuid)}

@router.delete("/groups/{group_uuid}/users/{user_uuid}", status_code=204)
async def remove_group_user_ep(group_uuid: UUID, user_uuid: UUID, session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_member_delete" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_member_delete")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    await remove_user_from_group(session, domain_id, group_uuid, user_uuid, actor=user)
    return Response(status_code=204)

# user -> groups

@router.get("/users/{user_uuid}/groups", response_model=dict)
async def list_user_groups_ep(user_uuid: UUID, show: Optional[str]=Query(None), session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    r = await session.execute(select(User).where(User.user_uuid==user_uuid))
    u = r.scalar_one_or_none()
    if not u:
        raise HTTPException(status_code=404, detail="User not found")
    # domain check if not show_all
    if not (show=="all") and str(u.domain_id)!=str(domain_id):
        # need user_all to view cross-domain
        perms = getattr(current, "_perms", None)
        if perms is None:
            from app.auth.service import get_user_permissions
            perms = await get_user_permissions(session, current)
        if not ("*" in perms or "user_all" in perms):
            raise HTTPException(status_code=404, detail="User not found")
    show_all = (show=="all")
    groups = await get_user_groups(session, domain_id, user_uuid, show_all=show_all)
    data = [_ser(g) for g in groups]
    return {"data": data, "total": len(data)}

@router.post("/users/{user_uuid}/groups", response_model=dict, status_code=201)
async def add_user_group_ep(user_uuid: UUID, payload: dict, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    perms = getattr(current, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, current)
    if not ("*" in perms or "user_group_add" in perms or "group_member_add" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_group_add")
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    group_uuid = payload.get("group_uuid") or payload.get("group_id") or payload.get("id")
    if not group_uuid:
        raise HTTPException(status_code=422, detail="group_uuid required")
    try:
        g_uuid = UUID(str(group_uuid))
    except Exception:
        raise HTTPException(status_code=422, detail="Invalid group_uuid")
    # verify user exists
    r = await session.execute(select(User).where(User.user_uuid==user_uuid))
    if not r.scalar_one_or_none():
        raise HTTPException(status_code=404, detail="User not found")
    ug = await add_user_to_group(session, domain_id, g_uuid, user_uuid, actor=current)
    return {"id": str(ug.id), "group_uuid": str(g_uuid), "user_uuid": str(user_uuid)}

@router.delete("/users/{user_uuid}/groups/{group_uuid}", status_code=204)
async def remove_user_group_ep(user_uuid: UUID, group_uuid: UUID, session: AsyncSession=Depends(get_session), current: User=Depends(get_current_user)):
    perms = getattr(current, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, current)
    if not ("*" in perms or "user_group_delete" in perms or "group_member_delete" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: user_group_delete")
    domain_id = getattr(current, "_domain_id", current.domain_id)
    await set_tenant(session, domain_id)
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    await remove_user_from_group(session, domain_id, group_uuid, user_uuid, actor=current)
    return Response(status_code=204)

# permissions

@router.get("/permissions", response_model=dict)
async def list_permissions_ep(session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = await list_permissions(session)
    data = [{"permission_uuid": str(p.permission_uuid), "name": p.name, "description": p.description} for p in perms]
    return {"data": data, "total": len(data)}

@router.get("/groups/{group_uuid}/permissions", response_model=dict)
async def get_group_permissions_ep(group_uuid: UUID, category: Optional[str]=Query(None), search: Optional[str]=Query(None), view: Optional[str]=Query(None), session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    perms = await get_group_permissions(session, group_uuid)
    # detailed flags
    from sqlalchemy import select as sel
    r = await session.execute(sel(GroupPermission).where(GroupPermission.group_uuid==group_uuid))
    gp_map = {str(row.permission_uuid): row for row in r.scalars().all()}
    # also need to join with v_permissions? we have permissions table only; legacy has v_permissions with application_name; we lack application_name, so category filter limited
    # get all permissions with flags
    all_perms = await list_permissions(session)
    # filter
    result = []
    for p in all_perms:
        gp = gp_map.get(str(p.permission_uuid))
        assigned = gp.permission_assigned if gp else False
        protected = gp.permission_protected if gp else False
        # view filter
        if view=="assigned" and not assigned:
            continue
        if view=="unassigned" and assigned:
            continue
        if view=="protected" and not protected:
            continue
        if search and search.lower() not in p.name.lower():
            continue
        # category: we don't have application_name; skip
        result.append({"permission_uuid": str(p.permission_uuid), "name": p.name, "description": p.description, "assigned": assigned, "protected": protected, "group_permission_uuid": str(gp.id) if gp else None})
    # also sort by name
    return {"data": result, "total": len(result), "group_name": g.name, "group_uuid": str(g.group_uuid)}

@router.put("/groups/{group_uuid}/permissions", response_model=dict)
async def put_group_permissions_ep(group_uuid: UUID, payload: GroupPermissionUpdate, session: AsyncSession=Depends(get_session), user: User=Depends(get_current_user)):
    perms = getattr(user, "_perms", None)
    if perms is None:
        from app.auth.service import get_user_permissions
        perms = await get_user_permissions(session, user)
    if not ("*" in perms or "group_permission_edit" in perms or "group_permissions" in perms or "domain_add" in perms):
        raise HTTPException(status_code=403, detail="Missing permission: group_permission_edit")
    domain_id = getattr(user, "_domain_id", user.domain_id)
    await set_tenant(session, domain_id)
    g = await get_group(session, domain_id, group_uuid, show_all=True)
    if not g:
        raise HTTPException(status_code=404, detail="Group not found")
    perm_uuids: List[UUID]=[]
    protected_map = {}
    if payload.permission_uuids:
        perm_uuids = payload.permission_uuids
    elif payload.permission_names:
        names = payload.permission_names
        r = await session.execute(select(Permission).where(Permission.name.in_(names)))
        found = {p.name: p.permission_uuid for p in r.scalars().all()}
        missing = [n for n in names if n not in found]
        if missing:
            raise HTTPException(status_code=404, detail=f"Permissions not found: {missing}")
        perm_uuids = [found[n] for n in names]
    elif payload.permissions:
        names = payload.permissions
        r = await session.execute(select(Permission).where(Permission.name.in_(names)))
        found = {p.name: p.permission_uuid for p in r.scalars().all()}
        missing = [n for n in names if n not in found]
        if missing:
            raise HTTPException(status_code=404, detail=f"Permissions not found: {missing}")
        perm_uuids = [found[n] for n in names]
    else:
        # allow empty to clear
        perm_uuids = []
    # also handle protected flag if payload has extra field group_permissions with protected? payload extra?
    # try to read raw body protected map from payload model_extra
    if hasattr(payload, "model_extra") and payload.model_extra:
        # may contain list of dicts with permission_protected
        pass
    if perm_uuids:
        r = await session.execute(select(Permission).where(Permission.permission_uuid.in_(perm_uuids)))
        found_uuids = {p.permission_uuid for p in r.scalars().all()}
        missing = [str(u) for u in perm_uuids if u not in found_uuids]
        if missing:
            raise HTTPException(status_code=404, detail=f"Permission UUIDs not found: {missing}")
    await set_group_permissions(session, group_uuid, perm_uuids, protected_map)
    perms_list = await get_group_permissions(session, group_uuid)
    data = [{"permission_uuid": str(p.permission_uuid), "name": p.name} for p in perms_list]
    return {"data": data, "total": len(data)}

