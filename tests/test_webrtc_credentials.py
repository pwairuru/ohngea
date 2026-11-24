import pytest

pytestmark = pytest.mark.asyncio


async def test_credentials_extension_mode_ok(client, auth_headers, domain, extension):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "extension", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    resp = await client.post("/api/v1/webrtc/credentials", json={"extension": extension.extension}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    data = resp.json()
    assert data["auth_user"] == extension.extension
    assert "password" in data
    assert data["sip_uri"].startswith("sip:")
    assert data["wss_url"] == "wss://pbx.example.com/wss"
    assert "expires_in" in data
    assert data["expires_in"] == 3600
    assert "display_name" in data


async def test_credentials_wrong_auth_mode_rejected(client, auth_headers, domain, extension):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "webrtc", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    resp = await client.post("/api/v1/webrtc/credentials", json={"extension": extension.extension}, headers=auth_headers)
    assert resp.status_code == 400, resp.text


async def test_credentials_nonexistent_extension_404(client, auth_headers, domain):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "extension", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    resp = await client.post("/api/v1/webrtc/credentials", json={"extension": "9999"}, headers=auth_headers)
    assert resp.status_code == 404, resp.text


async def test_credentials_cross_tenant_forbidden(client, auth_headers, other_auth_headers, domain, other_domain, extension):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "extension", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    # ensure other domain's webrtc is enabled so we hit tenant isolation not disabled check
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{other_domain.domain_id}/settings", json={"enabled": True, "auth_mode": "extension", "wss_url": "wss://pbx.example.com/wss"}, headers=other_auth_headers)
    assert resp.status_code == 200, resp.text
    resp = await client.post("/api/v1/webrtc/credentials", json={"extension": extension.extension}, headers=other_auth_headers)
    assert resp.status_code in (403, 404), resp.text


async def test_sip_profiles_wss_fields_persist(client, auth_headers, domain):
    resp = await client.post("/api/v1/sip_profiles", json={"name": "internal", "sip_port": 5060, "wss_port": 7443, "wss_binding": ":7443"}, headers=auth_headers)
    # if not implemented, will be 422 or missing fields
    assert resp.status_code == 201, resp.text
    data = resp.json()
    assert data.get("wss_port") == 7443 or "wss_port" in data
    # verify via GET
    pid = data.get("sip_profile_uuid")
    assert pid is not None
    resp2 = await client.get(f"/api/v1/sip_profiles/{pid}", headers=auth_headers)
    assert resp2.status_code == 200, resp2.text
    assert resp2.json().get("wss_port") == 7443
    assert resp2.json().get("wss_binding") == ":7443"


async def test_sip_profiles_wss_tls_fields(client, auth_headers, domain):
    resp = await client.post("/api/v1/sip_profiles", json={"name": "internal2", "sip_port": 5060, "wss_port": 7443, "wss_binding": ":7443", "tls_cert_path": "/etc/certs/cert.pem", "tls_key_path": "/etc/certs/key.pem"}, headers=auth_headers)
    assert resp.status_code == 201, resp.text
    data = resp.json()
    assert data.get("tls_cert_path") == "/etc/certs/cert.pem"
    assert data.get("tls_key_path") == "/etc/certs/key.pem"


async def test_credentials_webrtc_mode_ok(client, auth_headers, domain):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "webrtc", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    resp = await client.post("/api/v1/webrtc/credentials", json={"webrtc_user": "alice", "webrtc_pass": "s3cr3t"}, headers=auth_headers)
    assert resp.status_code == 200, resp.text
    data = resp.json()
    assert data["auth_user"] == "alice"
    assert data["password"] == "s3cr3t"
    assert data["sip_uri"].startswith("sip:")


async def test_credentials_both_mode_accepts_extension(client, auth_headers, domain, extension):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "both", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200
    resp = await client.post("/api/v1/webrtc/credentials", json={"extension": extension.extension}, headers=auth_headers)
    assert resp.status_code == 200


async def test_credentials_both_mode_accepts_webrtc_user(client, auth_headers, domain):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "both", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200
    resp = await client.post("/api/v1/webrtc/credentials", json={"webrtc_user": "bob", "webrtc_pass": "pw"}, headers=auth_headers)
    assert resp.status_code == 200


async def test_credentials_disabled_returns_400(client, auth_headers, domain, extension):
    resp = await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": False, "auth_mode": "extension", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    assert resp.status_code == 200
    resp = await client.post("/api/v1/webrtc/credentials", json={"extension": extension.extension}, headers=auth_headers)
    assert resp.status_code == 400


async def test_credentials_missing_payload_422(client, auth_headers, domain):
    await client.put(f"/api/v1/webrtc-settings/domains/{domain.domain_id}/settings", json={"enabled": True, "auth_mode": "extension", "wss_url": "wss://pbx.example.com/wss"}, headers=auth_headers)
    resp = await client.post("/api/v1/webrtc/credentials", json={}, headers=auth_headers)
    assert resp.status_code == 422, resp.text


async def test_sip_profiles_patch_wss_fields(client, auth_headers, domain):
    resp = await client.post("/api/v1/sip_profiles", json={"name": "patchtest", "sip_port": 5060}, headers=auth_headers)
    assert resp.status_code == 201, resp.text
    pid = resp.json()["sip_profile_uuid"]
    resp2 = await client.patch(f"/api/v1/sip_profiles/{pid}", json={"wss_port": 7443, "wss_binding": ":7443", "tls_cert_path": "/tmp/cert", "tls_key_path": "/tmp/key"}, headers=auth_headers)
    assert resp2.status_code == 200, resp2.text
    assert resp2.json().get("wss_port") == 7443
    assert resp2.json().get("wss_binding") == ":7443"
    assert resp2.json().get("tls_cert_path") == "/tmp/cert"
    assert resp2.json().get("tls_key_path") == "/tmp/key"
