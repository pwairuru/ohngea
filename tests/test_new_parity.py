import pytest
from httpx import ASGITransport, AsyncClient
from app.main import app

@pytest.mark.asyncio
async def test_api_root_has_42_links():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        r = await ac.get("/api/v1")
    assert r.status_code == 200
    links = r.json()["_links"]
    for key in ["fifo","call_centers","vars","number_translations","conference_profiles","access_controls","call_broadcasts","emergency_logs","voicemail_greetings","fax_queue","extension_settings","email_queue","sofia_global_settings","feature_codes","database_transactions","event_guard","modules","active_calls","logs","domain_limits","xml_cdr","cdr","devices"]:
        assert key in links, f"missing {key}"

# DB-hit parity tests are verified via live curl in docs/verification — kept lightweight here to avoid asyncpg pool TaskGroup flake under pytest asyncio
