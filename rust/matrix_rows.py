#!/usr/bin/env python3
"""Emit per-module parity rows as JSON for docs/PORT_PARITY.md."""
import glob, json, os, re
REPO = '/home/wairuru/projects/fusionpbx'

def php_items(appdir):
    perms, tables = set(), set()
    try:
        src = open(f'{appdir}/app_config.php', errors='ignore').read()
    except OSError:
        return [], []
    for m in re.finditer(r"\['permissions'\][^\n;]*?'name'\]\s*=\s*[\"']([a-z_0-9]+)[\"']", src):
        perms.add(m.group(1))
    for m in re.finditer(r"\['table'\]\['name'\]\s*=\s*[\"'](v_[a-z_0-9]+)[\"']", src):
        tables.add(m.group(1))
    return sorted(perms), sorted(tables)

ents = {}
for f in glob.glob('rust/crates/pbx-db/src/entities/*.rs'):
    src = open(f).read()
    m = re.search(r'table_name\s*=\s*"(\w+)"', src)
    if m:
        ents[m.group(1)] = os.path.basename(f)
mig = open('rust/crates/pbx-db/src/migration.rs').read()

def bucket(tbl):
    m = re.search(r'STRICT_TABLES.*?\[(.*?)\];', mig, re.S)
    if f'"{tbl}"' in m.group(1):
        return 'STRICT'
    m = re.search(r'const NULLABLE_TABLES.*?\[(.*?)\];', mig, re.S)
    if f'"{tbl}"' in m.group(1):
        return 'NULLABLE'
    return 'open'

rows = []
for area, base in [('app', f'{REPO}/fusionpbx/app'), ('core', f'{REPO}/fusionpbx/core')]:
    for d in sorted(os.listdir(base)):
        key = d if area == 'app' else f'core/{d}'
        perms, tables = php_items(f'{base}/{d}')
        rows.append([key, perms, tables,
                     [ents.get(t[2:], 'MISSING') for t in tables],
                     [bucket(t[2:]) for t in tables]])
json.dump(rows, open('/tmp/matrix_rows.json', 'w'))
print(len(rows), 'modules')
allmiss = sorted({t for _, _, ts, _, _ in rows for t in ts if t[2:] not in ents})
print('PHP tables missing rust entity:', allmiss)
