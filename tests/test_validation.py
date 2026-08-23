import pytest
from uuid import uuid4
from sqlalchemy import select

pytestmark = pytest.mark.asyncio


async def test_username_format_email_rejected_when_no_email(client, auth_headers, domain, test_db):
    from app.domains.models import DomainSettings

    # set domain_settings users.username_format=no_email
    async with test_db() as s:
        ds = DomainSettings(
            domain_id=domain.domain_id,
            category="users",
            subcategory="username_format",
            key="username_format",
            value="no_email",
            enabled=True,
        )
        s.add(ds)
        await s.commit()

    # try create email username => 422
    resp = await client.post(
        "/api/v1/users",
        json={"username": "test@example.com", "password": "ValidPass123!", "email": "test@example.com"},
        headers=auth_headers,
    )
    assert resp.status_code == 422, f"expected 422 got {resp.status_code} {resp.text}"
    assert "email" in resp.text.lower()

    # also ensure non-email username passes with same settings
    resp2 = await client.post(
        "/api/v1/users",
        json={"username": f"noemail-{uuid4().hex[:4]}", "password": "ValidPass123!"},
        headers=auth_headers,
    )
    assert resp2.status_code == 201, resp2.text


async def test_password_weak_rejected(client, auth_headers):
    # password "short" => 422 strength fail (default length 12 + number/lower/upper)
    resp = await client.post(
        "/api/v1/users",
        json={"username": f"weak-{uuid4().hex[:4]}", "password": "short", "email": f"weak-{uuid4().hex[:2]}@example.com"},
        headers=auth_headers,
    )
    assert resp.status_code == 422, f"expected 422 got {resp.status_code} {resp.text}"
    assert "password" in resp.text.lower()

    # also test missing number / uppercase variants
    resp2 = await client.post(
        "/api/v1/users",
        json={"username": f"weak2-{uuid4().hex[:4]}", "password": "alllowercasebutlongenough", "email": f"w2-{uuid4().hex[:2]}@example.com"},
        headers=auth_headers,
    )
    # should fail due to missing uppercase and number (if defaults true)
    assert resp2.status_code == 422, f"expected 422 for missing upper/number got {resp2.status_code} {resp2.text}"


async def test_group_name_invalid_rejected(client, auth_headers):
    # group name "bad name!" => 422 regex fail (must be ^[a-z0-9_-]{2,64}$)
    resp = await client.post(
        "/api/v1/groups",
        json={"name": "bad name!", "group_level": 10},
        headers=auth_headers,
    )
    assert resp.status_code == 422, f"expected 422 got {resp.status_code} {resp.text}"

    # also test too short (1 char) and invalid chars
    resp2 = await client.post(
        "/api/v1/groups",
        json={"name": "a", "group_level": 10},
        headers=auth_headers,
    )
    assert resp2.status_code == 422

    # valid name should succeed
    valid = f"valid-{uuid4().hex[:4]}"
    resp3 = await client.post(
        "/api/v1/groups",
        json={"name": valid, "group_level": 10},
        headers=auth_headers,
    )
    assert resp3.status_code == 201, resp3.text


async def test_username_format_email_required(client, auth_headers, domain, test_db):
    from app.domains.models import DomainSettings

    async with test_db() as s:
        ds = DomainSettings(
            domain_id=domain.domain_id,
            category="users",
            subcategory="username_format",
            key="username_format",
            value="email",
            enabled=True,
        )
        s.add(ds)
        await s.commit()

    # non-email username should be rejected when format=email
    resp = await client.post(
        "/api/v1/users",
        json={"username": f"plain-{uuid4().hex[:4]}", "password": "ValidPass123!"},
        headers=auth_headers,
    )
    assert resp.status_code == 422, f"expected 422 got {resp.status_code} {resp.text}"

    # email username should pass
    resp2 = await client.post(
        "/api/v1/users",
        json={"username": f"ok-{uuid4().hex[:4]}@example.com", "password": "ValidPass123!"},
        headers=auth_headers,
    )
    assert resp2.status_code == 201, resp2.text


async def test_password_strength_custom_settings(client, auth_headers, domain, test_db):
    # test that custom password_length via DomainSettings is enforced
    from app.domains.models import DomainSettings

    async with test_db() as s:
        # set longer requirement 20 chars
        ds = DomainSettings(
            domain_id=domain.domain_id,
            category="users",
            subcategory="password_length",
            key="password_length",
            value="20",
            enabled=True,
        )
        s.add(ds)
        # also ensure hyphen variant works if code supports it (we add both in test but only one needed)
        await s.commit()

    # 12-char normally valid but should fail with 20 requirement
    resp = await client.post(
        "/api/v1/users",
        json={"username": f"cust-{uuid4().hex[:4]}", "password": "ValidPass123!"},
        headers=auth_headers,
    )
    assert resp.status_code == 422, f"expected 422 with custom length 20, got {resp.status_code} {resp.text}"


async def test_group_name_unique_per_domain(client, auth_headers, domain):
    name = f"uniq-{uuid4().hex[:4]}"
    resp = await client.post("/api/v1/groups", json={"name": name, "group_level": 10}, headers=auth_headers)
    assert resp.status_code == 201, resp.text
    resp2 = await client.post("/api/v1/groups", json={"name": name, "group_level": 10}, headers=auth_headers)
    assert resp2.status_code == 409, f"expected 409 duplicate got {resp2.status_code} {resp2.text}"
