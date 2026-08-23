import pytest
from uuid import uuid4
from sqlalchemy import select, func

pytestmark = pytest.mark.asyncio

async def _create_user_via_api(client, headers, username=None, domain_id=None, extra=None):
    uname = username or f"user-{uuid4().hex[:4]}"
    payload = {"username": uname, "password": "Password123!", "email": f"{uname}@example.com", "enabled": True}
    if domain_id:
        payload["domain_uuid"] = str(domain_id)
    if extra:
        payload.update(extra)
    resp = await client.post("/api/v1/users", json=payload, headers=headers)
    return resp

async def _make_level_user(test_db, domain, username, level, perms=None):
    from app.auth.models import Group, User, UserGroup, Permission, GroupPermission
    from app.auth.service import create_token, hash_password
    from sqlalchemy import select as sel
    if perms is None:
        perms = ["user_view", "user_add", "user_edit", "user_delete", "user_group_view", "user_group_add", "user_group_delete", "group_view", "group_add", "group_edit", "group_delete", "group_member_view", "group_member_add", "group_member_delete", "group_permission_view", "group_permission_edit"]
    async with test_db() as s:
        # ensure perms exist and link to group
        for pname in perms:
            r = await s.execute(sel(Permission).where(Permission.name == pname))
            if not r.scalar_one_or_none():
                s.add(Permission(name=pname))
        await s.commit()
        g = Group(domain_id=domain.domain_id, name=f"lvl{level}-{username}-{uuid4().hex[:4]}", group_level=level)
        s.add(g)
        await s.commit()
        await s.refresh(g)
        for pname in perms:
            r = await s.execute(sel(Permission).where(Permission.name == pname))
            perm = r.scalar_one()
            s.add(GroupPermission(group_uuid=g.group_uuid, permission_uuid=perm.permission_uuid, group_name=g.name, permission_name=pname, permission_assigned=True))
        await s.commit()
        u = User(username=username, domain_id=domain.domain_id, password_hash=hash_password("password123"), enabled=True)
        s.add(u)
        await s.commit()
        await s.refresh(u)
        s.add(UserGroup(user_uuid=u.user_uuid, group_uuid=g.group_uuid, domain_id=domain.domain_id, group_name=g.name))
        await s.commit()
        token = create_token(u, permissions=perms)
        return u, token, g

async def test_create_user_with_all_legacy_fields(client, auth_headers, domain):
    resp = await _create_user_via_api(client, auth_headers, username=f"full-{uuid4().hex[:4]}", extra={
        "user_status": "Available",
        "user_type": "virtual",
        "user_language": "en-us",
        "user_time_zone": "America/New_York",
        "contact_organization": "Acme",
        "contact_name_given": "John",
        "contact_name_family": "Doe",
        "api_key": "testkey123",
        "user_totp_secret": "JBSWY3DPEHPK3PXP",
        "group_uuid_name": ""  # no group
    })
    assert resp.status_code == 201, resp.text
    data = resp.json()
    assert data["username"].startswith("full-")
    assert data["user_status"] == "Available"
    assert data["user_type"] == "virtual"
    assert data["contact_organization"] == "Acme"
    assert data["contact_name"] is not None
    assert data["user_language"] == "en-us"
    assert data["user_time_zone"] == "America/New_York"
    assert data["api_key"] == "testkey123"

async def test_create_user_duplicate_username_per_domain(client, auth_headers, domain, other_domain, other_auth_headers):
    uname = f"dup-{uuid4().hex[:4]}"
    r1 = await _create_user_via_api(client, auth_headers, username=uname)
    assert r1.status_code == 201
    r2 = await _create_user_via_api(client, auth_headers, username=uname)
    assert r2.status_code == 409
    # same username in other domain should succeed (per-domain unique)
    r3 = await _create_user_via_api(client, other_auth_headers, username=uname)
    assert r3.status_code == 201

async def test_users_list_search_and_group_names(client, auth_headers, domain):
    base = uuid4().hex[:6]
    # create user with specific org
    await _create_user_via_api(client, auth_headers, username=f"search-{base}-a", extra={"contact_organization": f"Org{base}", "email": f"a-{base}@t.com"})
    await _create_user_via_api(client, auth_headers, username=f"search-{base}-b", extra={"email": f"b-{base}@t.com"})
    # create group and assign to first user to test group_names search
    from app.auth.models import Group, User, UserGroup
    # we will not test group_names via API assignment here, but list should support search by username
    r = await client.get(f"/api/v1/users?search=search-{base}-a", headers=auth_headers)
    assert r.status_code == 200
    assert any(f"search-{base}-a" in u["username"] for u in r.json()["data"])
    # search by organization
    r2 = await client.get(f"/api/v1/users?search=Org{base}", headers=auth_headers)
    assert any(u["contact_organization"] == f"Org{base}" for u in r2.json()["data"])

async def test_users_list_paging_and_order(client, auth_headers):
    base = uuid4().hex[:4]
    for i in range(3):
        await _create_user_via_api(client, auth_headers, username=f"page-{base}-{i:02d}")
    r1 = await client.get("/api/v1/users?page=1&per_page=1&order_by=username&order=asc", headers=auth_headers)
    r2 = await client.get("/api/v1/users?page=2&per_page=1&order_by=username&order=asc", headers=auth_headers)
    assert r1.status_code == 200 and r2.status_code == 200
    assert r1.json()["data"][0]["user_uuid"] != r2.json()["data"][0]["user_uuid"]
    assert "X-Total-Count" in r1.headers
    # order desc should reverse
    r_asc = await client.get("/api/v1/users?order_by=username&order=asc&per_page=100", headers=auth_headers)
    r_desc = await client.get("/api/v1/users?order_by=username&order=desc&per_page=100", headers=auth_headers)
    assert r_asc.json()["data"][0]["username"] != r_desc.json()["data"][0]["username"]

async def test_users_list_level_filter_and_show_all(client, test_db, domain, other_domain):
    # create high level group and user in domain
    from app.auth.models import Group, User, UserGroup
    from app.auth.service import hash_password
    async with test_db() as s:
        g_high = Group(domain_id=domain.domain_id, name=f"high-{uuid4().hex[:4]}", group_level=80)
        s.add(g_high)
        await s.commit()
        await s.refresh(g_high)
        u_high = User(username=f"highuser-{uuid4().hex[:4]}", domain_id=domain.domain_id, password_hash=hash_password("p"), enabled=True)
        s.add(u_high)
        await s.commit()
        s.add(UserGroup(user_uuid=u_high.user_uuid, group_uuid=g_high.group_uuid, domain_id=domain.domain_id, group_name=g_high.name))
        await s.commit()
        high_name = u_high.username
        # also create low user
        g_low = Group(domain_id=domain.domain_id, name=f"low-{uuid4().hex[:4]}", group_level=10)
        s.add(g_low)
        await s.commit()
        u_low = User(username=f"lowuser-{uuid4().hex[:4]}", domain_id=domain.domain_id, password_hash=hash_password("p"), enabled=True)
        s.add(u_low)
        await s.commit()
        s.add(UserGroup(user_uuid=u_low.user_uuid, group_uuid=g_low.group_uuid, domain_id=domain.domain_id, group_name=g_low.name))
        await s.commit()
        # other domain user
        u_other = User(username=f"other-{uuid4().hex[:4]}", domain_id=other_domain.domain_id, password_hash=hash_password("p"), enabled=True)
        s.add(u_other)
        await s.commit()
    # low level user should not see high user
    _, token_low, _ = await _make_level_user(test_db, domain, f"viewer-{uuid4().hex[:4]}", 10)
    headers_low = {"Authorization": f"Bearer {token_low}"}
    r = await client.get("/api/v1/users", headers=headers_low)
    assert r.status_code == 200
    usernames = [u["username"] for u in r.json()["data"]]
    assert high_name not in usernames
    # admin with * sees all in domain
    from tests.conftest import _make_user as _mu
    # use existing auth_headers from admin (we need to get one via fixture? We'll create new admin)
    from app.domains.models import Domain as Dom
    # instead use high user token with *? Create admin via helper
    admin_tok = None
    async with test_db() as s:
        # create admin with *
        from app.auth.models import User as U
        from app.auth.service import create_token, hash_password
        adm = U(username=f"adm-{uuid4().hex[:4]}", domain_id=domain.domain_id, password_hash=hash_password("p"), enabled=True)
        s.add(adm)
        await s.commit()
        admin_tok = create_token(adm, permissions=["*"])
    headers_admin = {"Authorization": f"Bearer {admin_tok}"}
    r2 = await client.get("/api/v1/users", headers=headers_admin)
    assert any(u["username"] == high_name for u in r2.json()["data"])
    # other domain isolation: admin of domain should not see other domain without show_all
    r3 = await client.get("/api/v1/users", headers=headers_admin)
    # other user is in other_domain, should not appear
    async with test_db() as s:
        from sqlalchemy import select
        from app.auth.models import User as U
        res = await s.execute(select(U).where(U.username.like("other-%")))
        other_u = res.scalars().first()
        other_name = other_u.username if other_u else None
    if other_name:
        assert other_name not in [u["username"] for u in r3.json()["data"]]
        # with show_all and user_all? admin has * so show_all should reveal?
        r4 = await client.get("/api/v1/users?show=all", headers=headers_admin)
        assert r4.status_code == 200

async def test_user_patch_and_domain_change(client, auth_headers, domain, other_domain, test_db):
    resp = await _create_user_via_api(client, auth_headers, username=f"patch-{uuid4().hex[:4]}")
    assert resp.status_code == 201
    uid = resp.json()["user_uuid"]
    # patch status, type, language, etc
    resp2 = await client.patch(f"/api/v1/users/{uid}", json={
        "user_status": "On Break",
        "user_type": "virtual",
        "user_language": "fr-fr",
        "user_time_zone": "Europe/Paris",
        "enabled": False,
        "email": "newemail@example.com"
    }, headers=auth_headers)
    assert resp2.status_code == 200
    assert resp2.json()["user_status"] == "On Break"
    assert resp2.json()["user_language"] == "fr-fr"
    assert resp2.json()["enabled"] is False
    # verify via GET
    resp3 = await client.get(f"/api/v1/users/{uid}", headers=auth_headers)
    assert resp3.status_code == 200
    assert resp3.json()["user_status"] == "On Break"
    assert resp3.json()["groups"] is not None
    # domain change requires user_domain perm (admin has *)
    resp4 = await client.patch(f"/api/v1/users/{uid}", json={"domain_uuid": str(other_domain.domain_id)}, headers=auth_headers)
    assert resp4.status_code == 200
    assert resp4.json()["domain_id"] == str(other_domain.domain_id)

async def test_user_group_delegation(client, test_db, domain):
    # admin lvl 50 creates group lvl 30 sales
    _, token_admin, _ = await _make_level_user(test_db, domain, f"adm2-{uuid4().hex[:4]}", 50)
    headers_admin = {"Authorization": f"Bearer {token_admin}"}
    r = await client.post("/api/v1/groups", json={"name": f"sales-{uuid4().hex[:4]}", "group_level": 30}, headers=headers_admin)
    assert r.status_code == 201
    gid_sales = r.json()["group_uuid"]
    # low user lvl 30
    _, token_low, _ = await _make_level_user(test_db, domain, f"low3-{uuid4().hex[:4]}", 30)
    headers_low = {"Authorization": f"Bearer {token_low}"}
    # low user tries to assign charlie to admin lvl 50 -> fail
    resp_charlie = await _create_user_via_api(client, headers_admin, username=f"charlie-{uuid4().hex[:4]}")
    charlie_uuid = resp_charlie.json()["user_uuid"]
    # create admin group lvl 50 to test delegation failure
    r_admin_grp = await client.post("/api/v1/groups", json={"name": f"admgrp-{uuid4().hex[:4]}", "group_level": 50}, headers=headers_admin)
    gid_admin = r_admin_grp.json()["group_uuid"]
    # low tries to add charlie to admin group -> 403 (level 50 >=30 and not member)
    resp = await client.post(f"/api/v1/users/{charlie_uuid}/groups", json={"group_uuid": gid_admin}, headers=headers_low)
    assert resp.status_code == 403
    # low can add to sales (same level but need membership? Our logic: same level requires membership. Low is not member of sales yet, but level 30 ==30, so should fail unless member
    resp2 = await client.post(f"/api/v1/users/{charlie_uuid}/groups", json={"group_uuid": gid_sales}, headers=headers_low)
    # Since low not member of sales, same level should be 403
    assert resp2.status_code == 403
    # admin adds low to sales first
    # need bob uuid - find low user uuid
    from sqlalchemy import select
    from app.auth.models import User
    async with test_db() as s:
        res = await s.execute(select(User).where(User.domain_id == domain.domain_id))
        users = res.scalars().all()
        low_u = [u for u in users if u.username.startswith("low3-")][-1]
        low_uuid = str(low_u.user_uuid)
    resp3 = await client.post(f"/api/v1/users/{low_uuid}/groups", json={"group_uuid": gid_sales}, headers=headers_admin)
    assert resp3.status_code == 201
    # now low is member of sales, can assign charlie to sales (same level but is member)
    # need new token for low to reflect new group? Level still 30, but membership now includes sales, so should pass
    # Our service checks my_group_uuids fresh from DB, so should pass even with old token
    resp4 = await client.post(f"/api/v1/users/{charlie_uuid}/groups", json={"group_uuid": gid_sales}, headers=headers_low)
    assert resp4.status_code == 201

async def test_user_bulk_actions(client, auth_headers):
    ids = []
    for i in range(3):
        r = await _create_user_via_api(client, auth_headers, username=f"bulkU-{uuid4().hex[:4]}")
        assert r.status_code == 201
        ids.append(r.json()["user_uuid"])
    # bulk toggle first 2
    resp = await client.post("/api/v1/users/bulk", json={"action": "toggle", "ids": ids[:2]}, headers=auth_headers)
    assert resp.status_code == 200
    assert len(resp.json()["data"]) == 2
    # verify toggled to False (original True)
    for uid in ids[:2]:
        r = await client.get(f"/api/v1/users/{uid}", headers=auth_headers)
        assert r.json()["enabled"] is False
    # bulk copy one
    resp2 = await client.post("/api/v1/users/bulk", json={"action": "copy", "ids": [ids[0]]}, headers=auth_headers)
    assert resp2.status_code == 200
    assert resp2.json()["data"][0]["action"] == "copied"
    new_name = resp2.json()["data"][0]["username"]
    assert "copy" in new_name
    # bulk delete
    resp3 = await client.post("/api/v1/users/bulk", json={"action": "delete", "ids": [ids[1]]}, headers=auth_headers)
    assert resp3.status_code == 200
    r = await client.get(f"/api/v1/users/{ids[1]}", headers=auth_headers)
    assert r.status_code == 404

async def test_user_settings_endpoints(client, auth_headers):
    r = await _create_user_via_api(client, auth_headers, username=f"sett-{uuid4().hex[:4]}")
    uid = r.json()["user_uuid"]
    # get settings empty? may have lang/tz if created
    resp = await client.get(f"/api/v1/users/{uid}/settings", headers=auth_headers)
    assert resp.status_code == 200
    # put settings
    resp2 = await client.put(f"/api/v1/users/{uid}/settings", json={"settings": [{"category": "domain", "subcategory": "language", "name": "code", "value": "de-de", "enabled": True}]}, headers=auth_headers)
    assert resp2.status_code == 200
    resp3 = await client.get(f"/api/v1/users/{uid}/settings", headers=auth_headers)
    assert any(s["value"] == "de-de" for s in resp3.json()["data"])

async def test_superadmin_user_protection(client, test_db, domain):
    # create superadmin group and user
    from app.auth.models import Group, User, UserGroup, Permission, GroupPermission
    from app.auth.service import hash_password, create_token
    from sqlalchemy import select as sel2
    async with test_db() as s:
        # ensure perms
        for pname in ["user_view", "user_delete", "user_edit"]:
            r = await s.execute(sel2(Permission).where(Permission.name == pname))
            if not r.scalar_one_or_none():
                s.add(Permission(name=pname))
        await s.commit()
        g_sup = Group(domain_id=domain.domain_id, name="superadmin", group_level=80, group_protected=False)
        s.add(g_sup)
        await s.commit()
        await s.refresh(g_sup)
        for pname in ["user_delete", "user_edit", "user_view"]:
            r = await s.execute(sel2(Permission).where(Permission.name == pname))
            perm = r.scalar_one()
            s.add(GroupPermission(group_uuid=g_sup.group_uuid, permission_uuid=perm.permission_uuid, group_name=g_sup.name, permission_name=pname, permission_assigned=True))
        await s.commit()
        u_sup = User(username=f"sup-{uuid4().hex[:4]}", domain_id=domain.domain_id, password_hash=hash_password("p"), enabled=True)
        s.add(u_sup)
        await s.commit()
        s.add(UserGroup(user_uuid=u_sup.user_uuid, group_uuid=g_sup.group_uuid, domain_id=domain.domain_id, group_name="superadmin"))
        await s.commit()
        token_sup = create_token(u_sup, permissions=["user_view", "user_delete", "user_edit"])
        g_low = Group(domain_id=domain.domain_id, name=f"low-{uuid4().hex[:4]}", group_level=10)
        s.add(g_low)
        await s.commit()
        u_low = User(username=f"low-{uuid4().hex[:4]}", domain_id=domain.domain_id, password_hash=hash_password("p"), enabled=True)
        s.add(u_low)
        await s.commit()
        s.add(UserGroup(user_uuid=u_low.user_uuid, group_uuid=g_low.group_uuid, domain_id=domain.domain_id, group_name=g_low.name))
        await s.commit()
        token_low = create_token(u_low, permissions=["user_view"])
        sup_uuid = str(u_sup.user_uuid)
    headers_low = {"Authorization": f"Bearer {token_low}"}
    headers_sup = {"Authorization": f"Bearer {token_sup}"}
    # low tries to delete superadmin -> 403
    resp = await client.delete(f"/api/v1/users/{sup_uuid}", headers=headers_low)
    assert resp.status_code == 403
    # low tries to edit superadmin
    resp2 = await client.patch(f"/api/v1/users/{sup_uuid}", json={"email": "hacked@t.com"}, headers=headers_low)
    assert resp2.status_code == 403
    # superadmin can delete low
    async with test_db() as s:
        from sqlalchemy import select
        from app.auth.models import User as U
        res = await s.execute(select(U).where(U.username.like("low-%")))
        low_u = res.scalars().first()
        low_uuid = str(low_u.user_uuid)
    resp3 = await client.delete(f"/api/v1/users/{low_uuid}", headers=headers_sup)
    assert resp3.status_code == 204
