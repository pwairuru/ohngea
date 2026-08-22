def test_discover_apps_finds_core_and_plugins():
    from app.registry import discover_apps
    apps = discover_apps()
    names = {a["name"] for a in apps}
    assert "ivr_menus" in names  # core app
    assert "fifo" in names
    assert "call_center" in names  # plugin
    assert "extensions" in names


def test_app_has_depends_on_field():
    from app.registry import discover_apps
    apps = {a["name"]: a for a in discover_apps()}
    assert "depends_on" in apps["destinations"]
    assert isinstance(apps["destinations"]["depends_on"], list)


def test_license_gate_preserved():
    from app.registry import validate_license
    import pytest
    with pytest.raises(ValueError):
        validate_license("GPL-3.0")
    assert validate_license("MIT") is True
