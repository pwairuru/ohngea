"""Regenerate rust/crates/pbx-db/src/seeds_data.rs from app/seeds/*.py.

Usage (repo root):  python3 rust/crates/pbx-db/gen_seeds.py
"""

from pathlib import Path

from app.seeds.vars import VARS
from app.seeds.sofia import SOFIA_GLOBAL_SETTINGS as SOF
from app.seeds.domain_settings import DOMAIN_DEFAULTS as DD
from app.seeds.default_settings import DEFAULT_SETTINGS as D


def rs(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


def opt(s) -> str:
    return "None" if s is None or s == "" else f"Some({rs(s)})"


def main() -> None:
    out = Path(__file__).with_name("src") / "seeds_data.rs"
    lines: list[str] = [
        "// GENERATED from app/seeds/*.py — do not hand-edit.",
        "// Regenerate with: python3 rust/crates/pbx-db/gen_seeds.py",
        "// default_settings here is a representative subset (>=50 rows);",
        "// the full 1338 rows seed via alembic migration 035_provision_defaults.",
        "//",
        "// Included by `seeds.rs` via `include!` — seed structs are in scope there.",
        "",
        "/// 68 rows from app/seeds/vars.py (fusionpbx vars.xml X-PRE-PROCESS).",
        "pub const VARS_SEED: &[VarSeed] = &[",
    ]
    for v in VARS:
        lines += [
            "    VarSeed {",
            f"        var_category: {rs(v['var_category'])},",
            f"        var_name: {rs(v['var_name'])},",
            f"        var_value: {rs(v['var_value'])},",
            f"        var_command: {opt(v['var_command'])},",
            f"        var_enabled: {'true' if v['var_enabled'] else 'false'},",
            f"        var_order: {v['var_order']},",
            f"        var_description: {opt(v['var_description'])},",
            f"        var_hostname: {opt(v['var_hostname'])},",
            "    },",
        ]
    lines += [
        "];",
        "",
        "/// 6 rows from app/seeds/sofia.py (sofia_global_settings, global table).",
        "pub const SOFIA_SEED: &[SofiaSeed] = &[",
    ]
    for r in SOF:
        lines += [
            "    SofiaSeed {",
            f"        uuid: {rs(r['uuid'])},",
            f"        name: {rs(r['name'])},",
            f"        value: {rs(r['value'])},",
            f"        enabled: {'true' if r['enabled'] else 'false'},",
            f"        description: {opt(r['description'])},",
            "    },",
        ]
    lines += [
        "];",
        "",
        "/// 14 rows from app/seeds/domain_settings.py DOMAIN_DEFAULTS.",
        "pub const DOMAIN_DEFAULTS_SEED: &[DomainDefaultSeed] = &[",
    ]
    for cat, sub, k, v in DD:
        lines += [
            "    DomainDefaultSeed {",
            f"        category: {rs(cat)},",
            f"        subcategory: {rs(sub)},",
            f"        key: {rs(k)},",
            f"        value: {rs(v)},",
            "    },",
        ]
    subset = D[:30] + [r for r in D if r["category"] == "theme"]
    lines += [
        "];",
        "",
        f"/// Representative subset ({len(subset)} of all {len(D)} rows) of app/seeds/default_settings.py.",
        "/// Full parity set seeds via alembic 035_provision_defaults (PROVISION_DEFAULTS).",
        "pub const DEFAULT_SETTINGS_SEED: &[DefaultSettingSeed] = &[",
    ]
    for r in subset:
        lines += [
            "    DefaultSettingSeed {",
            f"        uuid: {rs(r['uuid'])},",
            f"        category: {rs(r['category'])},",
            f"        subcategory: {rs(r['subcategory'])},",
            f"        name: {rs(r['name'])},",
            f"        value: {opt(r['value'])},",
            f"        enabled: {'true' if r['enabled'] else 'false'},",
            f"        description: {opt(r['description'])},",
            "    },",
        ]
    lines += ["];"]
    out.write_text("\n".join(lines) + "\n")
    print(f"wrote {out}: vars={len(VARS)} sofia={len(SOF)} domain={len(DD)} subset={len(subset)}")


if __name__ == "__main__":
    main()
