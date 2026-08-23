import pytest
from app.call_forward.models import CallForward


@pytest.mark.anyio
async def test_call_forward_crud_and_rls(client, auth_headers, other_auth_headers, domain, other_domain):
    # create
    r = await client.post("/api/v1/call_forward", json={"forward_all_destination": "2000", "forward_all_enabled": True, "description": "test"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    cf_uuid = r.json()["call_forward_uuid"]
    assert r.json()["forward_all_destination"] == "2000"
    assert r.json()["forward_all_enabled"] is True
    assert r.headers["Location"].endswith(cf_uuid)

    # list
    r = await client.get("/api/v1/call_forward", headers=auth_headers)
    assert r.status_code == 200
    assert r.json()["total"] == 1

    # get
    r = await client.get(f"/api/v1/call_forward/{cf_uuid}", headers=auth_headers)
    assert r.status_code == 200
    assert r.json()["forward_all_destination"] == "2000"

    # RLS isolation - other domain can't see
    r = await client.get(f"/api/v1/call_forward/{cf_uuid}", headers=other_auth_headers)
    assert r.status_code == 404

    r = await client.get("/api/v1/call_forward", headers=other_auth_headers)
    assert r.json()["total"] == 0

    # patch
    r = await client.patch(f"/api/v1/call_forward/{cf_uuid}", json={"forward_busy_destination": "2001", "forward_busy_enabled": True}, headers=auth_headers)
    assert r.status_code == 200
    assert r.json()["forward_busy_destination"] == "2001"

    # alias forward_destination maps to forward_all
    r = await client.post("/api/v1/call_forward", json={"forward_destination": "3000", "forward_enabled": True}, headers=auth_headers)
    assert r.status_code == 201
    assert r.json()["forward_all_destination"] == "3000"
    cf2 = r.json()["call_forward_uuid"]

    # delete
    r = await client.delete(f"/api/v1/call_forward/{cf_uuid}", headers=auth_headers)
    assert r.status_code == 204
    r = await client.get(f"/api/v1/call_forward/{cf_uuid}", headers=auth_headers)
    assert r.status_code == 404

    # pagination headers
    r = await client.get("/api/v1/call_forward", headers=auth_headers)
    assert "X-Total-Count" in r.headers


@pytest.mark.anyio
async def test_call_forward_domain_mismatch(client, auth_headers, domain):
    import uuid
    fake = str(uuid.uuid4())
    r = await client.post("/api/v1/call_forward", json={"domain_id": fake, "forward_all_destination": "2000"}, headers=auth_headers)
    assert r.status_code == 403


@pytest.mark.anyio
async def test_call_forward_requires_auth(client):
    r = await client.post("/api/v1/call_forward", json={"forward_all_destination": "2000"})
    assert r.status_code in (401, 403)


@pytest.mark.anyio
async def test_call_forward_with_extension_uuid(client, auth_headers, domain, test_db):
    from app.extensions.models import Extension
    async with test_db() as session:
        ext = Extension(extension="1009", effective_caller_id_name="Test", domain_id=domain.domain_id, password="secret123", enabled=True)
        session.add(ext)
        await session.commit()
        await session.refresh(ext)
        ext_uuid = str(ext.extension_uuid)

    r = await client.post("/api/v1/call_forward", json={"extension_uuid": ext_uuid, "forward_all_destination": "4000", "forward_all_enabled": True}, headers=auth_headers)
    assert r.status_code == 201, r.text
    assert r.json()["extension_uuid"] == ext_uuid
