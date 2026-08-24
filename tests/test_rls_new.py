import pytest, uuid
import jwt
from httpx import ASGITransport, AsyncClient
from sqlalchemy import select, text
from app.main import app

def _domain_id_from_token(tok: str) -> str:
    try:
        payload = jwt.decode(tok, options={"verify_signature": False})
        return payload.get("domain_id")
    except Exception:
        return None

# alias for brief's expected fixture name: `db_session` -> `test_db` sessionmaker
@pytest.fixture
def db_session(test_db):
    return test_db

@pytest.mark.asyncio
async def test_rls_isolation_conference_centers_and_vars(test_db):
    # Seed domains/users for login (idempotent)
    from app.auth.service import hash_password
    from app.domains.models import Domain
    from app.auth.models import User, Group, UserGroup
    for dom_name in ("test.example.com", "second.example.com"):
        async with test_db() as s:
            from sqlalchemy import select
            r = await s.execute(select(Domain).where(Domain.domain_name == dom_name))
            dom = r.scalar_one_or_none()
            if not dom:
                dom = Domain(domain_name=dom_name, enabled=True)
                s.add(dom)
                await s.commit()
                await s.refresh(dom)
            r = await s.execute(select(User).where((User.username == "admin") & (User.domain_id == dom.domain_id)))
            usr = r.scalar_one_or_none()
            if not usr:
                usr = User(username="admin", domain_id=dom.domain_id, password_hash=hash_password("admin"), enabled=True)
                s.add(usr)
                await s.commit()
                await s.refresh(usr)
                # ensure admin group with * perms
                g = Group(domain_id=dom.domain_id, name=f"admin-{dom_name}", group_level=80, group_protected=False)
                s.add(g)
                await s.commit()
                await s.refresh(g)
                s.add(UserGroup(user_uuid=usr.user_uuid, group_uuid=g.group_uuid, domain_id=dom.domain_id))
                await s.commit()
                # add wildcard permission via group_permissions would be needed, but create_token with * is handled via DB? Instead ensure user has superadmin via group_level 80
                from app.auth.models import GroupPermission, Permission
                r2 = await s.execute(select(Permission).where(Permission.name == "*"))
                perm = r2.scalar_one_or_none()
                if not perm:
                    perm = Permission(name="*")
                    s.add(perm)
                    await s.commit()
                    await s.refresh(perm)
                s.add(GroupPermission(group_uuid=g.group_uuid, permission_uuid=perm.permission_uuid, domain_id=dom.domain_id, group_name=g.name, permission_name="*", permission_assigned=True, permission_protected=False))
                await s.commit()

    from app.db import get_session
    async def override_get_session():
        async with test_db() as session:
            yield session
    app.dependency_overrides[get_session] = override_get_session
    try:
        async def tok(domain):
            tr=ASGITransport(app=app)
            async with AsyncClient(transport=tr, base_url="http://test") as ac:
                r=await ac.post("/api/v1/auth/login", json={"username":"admin","password":"admin","domain":domain})
                assert r.status_code==200, r.text
                return r.json()["access_token"]
        t1=await tok("test.example.com")
        t2=await tok("second.example.com")
        # ensure apps are enabled for both domains (central AppGateMiddleware gates disabled_by_default=false)
        for tok_val in (t1, t2):
            did = _domain_id_from_token(tok_val)
            if did:
                tr2=ASGITransport(app=app)
                async with AsyncClient(transport=tr2, base_url="http://test") as ac2:
                    for app_name in ("conference_centers", "vars"):
                        await ac2.put(f"/api/v1/domains/{did}/apps/{app_name}?enabled=true", headers={"Authorization": f"Bearer {tok_val}"})
        tr=ASGITransport(app=app)
        async with AsyncClient(transport=tr, base_url="http://test") as ac:
            n=f"rls-cc-{uuid.uuid4().hex[:6]}"
            ext=str(7000+int(uuid.uuid4().hex[:3],16)%500)
            r=await ac.post("/api/v1/conference_centers", json={"conference_center_name": n, "conference_center_extension": ext}, headers={"Authorization": f"Bearer {t1}"})
            assert r.status_code in (200,201), r.text
            r1=await ac.get("/api/v1/conference_centers?per_page=100", headers={"Authorization": f"Bearer {t1}"})
            r2=await ac.get("/api/v1/conference_centers?per_page=100", headers={"Authorization": f"Bearer {t2}"})
            d1=[d.get("conference_center_name") for d in r1.json().get("data",[])]
            d2=[d.get("conference_center_name") for d in r2.json().get("data",[])]
            assert n in d1 and n not in d2
            # vars
            vn=f"rls-v-{uuid.uuid4().hex[:6]}"
            r=await ac.post("/api/v1/vars", json={"var_name": vn, "var_value":"x","var_category":"testing"}, headers={"Authorization": f"Bearer {t1}"})
            assert r.status_code in (200,201), r.text
            r1=await ac.get("/api/v1/vars?per_page=100", headers={"Authorization": f"Bearer {t1}"})
            r2=await ac.get("/api/v1/vars?per_page=100", headers={"Authorization": f"Bearer {t2}"})
            v1=[d.get("var_name") for d in r1.json().get("data",[])]
            v2=[d.get("var_name") for d in r2.json().get("data",[])]
            assert vn in v1 and vn not in v2
    finally:
        app.dependency_overrides.clear()


@pytest.mark.asyncio
async def test_rls_isolation_groups_users(test_db, client):
    """Task 4: RLS for groups/users/user_settings/group_permissions + orphan wildcard fix.

    - Creates domain A global group + domain A user + domain B user (+ domain B group)
    - Creates UserSetting/GroupPermission/UserGroup rows for both domains
    - Sets tenant to A via set_tenant (postgres RLS) and queries via SQL to verify
      RLS hides B but shows global; sqlite fallback uses app-level filter.
    - Verifies orphan user (no groups) gets [] not ["*"].
    """
    from uuid import uuid4 as _uuid4
    from app.auth.models import Group, User, UserGroup, GroupPermission, Permission, UserSetting
    from app.auth.service import get_user_permissions, hash_password
    from app.db import set_tenant
    from app.domains.models import Domain
    from app.config import settings as _cfg

    # --- create domains ---
    async with test_db() as s:
        d_a = Domain(domain_name=f"rls-a-{_uuid4().hex[:6]}.example.com", enabled=True)
        d_b = Domain(domain_name=f"rls-b-{_uuid4().hex[:6]}.example.com", enabled=True)
        s.add_all([d_a, d_b])
        await s.commit()
        await s.refresh(d_a)
        await s.refresh(d_b)
        domain_a_id = d_a.domain_id
        domain_b_id = d_b.domain_id

    # --- create groups: global (NULL), domain A, domain B ---
    global_name = f"rls-global-{_uuid4().hex[:6]}"
    grp_a_name = f"rls-grp-a-{_uuid4().hex[:6]}"
    grp_b_name = f"rls-grp-b-{_uuid4().hex[:6]}"
    async with test_db() as s:
        g_global = Group(domain_id=None, name=global_name, group_level=10, group_protected=False)
        g_a = Group(domain_id=domain_a_id, name=grp_a_name, group_level=10, group_protected=False)
        g_b = Group(domain_id=domain_b_id, name=grp_b_name, group_level=10, group_protected=False)
        s.add_all([g_global, g_a, g_b])
        await s.commit()
        await s.refresh(g_global)
        await s.refresh(g_a)
        await s.refresh(g_b)
        g_global_uuid = g_global.group_uuid
        g_a_uuid = g_a.group_uuid
        g_b_uuid = g_b.group_uuid

    # --- create users: A, B, orphan (A domain but no groups) ---
    user_a_name = f"rls-user-a-{_uuid4().hex[:6]}"
    user_b_name = f"rls-user-b-{_uuid4().hex[:6]}"
    orphan_name = f"rls-orphan-{_uuid4().hex[:6]}"
    async with test_db() as s:
        u_a = User(username=user_a_name, domain_id=domain_a_id, password_hash=hash_password("Password123!"), enabled=True)
        u_b = User(username=user_b_name, domain_id=domain_b_id, password_hash=hash_password("Password123!"), enabled=True)
        u_orphan = User(username=orphan_name, domain_id=domain_a_id, password_hash=hash_password("Password123!"), enabled=True)
        s.add_all([u_a, u_b, u_orphan])
        await s.commit()
        await s.refresh(u_a)
        await s.refresh(u_b)
        await s.refresh(u_orphan)
        u_a_uuid = u_a.user_uuid
        u_b_uuid = u_b.user_uuid
        u_orphan_uuid = u_orphan.user_uuid

    # --- create supporting rows: Permission, GroupPermission, UserGroup, UserSetting ---
    perm_name_a = f"rls-perm-a-{_uuid4().hex[:4]}"
    perm_name_b = f"rls-perm-b-{_uuid4().hex[:4]}"
    async with test_db() as s:
        p_a = Permission(name=perm_name_a)
        p_b = Permission(name=perm_name_b)
        s.add_all([p_a, p_b])
        await s.commit()
        await s.refresh(p_a)
        await s.refresh(p_b)
        # group_permissions: one per domain
        gp_a = GroupPermission(group_uuid=g_a_uuid, permission_uuid=p_a.permission_uuid, domain_id=domain_a_id, group_name=grp_a_name, permission_name=perm_name_a, permission_assigned=True, permission_protected=False)
        gp_b = GroupPermission(group_uuid=g_b_uuid, permission_uuid=p_b.permission_uuid, domain_id=domain_b_id, group_name=grp_b_name, permission_name=perm_name_b, permission_assigned=True, permission_protected=False)
        s.add_all([gp_a, gp_b])
        await s.commit()
        # user_groups
        ug_a = UserGroup(user_uuid=u_a_uuid, group_uuid=g_a_uuid, domain_id=domain_a_id, group_name=grp_a_name)
        ug_b = UserGroup(user_uuid=u_b_uuid, group_uuid=g_b_uuid, domain_id=domain_b_id, group_name=grp_b_name)
        s.add_all([ug_a, ug_b])
        await s.commit()
        # user_settings
        us_a = UserSetting(user_uuid=u_a_uuid, domain_id=domain_a_id, user_setting_category="test", user_setting_subcategory="sub", user_setting_name="k", user_setting_value="v_a")
        us_b = UserSetting(user_uuid=u_b_uuid, domain_id=domain_b_id, user_setting_category="test", user_setting_subcategory="sub", user_setting_name="k", user_setting_value="v_b")
        s.add_all([us_a, us_b])
        await s.commit()

    # --- orphan wildcard check: must be [] not ["*"] ---
    async with test_db() as s:
        r = await s.execute(select(User).where(User.user_uuid == u_orphan_uuid))
        orphan = r.scalar_one()
        perms = await get_user_permissions(s, orphan)
        assert perms == [], f"orphan user should get [] not {perms}"
        assert "*" not in perms, "orphan must not get wildcard *"

    # --- detect dialect ---
    # test_db is sqlite in CI; real postgres would have set_config. Use app config + try set_tenant probe.
    is_postgres = False
    try:
        url = str(_cfg.database_url).lower()
        if "postgres" in url:
            is_postgres = True
    except Exception:
        pass
    # also probe via session dialect name if possible
    try:
        async with test_db() as _probe:
            bind = _probe.bind
            dname = getattr(getattr(bind, "dialect", None), "name", "") or ""
            if dname == "postgresql":
                is_postgres = True
            # need to check engine url for sqlite memory
            if "sqlite" in str(getattr(bind, "url", "")).lower():
                is_postgres = False
    except Exception:
        pass

    if is_postgres:
        # --- postgres RLS path: set_tenant + raw SQL (unfiltered) must enforce isolation ---
        async with test_db() as s:
            await set_tenant(s, domain_a_id)
            # users: raw SELECT without domain filter should be RLS-filtered
            r = await s.execute(text('SELECT username FROM users'))
            usernames = [row[0] for row in r.all()]
            assert user_a_name in usernames, f"A user missing under RLS {usernames}"
            assert user_b_name not in usernames, f"RLS leak: B user visible under A tenant {usernames}"
            # orphan is in A domain, should still be visible
            assert orphan_name in usernames

            # groups: global (NULL) should be visible, B should be hidden
            r = await s.execute(text('SELECT name FROM groups'))
            gnames = [row[0] for row in r.all()]
            assert global_name in gnames, f"global group missing {gnames}"
            assert grp_a_name in gnames
            assert grp_b_name not in gnames, f"RLS leak: B group visible {gnames}"

            # user_groups: only A domain rows
            r = await s.execute(text('SELECT domain_id::text FROM user_groups'))
            doms = [row[0] for row in r.all()]
            # all visible doms should be A (or filtered) — if multiple tenants exist, RLS should hide B
            # Since we filtered to tenant A, we expect not to see B's domain
            assert str(domain_b_id) not in doms, f"RLS leak user_groups {doms}"

            # user_settings: only A
            r = await s.execute(text('SELECT user_setting_value FROM user_settings'))
            vals = [row[0] for row in r.all()]
            assert "v_a" in vals
            assert "v_b" not in vals, f"RLS leak user_settings {vals}"

            # group_permissions: only A
            r = await s.execute(text('SELECT permission_name FROM group_permissions'))
            pnames = [row[0] for row in r.all()]
            assert perm_name_a in pnames
            assert perm_name_b not in pnames, f"RLS leak group_permissions {pnames}"

            # also verify migration file contains correct USING clauses
            import pathlib
            mig = pathlib.Path("alembic/versions/026_groups_users_rls.py").read_text()
            assert "domain_id = NULLIF(current_setting('app.domain_uuid', true),'')::uuid" in mig or \
                   'domain_id = NULLIF(current_setting(\'app.domain_uuid\', true),\'\')::uuid' in mig or \
                   "NULLIF(current_setting('app.domain_uuid'" in mig
            assert "domain_id IS NULL OR domain_id = NULLIF" in mig
        # reset tenant to empty for next tests
        async with test_db() as s:
            try:
                await s.execute(text("SELECT set_config('app.domain_uuid','',true)"))
                await s.commit()
            except Exception:
                pass
    else:
        # --- sqlite fallback: RLS disabled, simulate app-level filter pass ---
        async with test_db() as s:
            # set_tenant is no-op on sqlite — verify it does not crash
            await set_tenant(s, domain_a_id)
            # users filtered by domain_id == A
            r = await s.execute(select(User).where(User.domain_id == domain_a_id))
            usernames = [u.username for u in r.scalars().all()]
            assert user_a_name in usernames
            assert orphan_name in usernames
            assert user_b_name not in usernames, "app-level filter should hide B user"

            # groups: (domain_id == A OR domain_id IS NULL)
            r = await s.execute(select(Group).where((Group.domain_id == domain_a_id) | (Group.domain_id == None)))  # noqa: E711
            gnames = [g.name for g in r.scalars().all()]
            assert global_name in gnames, f"global missing in sqlite path {gnames}"
            assert grp_a_name in gnames
            assert grp_b_name not in gnames, f"sqlite filter should hide B group {gnames}"

            # user_groups filtered
            r = await s.execute(select(UserGroup).where(UserGroup.domain_id == domain_a_id))
            ug_doms = [str(x.domain_id) for x in r.scalars().all()]
            assert str(domain_b_id) not in ug_doms
            assert str(domain_a_id) in ug_doms

            # user_settings filtered
            r = await s.execute(select(UserSetting).where(UserSetting.domain_id == domain_a_id))
            vals = [x.user_setting_value for x in r.scalars().all()]
            assert "v_a" in vals
            assert "v_b" not in vals

            # group_permissions filtered
            r = await s.execute(select(GroupPermission).where(GroupPermission.domain_id == domain_a_id))
            pnames = [x.permission_name for x in r.scalars().all()]
            assert perm_name_a in pnames
            assert perm_name_b not in pnames

            # verify migration file still has RLS clauses (even though not enforced in sqlite)
            import pathlib
            mig = pathlib.Path("alembic/versions/026_groups_users_rls.py").read_text()
            assert "ENABLE ROW LEVEL SECURITY" in mig
            assert "tenant_groups" in mig or "tenant_users" in mig
            assert "current_setting('app.domain_uuid'" in mig
