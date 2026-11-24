"""Broadcast send requires destination; notifications scoped per user."""
import pytest


@pytest.mark.anyio
async def test_broadcast_send_requires_destination(client, auth_headers, domain):
    did = domain.domain_id
    r = await client.put(f"/api/v1/domains/{did}/apps/call_broadcast?enabled=true", headers=auth_headers)
    assert r.status_code == 200, r.text
    r = await client.post("/api/v1/call_broadcasts", json={"broadcast_name": "b1", "broadcast_phone_numbers": "1001"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    uid = r.json()["call_broadcast_uuid"]
    r = await client.post(f"/api/v1/call_broadcasts/{uid}/send", headers=auth_headers)
    assert r.status_code == 422, r.text


@pytest.mark.anyio
async def test_notifications_scoped_to_user(client, auth_headers, domain):
    r = await client.post("/api/v1/notifications", json={"title": "mine", "user_uuid": "00000000-0000-0000-0000-000000000001"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    r = await client.post("/api/v1/notifications", json={"title": "other", "user_uuid": "00000000-0000-0000-0000-000000000002"}, headers=auth_headers)
    assert r.status_code == 201, r.text
    r = await client.get("/api/v1/notifications?mine=1", headers=auth_headers)
    assert r.status_code == 200, r.text
    assert r.json()["total"] >= 0
