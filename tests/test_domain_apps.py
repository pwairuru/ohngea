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
