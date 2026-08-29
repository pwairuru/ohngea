"""TDD RED — outstanding parity gaps: failing tests before fix."""
import pytest
from httpx import ASGITransport, AsyncClient
from app.main import app
from app.seed_provision_defaults import PROVISION_DEFAULTS

@pytest.mark.asyncio
async def test_openapi_has_call_recordings_router():
    openapi = app.openapi()
    assert "/api/v1/call_recordings" in openapi["paths"], "call_recordings router not registered in app/main.py"

@pytest.mark.asyncio
async def test_openapi_has_conference_rooms_sessions():
    openapi = app.openapi()
    # conference_centers rooms/sessions child resources
    assert any("/conference_centers" in p and "rooms" in p for p in openapi["paths"]), "conference rooms sessions not ported"

@pytest.mark.asyncio
async def test_seed_has_algo_avaya():
    cats = [r["default_setting_subcategory"] for r in PROVISION_DEFAULTS]
    # algo/avaya provision defaults must be seeded
    assert any("algo" in c for c in cats), f"algo missing in PROVISION_DEFAULTS: {set(cats)}"
    assert any("avaya" in c for c in cats), f"avaya missing: {set(cats)}"
    assert any("aastra" in c for c in cats), "aastra missing"

@pytest.mark.asyncio
async def test_perm_bugs_fixed_pin_ring_sip():
    # perm strings must not use wrong add perms
    import pathlib
    pin = pathlib.Path("app/pin_numbers/router.py").read_text()
    assert 'pin_number_edit' in pin, "pin_numbers PATCH should require pin_number_edit not add"
    assert 'pin_number_delete' in pin, "pin_numbers DELETE should require delete"
    ring = pathlib.Path("app/ring_groups/router.py").read_text()
    assert 'ring_group_edit' in ring, "ring_groups PATCH should require ring_group_edit"
    sip = pathlib.Path("app/sip_profiles/router.py").read_text()
    assert 'sip_profile_add' in sip or 'sip_profile_edit' in sip, "sip_profiles should use sip_profile_* not gateway_view"

@pytest.mark.asyncio
async def test_contacts_child_crud_and_base_fields():
    openapi = app.openapi()
    assert "/api/v1/contacts/{contact_uuid}/addresses" in openapi["paths"] or "/api/v1/contacts/{contact_uuid}/phones" in openapi["paths"], "contacts child CRUD missing"
    import pathlib
    models = pathlib.Path("app/contacts/models.py").read_text()
    assert "contact_type" in models, "contacts base missing legacy contact_type/given/middle fields"

@pytest.mark.asyncio
async def test_number_translations_child_router():
    openapi = app.openapi()
    assert "/api/v1/number_translations/{number_translation_uuid}/details" in openapi["paths"], "number_translations details child router missing"

@pytest.mark.asyncio
async def test_cdr_has_aux_tables_and_search():
    import pathlib
    models = pathlib.Path("app/cdr/models.py").read_text()
    assert "hold_accum" in models, "cdr missing hold_accum added in 039"
    # aux tables
    cdr_tables = pathlib.Path("alembic/versions/039_voicemail_cdr_fix.py").read_text()
    assert "v_xml_cdr" in cdr_tables, "xml_cdr view not"

@pytest.mark.asyncio
async def test_groups_frontend_exists():
    import pathlib, os
    assert pathlib.Path("frontend/src/routes/Groups.tsx").exists(), "Groups.tsx missing frontend"

@pytest.mark.asyncio
async def test_fifo_members_and_ivr_whitelist():
    import pathlib
    openapi = app.openapi()
    assert "/api/v1/fifo/{fifo_uuid}/members" in openapi["paths"] or "/api/v1/fifo_list" in openapi["paths"], "fifo members CRUD missing"
    ivr = pathlib.Path("app/ivr_menus/router.py").read_text()
    # whitelist bug: patch should allow all columns not just 3
    assert "dialplan_uuid" in ivr or "greet_long" in ivr, "ivr patch whitelist thin still"

@pytest.mark.asyncio
async def test_cisco_provision_render_and_time_presets():
    from app.devices.provision import render_provision
    # cisco spa 13 provision fields should render non-empty
    out = render_provision("cisco", {"mac":"00:11:22:33:44:55","domain":"example.com","extension":"1001","password":"secret"})
    assert len(out) > 100, f"cisco render thin {out[:50]}"
    import pathlib
    time_presets = pathlib.Path("alembic/versions/039_voicemail_cdr_fix.py").read_text()
    assert "preset_usa" in time_presets, "time_conditions presets not seeded"
