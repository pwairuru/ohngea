"""Seed parity — fresh installs must seed working PBX out of box."""


def test_global_default_settings_count():
    from app.seeds.default_settings import DEFAULT_SETTINGS

    cats = {r["category"] for r in DEFAULT_SETTINGS}
    assert len(DEFAULT_SETTINGS) >= 1200, f"only {len(DEFAULT_SETTINGS)} defaults"
    assert {"domain", "email", "extension", "fax", "voicemail", "cdr", "provision"} <= cats


def test_vars_sofia_perms_fixtures():
    from app.seeds.permissions import ALL_PERMISSIONS
    from app.seeds.sofia import SOFIA_GLOBAL_SETTINGS
    from app.seeds.vars import VARS

    assert len(VARS) >= 60, f"only {len(VARS)} vars"
    assert {v["var_name"] for v in VARS} >= {
        "global_codec_prefs",
        "default_language",
        "default_country",
    }
    assert len(SOFIA_GLOBAL_SETTINGS) == 6
    assert len(ALL_PERMISSIONS) >= 300
    assert "var_view" in ALL_PERMISSIONS and "sofia_global_setting_view" in ALL_PERMISSIONS


def test_domain_seeds_cover_working_pbx():
    from app.seeds.dialplans import DEFAULT_DIALPLANS
    from app.seeds.domain_settings import DOMAIN_DEFAULTS
    from app.seeds.operational import ACCESS_CONTROLS, CONFERENCE_PROFILES, MOH_ENTRIES
    from app.seeds.sip_profiles import SIP_PROFILES

    assert len(SIP_PROFILES) >= 4
    assert sum(len(p["settings"]) for p in SIP_PROFILES) >= 200
    assert len(CONFERENCE_PROFILES) >= 5
    assert len(MOH_ENTRIES) >= 4
    assert len(ACCESS_CONTROLS) >= 2
    assert any(d["name"] == "local_extension" for d in DEFAULT_DIALPLANS)
    assert len(DOMAIN_DEFAULTS) >= 12


from uuid import UUID, uuid4

import pytest
from sqlalchemy import func, select

pytestmark = pytest.mark.asyncio


async def test_create_domain_seeds_working_pbx(client, auth_headers, test_db):
    from app.access_controls.models import AccessControl
    from app.conference_profiles.models import ConferenceProfile
    from app.dialplans.models import Dialplan
    from app.domains.models import DomainSettings
    from app.music_on_hold.models import MusicOnHold
    from app.sip_profiles.models import SipProfile, SipProfileSetting
    from app.vars.models import Var

    domain_name = f"seedpbx-{uuid4().hex[:8]}.example.com"
    resp = await client.post(
        "/api/v1/domains",
        json={"domain_name": domain_name, "description": "seed parity", "enabled": True},
        headers=auth_headers,
    )
    assert resp.status_code == 201, resp.text
    domain_id = UUID(resp.json()["domain_id"])

    async with test_db() as s:
        assert (
            await s.execute(select(func.count()).select_from(Var).where(Var.domain_id == domain_id))
        ).scalar_one() >= 60
        assert (
            await s.execute(
                select(func.count())
                .select_from(SipProfile)
                .where(SipProfile.domain_id == domain_id)
            )
        ).scalar_one() >= 4
        assert (
            await s.execute(select(func.count()).select_from(SipProfileSetting))
        ).scalar_one() >= 200
        assert (
            await s.execute(
                select(func.count()).select_from(Dialplan).where(Dialplan.domain_id == domain_id)
            )
        ).scalar_one() >= 3
        assert (
            await s.execute(
                select(func.count())
                .select_from(DomainSettings)
                .where(DomainSettings.domain_id == domain_id)
            )
        ).scalar_one() >= 12
        assert (
            await s.execute(
                select(func.count())
                .select_from(AccessControl)
                .where(AccessControl.domain_id == domain_id)
            )
        ).scalar_one() >= 2
        assert (
            await s.execute(
                select(func.count())
                .select_from(ConferenceProfile)
                .where(ConferenceProfile.domain_id == domain_id)
            )
        ).scalar_one() >= 5
        assert (
            await s.execute(
                select(func.count())
                .select_from(MusicOnHold)
                .where(MusicOnHold.domain_id == domain_id)
            )
        ).scalar_one() >= 4
