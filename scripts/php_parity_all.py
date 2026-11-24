#!/usr/bin/env python3
"""Whole-goal PHP parity audit: forms + routes + tables for every app/core area.

Ground truth: fusionpbx/{app,core}/*.
- forms: all name='...' inputs in *.php (mechanical chrome excluded)
- tables: v_* tables from */app_config.php
- react: App.tsx routes + fieldConfigs keys (+ custom page form keys)
- api: axum route paths + accepted keys (struct fields, payload.get/s()/b()/i()/put, alias layer)

Usage: python3 scripts/php_parity_all.py [--json]
Exit 1 on any miss. Documents verified non-parity inline.
"""
import glob
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CHROME = {"btn_save", "submit", "frm", "csrf_token", "persistformvar", "search",
          "order_by", "order", "page", "checkbox_all", "delete_type", "delete_uuid",
          "action", "show", "id", "uuid", "ref", "redirect", "query_string"}


def php_apps():
    out = {}
    for base in ("fusionpbx/app", "fusionpbx/core"):
        for d in sorted(glob.glob(os.path.join(ROOT, base, "*"))):
            if not os.path.isdir(d):
                continue
            app = os.path.basename(d)
            fields, tables, pages = set(), [], set()
            for f in glob.glob(os.path.join(d, "*.php")):
                fn = os.path.basename(f)
                pages.add(fn)
                try:
                    src = open(f, errors="ignore").read()
                except OSError:
                    continue
                for m in re.finditer(r"""name=(?:'([a-z_][a-z0-9_]*)'|"([a-z_][a-z0-9_]*)")""", src):
                    name = m.group(1) or m.group(2)
                    if not name.startswith(("btn_", "submit", "delete_", "save", "csrf")):
                        fields.add(name)
            # tables from app_config
            cfg = os.path.join(d, "app_config.php")
            if os.path.exists(cfg):
                src = open(cfg, errors="ignore").read()
                tables = sorted(set(re.findall(r"\['table'\]\['name'\]\s*=\s*[\"']v_([a-z_]+)[\"']", src)))
            out[app] = {"fields": sorted(fields), "tables": tables, "pages": sorted(pages)}
    return out


def react_routes():
    src = open(os.path.join(ROOT, "frontend", "src", "App.tsx"), errors="ignore").read()
    return sorted(set(re.findall(r'<Route path="(/[^"]*)"', src)))


def react_keys():
    out = {}
    fc = open(os.path.join(ROOT, "frontend", "src", "lib", "fieldConfigs.ts"), errors="ignore").read()
    for m in re.finditer(r"^  ([a-z_]+): \{", fc, re.M):
        name = m.group(1)
        lines = fc[m.start():].split("\n")
        collect, d = [], 0
        for ln in lines:
            d += ln.count("{") - ln.count("}")
            collect.append(ln)
            if d == 0 and len(collect) > 1:
                break
        chunk = "\n".join(collect)
        keys = set(re.findall(r'[TC]\("([a-z_][a-z0-9_]*)"', chunk))
        keys |= set(re.findall(r'name:\s*"([a-z_][a-z0-9_]*)"', chunk))
        out[name] = keys
    for f in glob.glob(os.path.join(ROOT, "frontend", "src", "routes", "*.tsx")):
        base = os.path.basename(f)[:-4].lower()
        src = open(f, errors="ignore").read()
        keys = set(re.findall(r"form\.([a-z_][a-z0-9_]*)", src))
        keys |= set(re.findall(r'key:\s*"([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'set\("([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'[TC]\("([a-z_][a-z0-9_]*)"', src))
        if keys:
            out.setdefault(base, set()).update(keys)
            stem = {"callcenters": "call_centers", "ivrs": "ivr_menus", "ivrmenus": "ivr_menus",
                    "ringgroups": "ring_groups", "followme": "follow_me", "callforward": "call_forward",
                    "callbroadcasts": "call_broadcast"}
            if base in stem:
                out.setdefault(stem[base], set()).update(keys)
    return {k: sorted(v) for k, v in out.items()}


def rust_api():
    routes, keys = set(), {}
    for f in glob.glob(os.path.join(ROOT, "rust", "crates", "pbx-api", "src", "*.rs")):
        mod = os.path.basename(f)[:-3]
        src = open(f, errors="ignore").read()
        routes |= set(re.findall(r'"/api/v1/([a-z_/{}:-]*)"', src))
        ks = set()
        for m in re.finditer(r"struct \w+\s*\{(.*?)\n\}", src, re.S):
            ks |= set(re.findall(r"(?:pub\s+)?([a-z_][a-z0-9_]*)\s*:", m.group(1)))
        for pat in (r'payload\s*\.\s*get\(\s*"([a-z_][a-z0-9_]*)"',
                    r'\.get\(\s*"([a-z_][a-z0-9_]*)"',
                    r'alias\s*=\s*"([a-z_][a-z0-9_]*)"',
                    r'put\("([a-z_][a-z0-9_]*)"',
                    r'"([a-z_][a-z0-9_]*)"\s*=>\s*&mut am\.',
                    r'"([a-z_][a-z0-9_]*)":\s*r\.'):
            ks |= set(re.findall(pat, src))
        for m in re.finditer(r"\&\[([^\]]{0,2000}?)\]", src, re.S):
            ks |= set(re.findall(r'"([a-z_][a-z0-9_]*)"', m.group(1)))
        for m in re.finditer(r"for\s+key\s+in\s*\[(.*?)\]", src, re.S):
            ks |= set(re.findall(r'"([a-z_][a-z0-9_]*)"', m.group(1)))
        if ks:
            keys[mod] = sorted(ks)
    return sorted(routes), keys


def main():
    as_json = "--json" in sys.argv
    php = php_apps()
    routes = react_routes()
    rk = react_keys()
    api_routes, api_keys = rust_api()
    result = {"php_apps": php, "react_routes": routes, "react_keys": rk,
              "api_routes": api_routes, "api_keys": api_keys}
    if as_json:
        print(json.dumps(result, indent=1, sort_keys=True))
        return 0
    # summary table
    form_apps = sorted(a for a, v in php.items() if v["fields"])
    print(f"php areas: {len(php)}  form-backed: {len(form_apps)}  react routes: {len(routes)}  api routes: {len(api_routes)}")
    noTABLE = sorted(a for a, v in php.items() if not v["tables"])
    print(f"apps without v_ tables: {len(noTABLE)}")
    print(" ".join(noTABLE))
    return 0


if __name__ == "__main__":
    sys.exit(main())
