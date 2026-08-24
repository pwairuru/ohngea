import pytest
from pydantic import ValidationError
from app.extensions.models import ExtensionBase
from app.extensions.schemas import ExtensionUpdate
from uuid import uuid4


def test_extension_valid():
    e = ExtensionBase(extension="1001", effective_caller_id_name="Alice", domain_id=uuid4())
    assert e.extension == "1001"


def test_extension_invalid_short():
    with pytest.raises(ValidationError):
        ExtensionBase(extension="1", effective_caller_id_name="Alice", domain_id=uuid4())


def test_extension_invalid_chars():
    with pytest.raises(ValidationError):
        ExtensionBase(extension="abc", effective_caller_id_name="Alice", domain_id=uuid4())


def test_extension_enabled_default():
    e = ExtensionBase(extension="1001", effective_caller_id_name="Bob", domain_id=uuid4())
    assert e.enabled is True


def test_extension_update_accepts_password():
    """PATCH must allow setting the SIP password (legacy PBX parity)."""
    u = ExtensionUpdate(password="1234")
    assert u.password == "1234"


@pytest.mark.anyio
async def test_extensions_list_requires_perm(client, test_db, domain):
    """Task 8: list/get require extension_view — no perm 403, with perm 200."""
    from tests.conftest import _make_user
    from uuid import uuid4
    # user with no extension_view -> should get 403 on list
    _, token_no_perm = await _make_user(test_db, domain, f"noperm-{uuid4().hex[:4]}", [])
    r = await client.get("/api/v1/extensions", headers={"Authorization": f"Bearer {token_no_perm}"})
    assert r.status_code == 403, f"expected 403 without perm, got {r.status_code}: {r.text}"
    # also GET by id should 403 — create an extension as superadmin then try fetch
    # create extension via superadmin token (reuse fixture via _make_user with *)
    _, token_admin = await _make_user(test_db, domain, f"adm-{uuid4().hex[:4]}", ["*"])
    # extension may already exist; create one for get test
    from app.extensions.models import Extension
    ext_uuid = None
    async with test_db() as s:
        from sqlalchemy import select as sel
        res = await s.execute(sel(Extension).where(Extension.domain_id == domain.domain_id).limit(1))
        row = res.scalar_one_or_none()
        if row:
            ext_uuid = row.extension_uuid
    if ext_uuid is None:
        # create via API as admin
        resp = await client.post(
            "/api/v1/extensions",
            json={"extension": "1999", "effective_caller_id_name": "Perm Test", "password": "1234"},
            headers={"Authorization": f"Bearer {token_admin}"},
        )
        assert resp.status_code in (201, 409), resp.text
        if resp.status_code == 201:
            ext_uuid = resp.json().get("extension_uuid") or resp.json().get("extension_uuid")
        else:
            # fetch again
            async with test_db() as s2:
                from sqlalchemy import select as sel2

                res2 = await s2.execute(sel2(Extension).where(Extension.domain_id == domain.domain_id).limit(1))
                ext_uuid = res2.scalar_one().extension_uuid
    # no perm user should also be 403 on get
    r2 = await client.get(f"/api/v1/extensions/{ext_uuid}", headers={"Authorization": f"Bearer {token_no_perm}"})
    assert r2.status_code == 403, f"expected 403 on get without perm, got {r2.status_code}: {r2.text}"
    # user with extension_view -> 200
    _, token_view = await _make_user(test_db, domain, f"view-{uuid4().hex[:4]}", ["extension_view"])
    r3 = await client.get("/api/v1/extensions", headers={"Authorization": f"Bearer {token_view}"})
    assert r3.status_code == 200, f"expected 200 with perm, got {r3.status_code}: {r3.text}"
    r4 = await client.get(f"/api/v1/extensions/{ext_uuid}", headers={"Authorization": f"Bearer {token_view}"})
    assert r4.status_code == 200, f"expected 200 on get with perm, got {r4.status_code}: {r4.text}"
    # wildcard * also passes
    r5 = await client.get("/api/v1/extensions", headers={"Authorization": f"Bearer {token_admin}"})
    assert r5.status_code == 200
