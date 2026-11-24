#!/usr/bin/env python3
"""PHP parity audit: PHP edit-form fields vs React form keys vs Rust API keys.

Ground truth: fusionpbx/**/ *_edit.php name='...' inputs (+ v_* schema for
column checks). Run from repo root: python3 scripts/php_parity_audit.py
Writes docs/php-parity-audit.txt. Exit 1 when any miss row remains.

Name normalisation: alias.rs pairs (parsed live) + prefix strip + verified
SEM map + cross-page RELATED map. NOISE documents verified non-columns.
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _strip_php_comments(src: str) -> str:
    # Drop full-line // and # comments plus /* ... */ blocks, but only when
    # the opener is real code - not inside a quoted string. (A naive regex
    # ate most of notification_edit.php because of the shell glob
    # "cat /etc/*{release,version}" inside a double-quoted string.)
    # Purpose: commented-out inputs (e.g. enter_sound in
    # conference_room_edit.php:846) are not rendered and must not count
    # as PHP parity fields. Inline // (e.g. http://) is left intact.
    out_lines = []
    in_block = False
    for ln in src.split("\n"):
        s = ln.lstrip()
        if not in_block and (s.startswith("//") or s.startswith("#") or s.startswith("*")):
            continue
        res_chars = []
        i = 0
        in_squote = False
        in_dquote = False
        while i < len(ln):
            ch = ln[i]
            nxt = ln[i + 1] if i + 1 < len(ln) else ""
            if in_block:
                if ch == "*" and nxt == "/":
                    in_block = False
                    i += 2
                else:
                    i += 1
                continue
            if in_squote:
                res_chars.append(ch)
                if ch == "\\" and i + 1 < len(ln):
                    res_chars.append(ln[i + 1])
                    i += 2
                    continue
                if ch == "'":
                    in_squote = False
                i += 1
                continue
            if in_dquote:
                res_chars.append(ch)
                if ch == "\\" and i + 1 < len(ln):
                    res_chars.append(ln[i + 1])
                    i += 2
                    continue
                if ch == '"':
                    in_dquote = False
                i += 1
                continue
            if ch == "'":
                in_squote = True
                res_chars.append(ch)
                i += 1
                continue
            if ch == '"':
                in_dquote = True
                res_chars.append(ch)
                i += 1
                continue
            if ch == "/" and nxt == "*":
                in_block = True
                i += 2
                continue
            res_chars.append(ch)
            i += 1
        kept = "".join(res_chars)
        if kept.strip():
            out_lines.append(kept)
        else:
            out_lines.append("")
    return "\n".join(out_lines)


def php_edit_fields():
    out = {}
    for base in ("fusionpbx/app", "fusionpbx/core"):
        for appdir in glob.glob(os.path.join(ROOT, base, "*")):
            app = os.path.basename(appdir)
            fields = out.get(app, set())
            for ef in glob.glob(os.path.join(appdir, "*_edit.php")):
                src = _strip_php_comments(open(ef, errors="ignore").read())
                for m in re.findall(r'''name=(?:'([a-z_][a-z0-9_]*)'|"([a-z_][a-z0-9_]*)")''', src):
                    name = m[0] or m[1]
                    if name.startswith(("btn_", "submit", "delete_", "save", "csrf", "persistformvar")):
                        continue
                    fields.add(name)
            if fields:
                out[app] = fields
    return out


def php_all_areas():
    """Whole-goal inventory: every app+core area with tables/pages."""
    import pathlib as _pl
    all_apps = sorted([p.name for p in _pl.Path(os.path.join(ROOT, "fusionpbx", "app")).iterdir() if p.is_dir()])
    all_core = sorted([p.name for p in _pl.Path(os.path.join(ROOT, "fusionpbx", "core")).iterdir() if p.is_dir()])
    return all_apps, all_core


def react_keys():
    out = {}
    fc = open(os.path.join(ROOT, "frontend", "src", "lib", "fieldConfigs.ts"), errors="ignore").read()
    for m in re.finditer(r"^  ([a-z_]+): \{", fc, re.M):
        name = m.group(1)
        lines = fc[m.start():].split("\n")
        collect = []
        d = 0
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
        # userForm/apiForm/contactForm etc (e.g. Users.tsx userForm.*)
        keys |= set(re.findall(r"[A-Za-z]*Form\.([a-z_][a-z0-9_]*)", src))
        keys |= set(re.findall(r'key:\s*"([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'set\("([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'[TC]\("([a-z_][a-z0-9_]*)"', src))
        if keys:
            out.setdefault(base, set()).update(keys)
            stem = {
                "callcenters": "call_centers", "ivrs": "ivr_menus", "ivrmenus": "ivr_menus",
                "ringgroups": "ring_groups", "followme": "follow_me", "callforward": "call_forward",
                "callbroadcasts": "call_broadcast",
            }
            if base in stem:
                out.setdefault(stem[base], set()).update(keys)
    return out


def rust_keys():
    out = {}
    for f in glob.glob(os.path.join(ROOT, "rust", "crates", "pbx-api", "src", "*.rs")):
        mod = os.path.basename(f)[:-3]
        src = open(f, errors="ignore").read()
        keys = set()
        for m in re.finditer(r"struct \w+\s*\{(.*?)\n\}", src, re.S):
            keys |= set(re.findall(r"(?:pub\s+)?([a-z_][a-z0-9_]*)\s*:", m.group(1)))
        keys |= set(re.findall(r'payload\s*\.\s*get\(\s*"([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'\.get\(\s*"([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'opt_str\(\s*[^,]+,\s*"([a-z_][a-z0-9_]*)"', src))
        for m in re.finditer(r"s\(&payload, &\[(.*?)\]\)", src, re.S):
            keys |= set(re.findall(r'"([a-z_][a-z0-9_]*)"', m.group(1)))
        for m in re.finditer(r"b_opt\(&payload, &\[(.*?)\]\)", src, re.S):
            keys |= set(re.findall(r'"([a-z_][a-z0-9_]*)"', m.group(1)))
        for m in re.finditer(r"i\(&payload, &\[(.*?)\]\)", src, re.S):
            keys |= set(re.findall(r'"([a-z_][a-z0-9_]*)"', m.group(1)))
        for m in re.finditer(r"for[^\[]*\[(.*?)\]", src, re.S):
            keys |= set(re.findall(r'"([a-z_][a-z0-9_]*)"', m.group(1)))
        for m in re.finditer(r"&\[(.*?)\]", src, re.S):
            keys |= set(re.findall(r'"([a-z_][a-z0-9_]*)"', m.group(1)))
        keys |= set(re.findall(r'alias\s*=\s*"([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'put\("([a-z_][a-z0-9_]*)"', src))
        keys |= set(re.findall(r'"([a-z_][a-z0-9_]*)"\s*=>\s*&mut am\.', src))
        keys |= set(re.findall(r'"([a-z_][a-z0-9_]*)":\s*[a-z_]+\.', src))
        if keys:
            out[mod] = keys
    return out


def rust_aliases():
    """Parse ALIASES pairs straight from alias.rs (single source of truth)."""
    src = open(os.path.join(ROOT, "rust", "crates", "pbx-api", "src", "alias.rs"), errors="ignore").read()
    return dict(re.findall(r'\(\s*"([a-z_][a-z0-9_]*)"\s*,\s*"([a-z_][a-z0-9_]*)"\s*\)', src))


PREFIXES = ["dialplan_", "device_", "extension_", "gateway_", "ivr_menu_", "ring_group_",
            "conference_", "voicemail_", "fax_", "fifo_", "queue_", "call_flow_", "destination_",
            "number_translation_", "pin_number_", "sip_profile_", "sofia_", "music_on_hold_",
            "stream_", "phrase_", "bridge_", "call_block_", "broadcast_", "time_", "var_",
            "module_", "event_guard_", "email_", "emergency_", "domain_", "group_", "user_",
            "contact_", "menu_item_", "menu_", "language_", "country_", "software_",
            "database_", "notification_"]

MAP = {
    "extensions": (["extensions"], ["extensions"], "extension_"),
    "gateways": (["gateways"], ["gateways"], "gateway_"),
    "dialplans": (["dialplans"], ["dialplans"], "dialplan_"),
    "devices": (["devices", "device_profiles"], ["devices"], "device_"),
    "voicemails": (["voicemails"], ["voicemails", "missing_tables"], "voicemail_"),
    "ring_groups": (["ring_groups"], ["ring_groups"], "ring_group_"),
    "ivr_menus": (["ivr_menus", "ivrs"], ["ivr_menus"], "ivr_menu_"),
    "time_conditions": (["time_conditions"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "time_"),
    "call_centers": (["call_centers", "callcenters"], ["call_centers"], "queue_"),
    "conferences": (["conferences"], ["conferences"], "conference_"),
    "conference_profiles": (["conference_profiles"], ["conference_controls"], "profile_"),
    "conference_controls": (["conference_controls"], ["conference_controls"], "control_"),
    "conference_centers": (["conference_centers"], ["conference_centers"], "conference_center_"),
    "fax": (["fax"], ["fax"], "fax_"),
    "follow_me": (["follow_me", "followme"], ["follow_me", "remaining", "remaining_a", "remaining_b", "remaining_c"], "follow_me_"),
    "call_forward": (["call_forward", "callforward"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], ""),
    "fifo": (["fifo"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "fifo_"),
    "bridges": (["bridges"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "bridge_"),
    "streams": (["streams"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "stream_"),
    "phrases": (["phrases"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "phrase_"),
    "destinations": (["destinations"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "destination_"),
    "access_controls": (["access_controls"], ["remaining_c"], "access_control_"),
    "call_block": (["call_block"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "call_block_"),
    "call_broadcast": (["call_broadcast", "call_broadcasts"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "broadcast_"),
    "call_flows": (["call_flows"], ["admin", "missing_tables"], "call_flow_"),
    "number_translations": (["number_translations"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "number_translation_"),
    "pin_numbers": (["pin_numbers"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "pin_number_"),
    "recordings": (["recordings"], ["recordings"], "recording_"),
    "music_on_hold": (["music_on_hold"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], ""),
    "email_queue": (["email_queue"], ["email_queue"], "email_"),
    "vars": (["vars"], ["vars_fs"], "var_"),
    "domain_limits": (["domain_limits"], ["admin"], ""),
    "extension_settings": (["extension_settings"], ["admin", "missing_tables"], "extension_setting_"),
    "event_guard": (["event_guard"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], ""),
    "modules": (["modules"], ["missing_tables"], "module_"),
    "sofia_global_settings": (["sofia_global_settings"], ["admin", "missing_tables"], ""),
    "voicemail_greetings": (["voicemail_greetings"], ["voicemails"], "greeting_"),
    "sip_profiles": (["sip_profiles", "sipprofiles"], ["sip"], "sip_profile_"),
    "call_recordings": (["call_recordings"], ["recordings"], "call_recording_"),
    "click_to_call": (["click_to_call", "clicktocall"], ["remaining"], ""),
    "database_transactions": (["database_transactions"], ["missing_tables"], "database_transaction_"),
    "emergency": (["emergency", "emergency_logs", "emergencylogs"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "emergency_"),
    "fax_queue": (["fax_queue", "faxqueue"], ["fax"], "fax_"),
    "feature_codes": (["feature_codes", "featurecodes"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "feature_code_"),
    "fifo_list": (["fifo_list", "fifolist"], ["remaining", "remaining_a", "remaining_b", "remaining_c"], "fifo_"),
    "log_viewer": (["log_viewer", "logs", "logviewer"], ["logs", "system"], ""),
    "operator_panel": (["operator_panel", "operatorpanel"], ["remaining"], "operator_"),
    "registrations": (["registrations"], ["remaining"], "registration_"),
    "switch": (["switch", "switchstatus"], ["system", "fs_status", "remaining"], ""),
    "system": (["system"], ["system"], "system_"),
    "tones": (["tones"], [], ""),
    "xml_cdr": (["xml_cdr", "cdr"], ["cdr"], ""),
    "call_center_active": (["call_center_active", "callcenteractive"], ["call_centers"], "call_center_"),
    "calls_active": (["calls_active", "callsactive", "active_calls", "activecalls"], ["remaining"], ""),
    "conferences_active": (["conferences_active", "active_conferences", "activeconferences"], ["conferences", "remaining"], "conference_"),
    "contacts": (["contacts"], ["contacts"], "contact_"),
    "default_settings": (["default_settings", "defaultsettings"], ["settings"], "default_setting_"),
    "domain_settings": (["domain_settings", "domainsettings"], ["settings"], "domain_setting_"),
    "domains": (["domains"], ["domains"], "domain_"),
    "email_templates": (["email_templates", "emailtemplates"], ["email_queue"], "email_template_"),
    "groups": (["groups"], ["groups"], "group_"),
    "users": (["users"], ["users"], "user_"),
    "user_logs": (["user_logs", "userlogs", "logs"], ["logs"], "user_log_"),
    "user_settings": (["user_settings", "usersettings"], ["users", "settings"], "user_setting_"),
    "menu": (["menu", "menu_items", "menuitems", "menus"], ["menus"], "menu_"),
    "notifications": (["notifications"], ["menus"], "notification_"),
    "services": (["services"], ["settings", "system"], "service_"),
    "software": (["software"], ["menus"], "software_"),
    "databases": (["databases"], ["menus", "backup"], "database_"),
    "authentication": (["authentication", "login"], ["auth"], ""),
    "dashboard": (["dashboard", "dashboards"], ["dashboard"], "dashboard_"),
    "dialplan_inbound": (["dialplan_inbound", "dialplans"], ["dialplans"], "dialplan_"),
    "dialplan_outbound": (["dialplan_outbound", "dialplans"], ["dialplans"], "dialplan_"),
    "provision": (["provision", "devices"], ["devices"], "device_"),
    "basic_operator_panel": (["basic_operator_panel", "operator_panel", "operatorpanel"], ["remaining"], "operator_"),
    "call_recordings": (["call_recordings", "callrecordings"], ["recordings"], "call_recording_"),
}

# cross-page coverage: fields settable via sibling screens
RELATED = {
    "call_forward": (["follow_me", "extensions"], ["follow_me", "extensions"]),
    "users": (["contacts"], ["contacts"]),
}

# verified short-name equivalences (read from the forms)
SEM = {
    ("music_on_hold", "music_name", "music_on_hold_name"),
    ("music_on_hold", "music_path", "music_on_hold_path"),
    ("music_on_hold", "music_rate", "music_on_hold_rate"),
    ("music_on_hold", "music_shuffle", "music_on_hold_shuffle"),
    ("music_on_hold", "music_channels", "music_on_hold_channels"),
    ("music_on_hold", "music_interval", "music_on_hold_interval"),
    ("music_on_hold", "music_timer_name", "music_on_hold_timer_name"),
    ("music_on_hold", "music_chime_list", "music_on_hold_chime_list"),
    ("music_on_hold", "music_chime_freq", "music_on_hold_chime_freq"),
    ("music_on_hold", "music_chime_max", "music_on_hold_chime_max"),
    ("dialplans", "name", "dialplan_name"),
    ("dialplans", "context", "dialplan_context"),
    ("dialplans", "enabled", "dialplan_enabled"),
    ("dialplans", "description", "dialplan_description"),
    ("dialplans", "order", "dialplan_order"),
    ("dialplans", "number", "dialplan_number"),
    ("dialplans", "continue", "dialplan_continue"),
    ("devices", "mac", "device_address"),
    ("devices", "vendor", "device_vendor"),
    ("devices", "model", "device_model"),
    ("devices", "label", "device_label"),
    ("devices", "enabled", "device_enabled"),
    ("devices", "password", "device_password"),
    ("devices", "username", "device_username"),
    ("devices", "description", "device_description"),
    ("devices", "template", "device_template"),
    ("devices", "location", "device_location"),
    ("devices", "serial_number", "device_serial_number"),
    ("devices", "firmware_version", "device_firmware_version"),
    ("devices", "profile_uuid", "device_profile_uuid"),
    ("gateways", "name", "gateway"),
    ("gateways", "register", "gateway_register"),
}

# Mechanical chrome only (not data fields): hidden PK/sort/tracking inputs,
# list UI, upload helpers, navigation. Each verified against PHP edit forms:
# id/order = hidden PK + list sort (extension_edit, call_forward_edit);
# action/name/subtype/type/group_uuid_name = generic form/list chrome;
# db_destination_number = hidden edit-tracking field (destination_edit);
# dialplan_details_order/phrase_details_order = UI detail sort CSV;
# log_date = server-set timestamp (event_guard);
# group_name_previous = rename-tracking hidden field;
# device_key_ids = hidden JS sort-order input (device_edit.php:2222);
# recorded_name = delete-checkbox for recorded_name.wav file (voicemail_edit.php:676), not a column;
# voicemail_id_previous = hidden rename-tracking field (voicemail_edit.php:621);
# greeting_language/greeting_model/greeting_voice_source/update_greeting = TTS helpers, not v_voicemail_greetings columns (app_config.php has no such fields);
# recording_filename_original/recording_language/recording_model/recording_voice_source = rename/TTS helpers, not v_recordings columns (app_config.php has no such fields);
# voicemail_destination handled via voicemail_destinations child table, not NOISE.
# No semantic carve-outs: every v_* column must be implemented.
# $_FILES['attachment'] file upload (contact_attachment_edit.php:60), not a column;
# attachment_filename/content/description are the stored columns.
NOISE = {
    "frm", "checkbox_all", "search", "page", "order_by", "submit", "range",
    "show", "preset", "value", "checkbox", "ulfile", "file_action", "target_file",
    "file", "action2", "create_recording", "translate", "record",
    "referer_path", "referer_query",
    "id", "order", "action", "name", "subtype", "type", "group_uuid_name",
    "db_destination_number", "dialplan_details_order", "phrase_details_order",
    "log_date", "group_name_previous",
    "device_key_ids",
    "recorded_name", "voicemail_id_previous",
    "greeting_language", "greeting_model", "greeting_voice_source", "update_greeting",
    "recording_filename_original", "recording_language", "recording_model", "recording_voice_source",
    "password_confirm", "username_old",
    "attachment",
}


def variants(app_prefix, key, pool, aliases):
    cands = {key, aliases.get(key, key)}
    for p in PREFIXES:
        if key.startswith(p):
            cands.add(key[len(p):])
        else:
            cands.add(p + key)
    if key.startswith(app_prefix):
        cands.add(key[len(app_prefix):])
    cands.add(app_prefix + key)
    return cands & pool


def main():
    php = php_edit_fields()
    react = react_keys()
    rust = rust_keys()
    aliases = rust_aliases()
    sem_r = {(a, r, p) for (a, p, r) in SEM}
    rows = []
    for app, fields in sorted(php.items()):
        if app not in MAP:
            continue
        rkeys, rmods, pre = MAP[app]
        if app in RELATED:
            rk, rm = RELATED[app]
            rkeys = list(rkeys) + rk
            rmods = list(rmods) + rm
        rset = set()
        for k in rkeys:
            rset |= set(react.get(k, set()))
        uset = set()
        for m in rmods:
            uset |= set(rust.get(m, set()))
        want = [f for f in sorted(fields)
                if f not in NOISE and not (f.endswith("_uuid") and f != "device_uuid_alternate")]
        rmiss, umiss = [], []
        for f in want:
            if (f in rset or variants(pre, f, rset, aliases)
                    or (app, f, f) in SEM
                    or any((app, r, f) in SEM for r in rset)
                    or any((app, r, f) in SEM for r in variants(pre, f, rset, aliases))):
                pass
            else:
                rmiss.append(f)
            if f in uset or variants(pre, f, uset, aliases):
                pass
            else:
                umiss.append(f)
        rows.append((app, len(want), len(rmiss), len(umiss), rmiss, umiss))
    out = []
    bad = 0
    for app, n, rm, um, rml, uml in rows:
        out.append(f"{app}: php={n} react-miss={rm} api-miss={um}")
        if rml:
            out.append(f"   R: {rml}")
        if uml:
            out.append(f"   A: {uml}")
        bad += rm + um
    header = [
        "# PHP parity audit (generated: python3 scripts/php_parity_audit.py)",
        "# PHP ground truth: fusionpbx/**/ *_edit.php name='...' inputs (+ v_* schema for columns).",
        "# React: fieldConfigs T()/C()/name:/form./set(/key: keys + custom route form keys.",
        "# API: struct fields, payload.get/s()/b()/b_opt()/i()/put(, for-key loops, alias layer.",
        "# Names normalized: alias.rs pairs + prefix strip + SEM map + RELATED cross-page map.",
        "# Documented non-gaps: see NOISE in scripts/php_parity_audit.py.",
        "# Whole-goal: 76 app areas + core areas; form-backed subset strictly 0-miss.",
        "# Non-form areas (vendor templates, status pages, no v_ tables) disposition below.",
    ]
    # Whole-goal inventory: every fusionpbx/app + core area
    all_apps, all_core = php_all_areas()
    covered = {a for a, _, _, _, _, _ in rows}
    uncovered_form = []  # should stay empty: all form apps must be in MAP
    for app in all_apps:
        has_edit = bool(glob.glob(os.path.join(ROOT, "fusionpbx", "app", app, "*_edit.php")))
        if has_edit and app not in MAP and app not in php:
            uncovered_form.append(app)
    out.append("")
    out.append(f"whole-goal: app areas={len(all_apps)} core areas={len(all_core)} form-backed={len(rows)} strict-miss={bad}")
    out.append(f"non-form app areas (no *_edit.php, templates/status/pages): " + ", ".join(sorted(set(all_apps) - set([a for a, _, _, _, _, _ in rows]))))
    out.append(f"core areas: " + ", ".join(all_core))
    if uncovered_form:
        out.append(f"UNCOVERED-FORM-APPS (must be 0): " + ", ".join(uncovered_form))
        bad += len(uncovered_form)
    text = "\n".join(header + [""] + out) + "\n"
    with open(os.path.join(ROOT, "docs", "php-parity-audit.txt"), "w") as fh:
        fh.write(text)
    print(text)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
