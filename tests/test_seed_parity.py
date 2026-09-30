"""Seed parity — fresh installs must seed working PBX out of box."""


def test_global_default_settings_count():
    from app.seeds.default_settings import DEFAULT_SETTINGS

    cats = {r["category"] for r in DEFAULT_SETTINGS}
    assert len(DEFAULT_SETTINGS) >= 1200, f"only {len(DEFAULT_SETTINGS)} defaults"
    assert {"domain", "email", "extension", "fax", "voicemail", "cdr", "provision"} <= cats
