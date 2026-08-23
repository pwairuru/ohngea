import pytest
from uuid import uuid4

pytestmark = pytest.mark.asyncio

async def test_user_patch_updates_contact_org(client, auth_headers):
    """Task 6 Step 1: create user with contact, PATCH nested contact org, verify via GET."""
    uname = f"ct-{uuid4().hex[:4]}"
    resp = await client.post("/api/v1/users", json={
        "username": uname,
        "password": "Password123!",
        "email": f"{uname}@example.com",
        "contact_organization": "OldOrg",
        "contact_name_given": "John",
        "contact_name_family": "Doe"
    }, headers=auth_headers)
    assert resp.status_code == 201, resp.text
    uid = resp.json()["user_uuid"]
    assert resp.json()["contact_organization"] == "OldOrg"
    # PATCH with nested contact dict
    resp2 = await client.patch(f"/api/v1/users/{uid}", json={"contact": {"organization": "NewOrg"}}, headers=auth_headers)
    assert resp2.status_code == 200, resp2.text
    # GET should reflect NewOrg
    resp3 = await client.get(f"/api/v1/users/{uid}", headers=auth_headers)
    assert resp3.status_code == 200
    assert resp3.json()["contact_organization"] == "NewOrg", f"expected NewOrg got {resp3.json()}"
    # also verify direct contact endpoint reflects change
    r4 = await client.get("/api/v1/contacts", headers=auth_headers)
    assert r4.status_code == 200
    assert any(c["organization"] == "NewOrg" for c in r4.json()["data"])

async def test_contacts_crud_and_domain_filter(client, auth_headers, other_auth_headers):
    """Verify contacts minimal CRUD with set_tenant + domain filter and RLS."""
    # create contact via contacts endpoint
    name = f"Contact-{uuid4().hex[:4]}"
    r = await client.post("/api/v1/contacts", json={"name": name, "number": "1001", "email": f"{name}@example.com", "organization": "TestOrg"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    cid = r.json()["contact_uuid"]
    # get contact
    r2 = await client.get(f"/api/v1/contacts/{cid}", headers=auth_headers)
    assert r2.status_code == 200
    assert r2.json()["name"] == name
    # patch contact
    r3 = await client.patch(f"/api/v1/contacts/{cid}", json={"organization": "UpdatedOrg"}, headers=auth_headers)
    assert r3.status_code == 200
    assert r3.json()["organization"] == "UpdatedOrg"
    # list should show updated
    r4 = await client.get("/api/v1/contacts", headers=auth_headers)
    assert r4.status_code == 200
    assert any(c["contact_uuid"] == cid and c["organization"] == "UpdatedOrg" for c in r4.json()["data"])
    # other domain should not see it (RLS / domain filter)
    r5 = await client.get(f"/api/v1/contacts", headers=other_auth_headers)
    assert r5.status_code == 200
    assert not any(c["contact_uuid"] == cid for c in r5.json()["data"])
    # cleanup
    r6 = await client.delete(f"/api/v1/contacts/{cid}", headers=auth_headers)
    assert r6.status_code == 204

async def test_user_create_links_contact_and_patch_creates_if_missing(client, auth_headers):
    """Create user without contact, then PATCH contact fields should create new Contact."""
    uname = f"nocont-{uuid4().hex[:4]}"
    resp = await client.post("/api/v1/users", json={
        "username": uname,
        "password": "Password123!",
        "email": f"{uname}@example.com"
    }, headers=auth_headers)
    assert resp.status_code == 201, resp.text
    uid = resp.json()["user_uuid"]
    assert resp.json()["contact_uuid"] is None
    # patch contact organization via flat field
    resp2 = await client.patch(f"/api/v1/users/{uid}", json={"contact_organization": "BrandNewOrg"}, headers=auth_headers)
    assert resp2.status_code == 200
    assert resp2.json()["contact_organization"] == "BrandNewOrg"
    # second patch via nested email
    resp3 = await client.patch(f"/api/v1/users/{uid}", json={"contact": {"email": "newemail@example.com"}}, headers=auth_headers)
    assert resp3.status_code == 200
    # GET verify
    resp4 = await client.get(f"/api/v1/users/{uid}", headers=auth_headers)
    assert resp4.json()["contact_organization"] == "BrandNewOrg"

async def test_users_import_csv(client, auth_headers):
    """POST /users/import CSV path (legacy user_imports.php:31)."""
    csv_content = "username,password,email\nimport1-{uuid},Password123!,import1@example.com\nimport2-{uuid},Password123!,import2@example.com".format(uuid=uuid4().hex[:4], uuid2=uuid4().hex[:4])
    # use deterministic names
    u1 = f"imp-{uuid4().hex[:4]}"
    u2 = f"imp-{uuid4().hex[:4]}"
    csv_content = f"username,password,email\n{u1},Password123!,{u1}@example.com\n{u2},Password123!,{u2}@example.com"
    resp = await client.post("/api/v1/users/import", json={"csv": csv_content}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    assert resp.json()["imported"] == 2
    assert len(resp.json()["data"]) == 2
    # verify users exist via search
    for uname in [u1, u2]:
        r = await client.get(f"/api/v1/users?search={uname}", headers=auth_headers)
        assert r.status_code == 200
        assert any(u["username"] == uname for u in r.json()["data"])

async def test_users_import_requires_perm(client, auth_headers_user):
    """Import should require user_add perm -> 403 for low user."""
    csv_content = "username,password,email\nshouldfail,Password123!,fail@example.com"
    resp = await client.post("/api/v1/users/import", json={"csv": csv_content}, headers=auth_headers_user)
    assert resp.status_code == 403
