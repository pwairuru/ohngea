import pytest
from uuid import UUID, uuid4
from sqlalchemy import select, func

pytestmark = pytest.mark.asyncio


async def test_create_domain_seeds_superadmin_level_80(client, auth_headers, test_db):
    from app.auth.models import Group, Permission, GroupPermission
    from app.domains.models import DomainSettings

    domain_name = f"seedtest-{uuid4().hex[:8]}.example.com"
    resp = await client.post(
        "/api/v1/domains",
        json={"domain_name": domain_name, "description": "seed test", "enabled": True},
        headers=auth_headers,
    )
    assert resp.status_code == 201, resp.text
    data = resp.json()
    domain_id = data["domain_id"]
    domain_uuid = UUID(domain_id)

    # verify via DB: 6 default groups with correct levels
    async with test_db() as s:
        r = await s.execute(select(Group).where(Group.domain_id == domain_uuid))
        groups = r.scalars().all()
        by_name = {g.name: g for g in groups}
        expected = {
            "superadmin": (80, True),
            "admin": (50, True),
            "user": (30, False),
            "agent": (20, False),
            "public": (10, False),
            "gateway": (10, False),
        }
        for name, (lvl, prot) in expected.items():
            assert name in by_name, f"missing group {name} groups found {[g.name for g in groups]}"
            g = by_name[name]
            assert g.group_level == lvl, f"{name} level {g.group_level} != {lvl}"
            assert g.group_protected == prot, f"{name} protected {g.group_protected} != {prot}"

        superadmin = by_name["superadmin"]
        assert superadmin.group_level == 80

        # permissions count check
        perm_count = (await s.execute(select(func.count()).select_from(Permission))).scalar_one()
        assert perm_count > 0, "permissions table empty - seeding failed"
        gp_count = (
            await s.execute(select(func.count()).select_from(GroupPermission).where(GroupPermission.group_uuid == superadmin.group_uuid))
        ).scalar_one()
        assert gp_count == perm_count, f"superadmin perms {gp_count} != total perms {perm_count}"

        # denorm fields check
        r2 = await s.execute(select(GroupPermission).where(GroupPermission.group_uuid == superadmin.group_uuid).limit(5))
        for gp in r2.scalars().all():
            assert str(gp.domain_id) == str(domain_id), f"gp domain_id {gp.domain_id} != {domain_id}"
            assert gp.group_name == "superadmin"
            assert gp.permission_name is not None
            assert gp.permission_assigned is True
            assert gp.permission_protected is False

        # domain_settings seeded (at least theme)
        r3 = await s.execute(select(DomainSettings).where(DomainSettings.domain_id == domain_uuid))
        settings = r3.scalars().all()
        assert len(settings) >= 3, f"expected >=3 domain_settings, got {len(settings)}"
        keys = {(s.category, s.key) for s in settings}
        # theme seeds expected
        assert ("theme", "primary_color") in keys or any(k[1] == "primary_color" for k in keys)

    # verify via API: GET /groups?show=all -> superadmin visible with level 80
    r = await client.get("/api/v1/groups?show=all&per_page=100", headers=auth_headers)
    assert r.status_code == 200, r.text
    found = [g for g in r.json()["data"] if g["name"] == "superadmin" and g["domain_id"] == domain_id]
    assert found, f"superadmin not found via API data {[ (x['name'], x['domain_id']) for x in r.json()['data'] ]}"
    assert found[0]["group_level"] == 80, f"API superadmin level {found[0]['group_level']} !=80"
    assert found[0]["group_protected"] is True

    # GET /groups/{superadmin}/permissions count == permissions count
    superadmin_uuid = str(superadmin.group_uuid)
    r2 = await client.get(f"/api/v1/groups/{superadmin_uuid}/permissions", headers=auth_headers)
    assert r2.status_code == 200, r2.text
    perms_data = r2.json()["data"]
    # count assigned via API should equal perm_count
    # API returns all perms with assigned flag
    assigned = [p for p in perms_data if p["assigned"]]
    # alternative: total length if view not filtered
    async with test_db() as s:
        perm_count2 = (await s.execute(select(func.count()).select_from(Permission))).scalar_one()
    # API should return perm_count total entries, assigned == perm_count
    assert len(assigned) == perm_count2, f"API assigned {len(assigned)} != {perm_count2}"
    # also check API returns per_page total matches
    assert r2.json()["total"] == perm_count2
