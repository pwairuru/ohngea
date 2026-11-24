import pytest
from uuid import uuid4

pytestmark = pytest.mark.asyncio


async def test_get_webrtc_settings_returns_disabled_by_default(client, auth_headers, domain):
    resp = await client.get(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", headers=auth_headers)
    assert resp.status_code == 200
    data = resp.json()
    assert data["enabled"] is False
    assert data["auth_mode"] == "extension"


async def test_put_webrtc_settings_requires_admin(client, auth_headers_user, domain):
    resp = await client.put(
        f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings",
        json={"enabled": True, "wss_url": "wss://example.com:7443", "auth_mode": "both"},
        headers=auth_headers_user,
    )
    assert resp.status_code == 403


async def test_put_webrtc_settings_admin_ok(client, auth_headers, domain):
    resp = await client.put(
        f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings",
        json={"enabled": True, "wss_url": "wss://example.com:7443", "auth_mode": "both", "sip_domain": "example.com"},
        headers=auth_headers,
    )
    assert resp.status_code == 200
    assert resp.json()["wss_url"] == "wss://example.com:7443"
    assert resp.json()["auth_mode"] == "both"
    # get reflects
    resp2 = await client.get(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", headers=auth_headers)
    assert resp2.json()["enabled"] is True


async def test_put_validates_wss_url(client, auth_headers, domain):
    resp = await client.put(
        f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings",
        json={"wss_url": "http://bad"},
        headers=auth_headers,
    )
    assert resp.status_code == 422


async def test_cross_tenant_isolation(client, auth_headers, domain, other_domain, other_auth_headers):
    await client.put(
        f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings",
        json={"enabled": True, "wss_url": "wss://a.example.com"},
        headers=auth_headers,
    )
    resp = await client.get(f"/api/v1/webrtc-settings/domains/{other_domain.domain_id}/settings", headers=other_auth_headers)
    assert resp.json()["enabled"] is False  # not leaked


async def test_invalid_auth_mode_rejected(client, auth_headers, domain):
    resp = await client.put(
        f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings",
        json={"auth_mode": "invalid"},
        headers=auth_headers,
    )
    assert resp.status_code == 422
