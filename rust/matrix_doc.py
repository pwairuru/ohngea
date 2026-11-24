#!/usr/bin/env python3
"""Emit docs/PORT_PARITY.md: per-module evidence table."""
import glob, json, os, re
REPO = '/home/wairuru/projects/fusionpbx'
rows = json.load(open('/tmp/matrix_rows.json'))

# Rust route inventory
rust_routes = {}
file_gates = {}
for f in glob.glob(f'{REPO}/rust/crates/pbx-api/src/*.rs'):
    src = open(f).read()
    for m in re.finditer(r'"/api/v1/([a-z_0-9/\{\}]+)"', src):
        pref = m.group(1).split('/')[0]
        rust_routes.setdefault(pref, set()).add(os.path.basename(f))
    gates = set()
    for m in re.finditer(r'deny_if_missing(?:_any)?\(\s*&claims?,\s*(?:&\[([^\]]*)\]|"([a-z_0-9]+)")', src):
        g1, g2 = m.groups()
        if g2:
            gates.add(g2)
        else:
            gates.update(re.findall(r'"([a-z_0-9]+)"', g1 or ''))
    file_gates[os.path.basename(f)] = sorted(gates)

MANUAL = {
    'aastra': ([], 'vendor metadata only (app_config, no tables/routes)'),
    'algo': ([], 'vendor metadata only'), 'avaya': ([], 'vendor metadata only'),
    'azure': ([], 'vendor metadata only'), 'cisco': ([], 'vendor metadata only'),
    'fanvil': ([], 'vendor metadata only'), 'flyingvoice': ([], 'vendor metadata only'),
    'grandstream': ([], 'vendor metadata only'), 'htek': ([], 'vendor metadata only'),
    'poly': ([], 'vendor metadata only'), 'polycom': ([], 'vendor metadata only'),
    'snom': ([], 'vendor metadata only'), 'swissvoice': ([], 'vendor metadata only'),
    'yealink': ([], 'vendor metadata only'),
    'core/events': ([], 'internal plugin bus (events class, no HTTP surface)'),
    'core/install': (['domains'], 'first-install via POST /api/v1/domains/bootstrap'),
    'core/upgrade': ([], 'schema upgrades via Migrator (fresh) + alembic (prod)'),
    'core/user_settings': (['users'], 'GET/PUT /api/v1/users/{id}/settings'),
    'core/websockets': ([], 'transport replaced by REST + ESL; surface = operator_panel'),
    'provision': (['provision'], 'GET /provision/{mac}.cfg vendor-matrix render'),
}
ALIASES = {
    'dialplan_inbound': ['dialplans'], 'dialplan_outbound': ['dialplans'],
    'calls_active': ['calls', 'active_calls'], 'active_calls': ['calls'],
    'conferences_active': ['conferences'], 'call_center_active': ['call_centers'],
    'basic_operator_panel': ['operator_panel'], 'click_to_call': ['click_to_call'],
    'database_transactions': ['database_transactions'], 'emergency': ['emergency_logs'],
    'extension_settings': ['extension_settings'], 'fifo_list': ['fifo_list'],
    'log_viewer': ['logs', 'log_viewer'], 'registrations': ['registrations'],
    'sip_status': ['sip_status'], 'sofia_global_settings': ['sofia_global_settings'],
    'domain_limits': ['domain_limits'], 'domain_settings': ['domain_settings'],
    'email_templates': ['email_templates'], 'user_logs': ['user_logs'],
    'xml_cdr': ['cdr', 'xml_cdr'], 'system': ['system'], 'switch': ['switch'],
    'default_settings': ['default_settings'], 'authentication': ['auth'],
    'contacts': ['contacts'], 'dashboard': ['dashboards'], 'databases': ['databases'],
    'domains': ['domains'], 'groups': ['groups'], 'menus': ['menus'],
    'notifications': ['notifications'], 'permissions': ['permissions'],
    'services': ['services'], 'software': ['software'], 'users': ['users'],
    'vars': ['vars'], 'tones': ['tones'], 'phrases': ['phrases'], 'modules': ['modules'],
    'event_guard': ['event_guard'], 'dialplans': ['dialplans'],
    'destinations': ['destinations'], 'devices': ['devices'], 'gateways': ['gateways'],
    'fax': ['fax'], 'fax_queue': ['fax_queue'], 'ring_groups': ['ring_groups'],
    'ivr_menus': ['ivr_menus'], 'conferences': ['conferences'],
    'conference_centers': ['conference_centers'],
    'conference_controls': ['conference_controls'],
    'conference_profiles': ['conference_profiles'],
    'call_centers': ['call_centers'], 'call_flows': ['call_flows'],
    'call_forward': ['call_forward'], 'call_block': ['call_block'],
    'call_broadcast': ['call_broadcast'], 'call_recordings': ['call_recordings'],
    'bridges': ['bridges'], 'streams': ['streams'], 'fifo': ['fifo'],
    'follow_me': ['follow_me'], 'feature_codes': ['feature_codes'],
    'music_on_hold': ['music_on_hold'], 'number_translations': ['number_translations'],
    'pin_numbers': ['pin_numbers'], 'recordings': ['recordings'],
    'time_conditions': ['time_conditions'], 'voicemails': ['voicemails'],
    'voicemail_greetings': ['voicemail_greetings'], 'email_queue': ['email_queue'],
    'extensions': ['extensions'], 'sip_profiles': ['sip_profiles'],
    'access_controls': ['access_controls'], 'operator_panel': ['operator_panel'],
}
# rename-table notes (PHP v_* -> Rust entity)
RENAMES = {'call_centers': 'v_call_center_queues', 'cdr': 'v_xml_cdr',
           'ring_group_members': 'v_ring_group_destinations (VIEW)',
           'domain_apps': 'Rust-only (plugin system)', 'domain_plugins': 'Rust-only (plugin system)',
           'cdr_archive': 'Rust-only (CDR archiving)', 'remaining_records': 'Rust-only (migration spillover)'}

with open(f'{REPO}/docs/PORT_PARITY.md', 'w') as o:
    o.write("# PHP → Rust port parity matrix\n\n")
    o.write("Generated evidence: every fusionpbx `app/*` + `core/*` module mapped to Rust routers, routes, permission gates, and tables. 111 Rust route prefixes total.\n\n")
    o.write("| PHP module | PHP perms | PHP tables (→ Rust entity [RLS]) | Rust routes | Rust files | Rust gates (sample) | note |\n")
    o.write("|---|---|---|---|---|---|---|\n")
    for key, perms, tables, ents, rls in rows:
        if key in MANUAL:
            prefs, note = MANUAL[key]
            tcol, ecol = '—', '—'
        else:
            prefs = ALIASES.get(key.split('/')[-1], [])
            if not prefs:
                guess = key.rstrip('s')
                prefs = sorted({p for p in rust_routes if p == key or p.startswith(key) or key.startswith(p) or guess in p or p in guess})
            tcol = ', '.join(f"{t}→{e}[{r}]" for t, e, r in zip(tables, ents, rls)) if tables else '—'
            note = ''
        files = sorted({f for p in prefs for f in rust_routes.get(p, set())})
        gates = sorted({g for f in files for g in file_gates.get(f, [])})
        o.write(f"| {key} | {len(perms)} | {tcol} | {','.join(prefs) if prefs else '—'} | {','.join(files) if files else '—'} | {','.join(gates[:5])} | {note} |\n")
    o.write("\n## Rename-table notes (same data, different names)\n\n")
    for k, v in RENAMES.items():
        o.write(f"- `{k}`: {v}\n")
print("wrote docs/PORT_PARITY.md")
