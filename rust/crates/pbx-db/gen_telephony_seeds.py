"""Regenerate rust/crates/pbx-db/src/seeds_data_telephony.rs from app/seeds/*.py.

Usage (repo root):  python3 rust/crates/pbx-db/gen_telephony_seeds.py
"""

from pathlib import Path

from app.seeds.dialplans import DEFAULT_DIALPLANS as DP
from app.seeds.sip_profiles import SIP_PROFILES as SIP


def rs(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


def opt(s) -> str:
    return "None" if s is None else f"Some({rs(s)})"


def main() -> None:
    out = Path(__file__).with_name("src") / "seeds_data_telephony.rs"
    lines: list[str] = [
        "// GENERATED from app/seeds/dialplans.py + app/seeds/sip_profiles.py — do not hand-edit.",
        "// Regenerate with: python3 rust/crates/pbx-db/gen_telephony_seeds.py",
        "//",
        "// Included by `seeds.rs` via `include!` — seed structs are in scope there.",
        "",
        f"/// {len(DP)} dialplans from app/seeds/dialplans.py (each with 2 details).",
        "pub const DIALPLAN_SEED: &[DialplanSeed] = &[",
    ]
    for d in DP:
        lines += [
            "    DialplanSeed {",
            f"        name: {rs(d['name'])},",
            f"        order: {d.get('order') or 100},",
            f"        description: {opt(d.get('description'))},",
            "        details: &[",
        ]
        for det in d.get("details", []):
            lines += [
                "            DialplanDetailSeed {",
                f"                tag: {rs(det['tag'])},",
                f"                r#type: {rs(det['type'])},",
                f"                data: {rs(det.get('data') or '')},",
                f"                order: {det.get('order') or 0},",
                "            },",
            ]
        lines += ["        ],", "    },"]
    lines += ["];", ""]
    total_settings = sum(len(p.get("settings", [])) for p in SIP)
    total_domains = sum(len(p.get("domains", [])) for p in SIP)
    lines += [
        f"/// {len(SIP)} profiles + {total_domains} domains + {total_settings} settings from app/seeds/sip_profiles.py.",
        "pub const SIP_PROFILE_SEED: &[SipProfileSeed] = &[",
    ]
    for p in SIP:
        lines += [
            "    SipProfileSeed {",
            f"        name: {rs(p['name'])},",
            f"        enabled: {'true' if p.get('enabled') else 'false'},",
            f"        port: {p.get('port') or 5060},",
            f"        description: {opt(p.get('description'))},",
            "        domains: &[",
        ]
        for d in p.get("domains", []):
            lines += [
                "            SipDomainSeed {",
                f"                name: {rs(d.get('name') or '')},",
                f"                alias: {rs(d.get('alias') or 'false')},",
                f"                parse: {rs(d.get('parse') or 'true')},",
                "            },",
            ]
        lines += ["        ],", "        settings: &["]
        for s in p.get("settings", []):
            if not s.get("name"):
                continue
            lines += [
                "            SipSettingSeed {",
                f"                name: {rs(s['name'])},",
                f"                value: {opt(s.get('value'))},",
                f"                enabled: {'true' if s.get('enabled', True) else 'false'},",
                "            },",
            ]
        lines += ["        ],", "    },"]
    lines += ["];", ""]
    out.write_text("\n".join(lines))
    print(f"wrote {out} ({len(lines)} lines)")


if __name__ == "__main__":
    main()
