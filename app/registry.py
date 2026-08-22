import tomllib
from pathlib import Path
from typing import List, Dict

from app.plugins.registry import is_license_allowed, validate_plugin_license

def validate_license(lic: str):
    validate_plugin_license(lic)
    return True

def _load_manifest(p: Path) -> Dict:
    with open(p, "rb") as f:
        data = tomllib.load(f)
    lic = data.get("license", "MIT")
    if not is_license_allowed(lic):
        raise ValueError(f"license {lic} not allowed")
    return data

def discover_apps(root_app="app", root_plugins="plugins") -> List[Dict]:
    apps = []
    # 1. Scan app/<name>/manifest.toml
    for m in Path(root_app).glob("*/manifest.toml"):
        d = _load_manifest(m)
        d.setdefault("depends_on", [])
        d.setdefault("enabled_by_default", True)
        d.setdefault("category", "advanced")
        d["source"] = "app"
        d["has_router"] = (m.parent / "router.py").exists()
        apps.append(d)
    # 2. Auto-detect app/<name>/router.py without manifest
    for router in Path(root_app).glob("*/router.py"):
        name = router.parent.name
        if any(a["name"] == name for a in apps):
            continue
        if name in ("auth", "db", "middleware", "plugins", "workers", "config", "freeswitch", "cli"):
            continue
        apps.append({"name": name, "version": "1.0.0", "license": "MIT", "description": f"Core app {name}", "routes": [f"/{name}"], "category": "advanced", "depends_on": [], "enabled_by_default": True, "source": "app", "has_router": True})
    # 3. Scan plugins
    if Path(root_plugins).exists():
        for m in Path(root_plugins).glob("*/manifest.toml"):
            d = _load_manifest(m)
            d.setdefault("depends_on", [])
            d.setdefault("enabled_by_default", True)
            d.setdefault("category", "advanced")
            d["source"] = "plugin"
            d["has_router"] = (m.parent / "router.py").exists()
            # dedupe if already in app
            if any(a["name"] == d["name"] for a in apps):
                continue
            apps.append(d)
    return sorted(apps, key=lambda x: x["name"])


def get_app(name: str, root_app="app", root_plugins="plugins") -> Dict | None:
    for a in discover_apps(root_app=root_app, root_plugins=root_plugins):
        if a.get("name") == name:
            return a
    return None
