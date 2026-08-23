import pytest


def test_domain_app_model_exists():
    from app.domains.models import DomainApp
    assert hasattr(DomainApp, "app_name")
    assert hasattr(DomainApp, "enabled")


@pytest.mark.asyncio
async def test_list_domain_apps_empty_by_default(client, domain, auth_headers):
    resp = await client.get(f"/api/v1/domains/{domain.domain_id}/apps", headers=auth_headers)
    assert resp.status_code == 200
    assert "data" in resp.json()


@pytest.mark.asyncio
async def test_toggle_app(client, domain, auth_headers):
    resp = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/fifo?enabled=false", headers=auth_headers)
    assert resp.status_code == 200
    assert resp.json()["enabled"] is False
    # list reflects
    resp2 = await client.get(f"/api/v1/domains/{domain.domain_id}/apps", headers=auth_headers)
    assert resp2.status_code == 200
    fifo = [x for x in resp2.json()["data"] if x["app_name"] == "fifo"][0]
    assert fifo["enabled"] is False


@pytest.mark.asyncio
async def test_toggle_unknown_app_404(client, domain, auth_headers):
    resp = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/does_not_exist?enabled=false", headers=auth_headers)
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_disabled_app_returns_403(client, domain, auth_headers):
    resp = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/fifo?enabled=false", headers=auth_headers)
    assert resp.status_code == 200
    resp = await client.get("/api/v1/fifo", headers=auth_headers)
    assert resp.status_code == 403
    assert "disabled" in resp.json()["detail"].lower()


@pytest.mark.asyncio
async def test_enabled_app_still_200(client, domain, auth_headers):
    await client.put(f"/api/v1/domains/{domain.domain_id}/apps/fifo?enabled=true", headers=auth_headers)
    resp = await client.get("/api/v1/fifo", headers=auth_headers)
    assert resp.status_code == 200


@pytest.mark.asyncio
async def test_ivr_menus_gated_by_middleware(client, domain, auth_headers):
    # ivr_menus enabled_by_default=true, so 200 initially
    resp = await client.get("/api/v1/ivr_menus", headers=auth_headers)
    assert resp.status_code == 200, resp.text
    # disable via central gate
    r = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/ivr_menus?enabled=false", headers=auth_headers)
    assert r.status_code == 200, r.text
    resp = await client.get("/api/v1/ivr_menus", headers=auth_headers)
    assert resp.status_code == 403, f"expected 403 got {resp.status_code} {resp.text}"
    assert "disabled" in resp.json()["detail"].lower()
    # re-enable
    r = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/ivr_menus?enabled=true", headers=auth_headers)
    assert r.status_code == 200
    resp = await client.get("/api/v1/ivr_menus", headers=auth_headers)
    assert resp.status_code == 200


@pytest.mark.asyncio
async def test_conferences_gated_by_middleware(client, domain, auth_headers):
    # conferences enabled_by_default=false, so need to enable first to get 200
    r = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/conferences?enabled=true", headers=auth_headers)
    assert r.status_code == 200, r.text
    resp = await client.get("/api/v1/conferences", headers=auth_headers)
    assert resp.status_code == 200, resp.text
    # disable
    r = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/conferences?enabled=false", headers=auth_headers)
    assert r.status_code == 200, r.text
    resp = await client.get("/api/v1/conferences", headers=auth_headers)
    assert resp.status_code == 403, f"expected 403 got {resp.status_code} {resp.text}"
    assert "disabled" in resp.json()["detail"].lower()
    r = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/conferences?enabled=true", headers=auth_headers)
    assert r.status_code == 200
    resp = await client.get("/api/v1/conferences", headers=auth_headers)
    assert resp.status_code == 200


@pytest.mark.asyncio
async def test_always_enabled_not_gated(client, domain, auth_headers):
    # 6 primary nav always enabled — even if DomainApp says disabled, middleware skips gate
    for app_name, endpoint in [
        ("extensions", "/api/v1/extensions"),
        ("dialplans", "/api/v1/dialplans"),
        ("gateways", "/api/v1/gateways"),
        ("devices", "/api/v1/devices"),
        ("cdr", "/api/v1/cdr"),
        ("domains", "/api/v1/domains"),
    ]:
        r = await client.put(f"/api/v1/domains/{domain.domain_id}/apps/{app_name}?enabled=false", headers=auth_headers)
        # some primary apps may have toggle allowed
        assert r.status_code in (200, 409), r.text
        # if toggle succeeded to false, still should be 200 via middleware skip
        if r.status_code == 200 and r.json().get("enabled") is False:
            resp = await client.get(endpoint, headers=auth_headers)
            assert resp.status_code == 200, f"{app_name} should stay 200 despite disabled override but got {resp.status_code} {resp.text}"
            # restore
            await client.put(f"/api/v1/domains/{domain.domain_id}/apps/{app_name}?enabled=true", headers=auth_headers)
