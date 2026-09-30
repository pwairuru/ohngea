"""Seed parity — fresh installs must seed working PBX out of box."""


def test_global_default_settings_count():
    from app.seeds.default_settings import DEFAULT_SETTINGS

    cats = {r["category"] for r in DEFAULT_SETTINGS}
    assert len(DEFAULT_SETTINGS) >= 1200, f"only {len(DEFAULT_SETTINGS)} defaults"
    assert {"domain", "email", "extension", "fax", "voicemail", "cdr", "provision"} <= cats


def test_vars_sofia_perms_fixtures():
    from app.seeds.vars import VARS
    from app.seeds.sofia import SOFIA_GLOBAL_SETTINGS
    from app.seeds.permissions import ALL_PERMISSIONS

    assert len(VARS) >= 60, f"only {len(VARS)} vars"
    assert {v["var_name"] for v in VARS} >= {"global_codec_prefs", "default_language", "default_country"}
    assert len(SOFIA_GLOBAL_SETTINGS) == 6
    assert len(ALL_PERMISSIONS) >= 300
    assert "var_view" in ALL_PERMISSIONS and "sofia_global_setting_view" in ALL_PERMISSIONS


def test_domain_seeds_cover_working_pbx():
    from app.seeds.sip_profiles import SIP_PROFILES
    from app.seeds.operational import CONFERENCE_PROFILES, MOH_ENTRIES, ACCESS_CONTROLS
    from app.seeds.dialplans import DEFAULT_DIALPLANS
    from app.seeds.domain_settings import DOMAIN_DEFAULTS

    assert len(SIP_PROFILES) >= 4
    assert sum(len(p["settings"]) for p in SIP_PROFILES) >= 200
    assert len(CONFERENCE_PROFILES) >= 5
    assert len(MOH_ENTRIES) >= 4
    assert len(ACCESS_CONTROLS) >= 2
    assert any(d["name"] == "local_extension" for d in DEFAULT_DIALPLANS)
    assert len(DOMAIN_DEFAULTS) >= 12
