import tomllib
from pathlib import Path
from typing import List

ALLOWED_LICENSES = ["MIT", "Apache-2.0", "BSD-3-Clause", "BSD-3", "BSD-2-Clause", "MPL-1.1", "CDDL-1.0", "ISC", "PostgreSQL", "MIT License", "Apache 2.0"]


def normalize_license(lic: str) -> str:
    return lic.strip()


def is_license_allowed(lic: str) -> bool:
    # allow exact or contains
    if lic in ALLOWED_LICENSES:
        return True
    # also accept common variations
    lic_lower = lic.lower()
    for allowed in ALLOWED_LICENSES:
        if allowed.lower() in lic_lower or lic_lower in allowed.lower():
            return True
    # explicitly reject GPL
    if "gpl" in lic_lower or "agpl" in lic_lower or "sspl" in lic_lower:
        return False
    return False


def load_plugins(root: Path | str = "plugins") -> List[dict]:
    root = Path(root)
    plugins = []
    if not root.exists():
        return plugins
    for manifest in root.glob("*/manifest.toml"):
        with open(manifest, "rb") as f:
            data = tomllib.load(f)
        lic = data.get("license", "")
        if not is_license_allowed(lic):
            raise ValueError(f"license {lic} not allowed for plugin {data.get('name', manifest.parent.name)} — see allowed {ALLOWED_LICENSES}")
        # optionally import router
        plugins.append(data)
    return plugins


def validate_plugin_license(lic: str):
    if not is_license_allowed(lic):
        raise ValueError(f"license {lic} not allowed")
