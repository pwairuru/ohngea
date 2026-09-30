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
