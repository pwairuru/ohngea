import tempfile
import pathlib
import pytest
from app.plugins.registry import load_plugins, is_license_allowed


def test_license_allowlist():
    assert is_license_allowed("MIT")
    assert is_license_allowed("Apache-2.0")
    assert is_license_allowed("MPL-1.1")
    assert not is_license_allowed("GPL-3.0")
    assert not is_license_allowed("AGPL-3.0")


def test_plugin_license_reject(tmp_path):
    # create bad plugin
    bad = tmp_path / "bad"
    bad.mkdir()
    (bad / "manifest.toml").write_text('name="bad"\nlicense="GPL-3.0"\n')
    with pytest.raises(ValueError, match="license"):
        load_plugins(tmp_path)


def test_plugin_load_example():
    # real plugins dir should have at least example
    plugins = load_plugins("plugins")
    assert len(plugins) >= 1
    names = [p["name"] for p in plugins]
    assert "example" in names
    assert "call_center" in names
