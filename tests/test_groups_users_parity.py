import pytest
from uuid import uuid4

import sqlalchemy as sa
from sqlalchemy import select, func

pytestmark = pytest.mark.asyncio

# Helpers for level-based users

async def _create_group_via_api(client, headers, name, level=30, protected=False, domain_id=None, description=None):
    payload = {"name": name, "group_level": level, "group_protected": protected}
    if description is not None:
        payload["description"] = description
    if domain_id is not None:
        payload["domain_id"] = str(domain_id)
    # explicit None for global: send null
    elif domain_id is None and name.endswith("_global"):
        payload["domain_id"] = None
    resp = await client.post("/api/v1/groups", json=payload, headers=headers)
    return resp

async def _make_level_user(test_db, domain, username, level, perms=None):
    """Create user with a group of specific level, return (user, token)."""
    from app.auth.models import Group, User, UserGroup, Permission, GroupPermission
    from app.auth.service import create_token, hash_password
    if perms is None:
        perms = ["group_view", "group_add", "group_edit", "group_delete", "group_member_view", "group_member_add", "group_member_delete", "group_permission_view", "group_permission_edit", "group_all"]
    async with test_db() as s:
        # ensure permissions exist
        for pname in perms:
            r = await s.execute(select(Permission).where(Permission.name == pname))
            if not r.scalar_one_or_none():
                s.add(Permission(name=pname))
        await s.commit()
        # create group with level
        g = Group(domain_id=domain.domain_id, name=f"lvl{level}-{username}-{uuid4().hex[:4]}", group_level=level, group_protected=False)
        s.add(g)
        await s.commit()
        await s.refresh(g)
        # link perms to group
        for pname in perms:
            r = await s.execute(select(Permission).where(Permission.name == pname))
            perm = r.scalar_one()
            s.add(GroupPermission(group_uuid=g.group_uuid, permission_uuid=perm.permission_uuid, group_name=g.name, permission_name=pname, permission_assigned=True))
        await s.commit()
        u = User(username=username, domain_id=domain.domain_id, password_hash=hash_password("password123"), enabled=True)
        s.add(u)
        await s.commit()
        await s.refresh(u)
        ug = UserGroup(user_uuid=u.user_uuid, group_uuid=g.group_uuid, domain_id=domain.domain_id, group_name=g.name)
        s.add(ug)
        await s.commit()
        token = create_token(u, permissions=perms)
        u._perms = perms
        u._domain_id = domain.domain_id
        return u, token, g


async def test_create_group_with_level_and_protected(client, auth_headers):
    # admin has * -> can create any level
    resp = await _create_group_via_api(client, auth_headers, "sales", level=30, protected=False, description="Sales team")
    assert resp.status_code == 201, resp.text
    data = resp.json()
    assert data["name"] == "sales"
    assert data["group_level"] == 30
    assert data["group_protected"] is False
    assert data["description"] == "Sales team"
    # duplicate 409
    resp2 = await _create_group_via_api(client, auth_headers, "sales", level=30)
    assert resp2.status_code == 409

async def test_create_group_global_and_domain_isolation(client, auth_headers, domain, other_domain, test_db, other_auth_headers):
    # create global group (domain_id null) - name ends with _global triggers null payload
    resp = await _create_group_via_api(client, auth_headers, "public_global", level=10, domain_id=None)
    # our helper sends None only if name ends with _global, so we manually send
    if resp.status_code != 201:
        # try explicit global via direct API
        resp = await client.post("/api/v1/groups", json={"name": "public_global2", "group_level": 10, "domain_id": None}, headers=auth_headers)
    assert resp.status_code == 201, resp.text
    assert resp.json()["domain_id"] is None
    # create domain-specific group
    resp2 = await client.post("/api/v1/groups", json={"name": "domain_specific", "group_level": 20, "domain_id": str(domain.domain_id)}, headers=auth_headers)
    assert resp2.status_code == 201
    # list without show_all should see both global and domain for same domain
    r = await client.get("/api/v1/groups", headers=auth_headers)
    assert r.status_code == 200
    names = [x["name"] for x in r.json()["data"]]
    assert "public_global" in names or "public_global2" in names
    # other domain should not see domain_specific but should see global if show_all? Actually global visible to all
    r2 = await client.get("/api/v1/groups", headers=other_auth_headers)
    assert r2.status_code == 200
    names2 = [x["name"] for x in r2.json()["data"]]
    # global should be visible to other domain
    assert any(n.startswith("public_global") for n in names2)
    # domain_specific should not be visible to other domain without show_all
    assert "domain_specific" not in names2

async def test_group_level_hierarchy_filter(client, test_db, domain):
    # create groups of varying levels
    from app.auth.models import Group
    async with test_db() as s:
        for lvl, name in [(10, "g10"), (30, "g30"), (50, "g50"), (80, "g80")]:
            s.add(Group(domain_id=domain.domain_id, name=f"{name}-{uuid4().hex[:4]}", group_level=lvl))
        await s.commit()
    # low level user lvl 30 should only see <=30
    _, token_low, _ = await _make_level_user(test_db, domain, f"low-{uuid4().hex[:4]}", 30)
    headers_low = {"Authorization": f"Bearer {token_low}"}
    r = await client.get("/api/v1/groups", headers=headers_low)
    assert r.status_code == 200
    levels = [x["group_level"] for x in r.json()["data"]]
    assert all(l <= 30 for l in levels)
    assert 50 not in levels and 80 not in levels
    # superadmin lvl 80 sees all
    _, token_high, _ = await _make_level_user(test_db, domain, f"high-{uuid4().hex[:4]}", 80)
    headers_high = {"Authorization": f"Bearer {token_high}"}
    r2 = await client.get("/api/v1/groups", headers=headers_high)
    assert r2.status_code == 200
    levels2 = [x["group_level"] for x in r2.json()["data"]]
    assert 80 in levels2 or 50 in levels2

async def test_create_group_hierarchy_enforced(client, test_db, domain):
    _, token_low, _ = await _make_level_user(test_db, domain, f"creator-{uuid4().hex[:4]}", 30)
    headers_low = {"Authorization": f"Bearer {token_low}"}
    # try create level 50 >30 -> 403
    resp = await client.post("/api/v1/groups", json={"name": f"toohigh-{uuid4().hex[:4]}", "group_level": 50}, headers=headers_low)
    assert resp.status_code == 403
    # level 10 <30 should succeed
    resp2 = await client.post("/api/v1/groups", json={"name": f"lowok-{uuid4().hex[:4]}", "group_level": 10}, headers=headers_low)
    assert resp2.status_code == 201

async def test_group_patch_and_protected_toggle(client, auth_headers):
    r = await _create_group_via_api(client, auth_headers, f"tog-{uuid4().hex[:4]}", level=20, protected=False)
    assert r.status_code == 201
    gid = r.json()["group_uuid"]
    # patch level and protected
    resp = await client.patch(f"/api/v1/groups/{gid}", json={"group_level": 40, "group_protected": True, "description": "updated"}, headers=auth_headers)
    assert resp.status_code == 200
    assert resp.json()["group_level"] == 40
    assert resp.json()["group_protected"] is True
    assert resp.json()["description"] == "updated"
    # toggle via dedicated endpoint
    resp2 = await client.post(f"/api/v1/groups/{gid}/toggle", headers=auth_headers)
    assert resp2.status_code == 200
    assert resp2.json()["group_protected"] is False
    resp3 = await client.post(f"/api/v1/groups/{gid}/toggle", headers=auth_headers)
    assert resp3.json()["group_protected"] is True

async def test_group_copy_copies_permissions(client, auth_headers, test_db, domain):
    # create group with permission
    from app.auth.models import Permission, Group, GroupPermission
    # ensure permission exists
    async with test_db() as s:
        perm = Permission(name=f"perm-{uuid4().hex[:4]}")
        s.add(perm)
        await s.commit()
        await s.refresh(perm)
        perm_name = perm.name
        perm_uuid = perm.permission_uuid
    # create group
    r = await _create_group_via_api(client, auth_headers, f"orig-{uuid4().hex[:4]}", level=20)
    assert r.status_code == 201
    gid = r.json()["group_uuid"]
    # assign permission via PUT
    resp = await client.put(f"/api/v1/groups/{gid}/permissions", json={"permission_names": [perm_name]}, headers=auth_headers)
    assert resp.status_code == 200
    # copy
    resp2 = await client.post(f"/api/v1/groups/{gid}/copy", headers=auth_headers)
    assert resp2.status_code == 201
    new_gid = resp2.json()["group_uuid"]
    assert new_gid != gid
    assert resp2.json()["name"].startswith("orig")
    # new group should have same permission
    resp3 = await client.get(f"/api/v1/groups/{new_gid}/permissions", headers=auth_headers)
    assert resp3.status_code == 200
    names = [p["name"] for p in resp3.json()["data"] if p.get("assigned")]
    assert perm_name in names

async def test_group_bulk_actions(client, auth_headers):
    ids = []
    for i in range(3):
        r = await _create_group_via_api(client, auth_headers, f"bulk-{uuid4().hex[:4]}", level=10)
        assert r.status_code == 201
        ids.append(r.json()["group_uuid"])
    # bulk toggle
    resp = await client.post("/api/v1/groups/bulk", json={"action": "toggle", "ids": ids[:2]}, headers=auth_headers)
    assert resp.status_code == 200
    assert len(resp.json()["data"]) == 2
    # verify toggled to True
    for gid in ids[:2]:
        r = await client.get(f"/api/v1/groups/{gid}", headers=auth_headers)
        assert r.json()["group_protected"] is True
    # bulk copy
    resp2 = await client.post("/api/v1/groups/bulk", json={"action": "copy", "ids": [ids[0]]}, headers=auth_headers)
    assert resp2.status_code == 200
    assert resp2.json()["data"][0]["action"] == "copied"
    # bulk delete
    resp3 = await client.post("/api/v1/groups/bulk", json={"action": "delete", "ids": [ids[1]]}, headers=auth_headers)
    assert resp3.status_code == 200
    r = await client.get(f"/api/v1/groups/{ids[1]}", headers=auth_headers)
    assert r.status_code == 404

async def test_group_search_order_paging(client, auth_headers):
    # create distinct names
    base = uuid4().hex[:6]
    for name in [f"aaa-{base}", f"zzz-{base}", f"mmm-{base}"]:
        await _create_group_via_api(client, auth_headers, name, level=10)
    # search
    r = await client.get(f"/api/v1/groups?search=aaa-{base}", headers=auth_headers)
    assert r.status_code == 200
    assert any(f"aaa-{base}" in x["name"] for x in r.json()["data"])
    # order asc vs desc
    r_asc = await client.get(f"/api/v1/groups?order_by=group_name&order=asc&per_page=100", headers=auth_headers)
    r_desc = await client.get(f"/api/v1/groups?order_by=group_name&order=desc&per_page=100", headers=auth_headers)
    assert r_asc.json()["data"][0]["name"] != r_desc.json()["data"][0]["name"]
    # paging
    r1 = await client.get("/api/v1/groups?page=1&per_page=1", headers=auth_headers)
    r2 = await client.get("/api/v1/groups?page=2&per_page=1", headers=auth_headers)
    assert r1.status_code == 200 and r2.status_code == 200
    assert r1.json()["data"][0]["group_uuid"] != r2.json()["data"][0]["group_uuid"]
    assert "X-Total-Count" in r1.headers

async def test_group_members_delegation_and_superadmin_guard(client, test_db, domain):
    # admin lvl 50 creates sales lvl 30 - need wildcard to allow user creation
    _, token_admin, g_admin = await _make_level_user(test_db, domain, f"adm-{uuid4().hex[:4]}", 50, perms=["*"])
    headers_admin = {"Authorization": f"Bearer {token_admin}"}
    r = await _create_group_via_api(client, headers_admin, f"sales-{uuid4().hex[:4]}", level=30)
    assert r.status_code == 201
    gid_sales = r.json()["group_uuid"]
    # create bob lvl 30 member of sales
    _, token_bob, _ = await _make_level_user(test_db, domain, f"bob-{uuid4().hex[:4]}", 30)
    # need to make bob member of sales? Our _make_level_user creates a different group, not sales. So add via admin
    # fetch bob user uuid via DB or via API list
    # create charlie user via DB to avoid permission issues (admin has * anyway)
    from app.auth.models import User as U2
    from app.auth.service import hash_password
    from sqlalchemy import select as sel2
    async with test_db() as s:
        charlie = U2(username=f"charlie-{uuid4().hex[:4]}", domain_id=domain.domain_id, password_hash=hash_password("Password123!"), enabled=True, email=f"c-{uuid4().hex[:4]}@t.com")
        s.add(charlie)
        await s.commit()
        await s.refresh(charlie)
        charlie_uuid = str(charlie.user_uuid)
    # admin adds bob to sales (should succeed, sales 30 < admin 50)
    # get bob uuid
    from sqlalchemy import select
    from app.auth.models import User
    async with test_db() as s:
        res = await s.execute(select(User).where(User.username.like("bob-%")))
        # find bob we just created via helper - username starts with bob-
        # we need to find the latest bob
        users = (await s.execute(select(User).where(User.domain_id == domain.domain_id))).scalars().all()
        bob = [u for u in users if u.username.startswith("bob-")][-1]
        bob_uuid = str(bob.user_uuid)
    # admin adds bob to sales
    resp = await client.post(f"/api/v1/groups/{gid_sales}/users", json={"user_uuid": bob_uuid}, headers=headers_admin)
    if resp.status_code != 201:
        print("ADD BOB FAILED", resp.status_code, resp.text)
    assert resp.status_code == 201
    # bob now lvl 30, tries to add charlie to superadmin lvl 80 -> should be blocked
    # create superadmin group directly via DB (admin lvl 50 cannot create lvl 80 via API)
    from app.auth.models import Group as G2
    async with test_db() as s:
        sup_g = G2(domain_id=domain.domain_id, name="superadmin", group_level=80, group_protected=False)
        s.add(sup_g)
        await s.commit()
        await s.refresh(sup_g)
        gid_sup = str(sup_g.group_uuid)
    headers_bob = {"Authorization": f"Bearer {token_bob}"}
    resp2 = await client.post(f"/api/v1/groups/{gid_sup}/users", json={"user_uuid": charlie_uuid}, headers=headers_bob)
    assert resp2.status_code in (403, 404)
    # bob can add charlie to public lvl 10 (<30) -> should succeed
    r_pub = await _create_group_via_api(client, headers_admin, f"pub-{uuid4().hex[:4]}", level=10)
    assert r_pub.status_code == 201
    gid_pub = r_pub.json()["group_uuid"]
    resp3 = await client.post(f"/api/v1/groups/{gid_pub}/users", json={"user_uuid": charlie_uuid}, headers=headers_bob)
    assert resp3.status_code == 201
    # verify members list
    resp4 = await client.get(f"/api/v1/groups/{gid_pub}/users", headers=headers_admin)
    assert resp4.status_code == 200
    assert any(m["user_uuid"] == charlie_uuid for m in resp4.json()["data"])
    # remove
    resp5 = await client.delete(f"/api/v1/groups/{gid_pub}/users/{charlie_uuid}", headers=headers_admin)
    assert resp5.status_code == 204

async def test_group_permissions_view_and_assign(client, auth_headers, test_db):
    from app.auth.models import Permission
    async with test_db() as s:
        p1 = Permission(name=f"permA-{uuid4().hex[:4]}")
        p2 = Permission(name=f"permB-{uuid4().hex[:4]}")
        s.add_all([p1, p2])
        await s.commit()
        await s.refresh(p1)
        await s.refresh(p2)
        n1 = p1.name
        n2 = p2.name
    r = await _create_group_via_api(client, auth_headers, f"permgrp-{uuid4().hex[:4]}", level=20)
    gid = r.json()["group_uuid"]
    # assign only p1
    resp = await client.put(f"/api/v1/groups/{gid}/permissions", json={"permission_names": [n1]}, headers=auth_headers)
    assert resp.status_code == 200
    # get with view assigned
    resp2 = await client.get(f"/api/v1/groups/{gid}/permissions?view=assigned", headers=auth_headers)
    assert resp2.status_code == 200
    assigned = [x for x in resp2.json()["data"] if x["assigned"]]
    assert len(assigned) == 1 and assigned[0]["name"] == n1
    # view unassigned should contain p2
    resp3 = await client.get(f"/api/v1/groups/{gid}/permissions?view=unassigned", headers=auth_headers)
    unassigned_names = [x["name"] for x in resp3.json()["data"] if not x["assigned"]]
    assert n2 in unassigned_names
    # search filter
    resp4 = await client.get(f"/api/v1/groups/{gid}/permissions?search={n1[:6]}", headers=auth_headers)
    assert any(x["name"] == n1 for x in resp4.json()["data"])

async def test_delete_protected_group_blocked(client, test_db, domain, auth_headers):
    # create protected group via API with protected=true
    resp = await client.post("/api/v1/groups", json={"name": f"prot-{uuid4().hex[:4]}", "group_level": 10, "group_protected": True}, headers=auth_headers)
    assert resp.status_code == 201, resp.text
    gid = resp.json()["group_uuid"]
    # make lvl 50 admin without *
    _, token_low, _ = await _make_level_user(test_db, domain, f"admin50-{uuid4().hex[:4]}", 50)
    headers_low = {"Authorization": f"Bearer {token_low}"}
    # attempt DELETE as admin (lvl 50, no *) => 403
    resp2 = await client.delete(f"/api/v1/groups/{gid}", headers=headers_low)
    assert resp2.status_code == 403, f"expected 403 got {resp2.status_code} {resp2.text}"
    # attempt DELETE as superadmin (*) => 204
    resp3 = await client.delete(f"/api/v1/groups/{gid}", headers=auth_headers)
    assert resp3.status_code == 204, resp3.text


async def test_groups_show_all_requires_permission(client, test_db, domain, other_domain):
    # low user with only group_view (no group_all) cannot see other domain even with show=all
    _, token_low, _ = await _make_level_user(test_db, domain, f"low2-{uuid4().hex[:4]}", 10, perms=["group_view"])
    headers_low = {"Authorization": f"Bearer {token_low}"}
    # create domain-specific group in other_domain
    from app.auth.models import Group
    async with test_db() as s:
        other_g = Group(domain_id=other_domain.domain_id, name=f"otherOnly-{uuid4().hex[:4]}", group_level=10)
        s.add(other_g)
        await s.commit()
        other_name = other_g.name
    # low user without show_all should not see other domain's group
    r = await client.get("/api/v1/groups", headers=headers_low)
    assert r.status_code == 200, r.text
    assert other_name not in [x["name"] for x in r.json()["data"]]
    # with show_all param but without permission, our code falls back to false, so still not see
    r2 = await client.get("/api/v1/groups?show=all", headers=headers_low)
    assert r2.status_code == 200, r2.text
    assert other_name not in [x["name"] for x in r2.json()["data"]]
