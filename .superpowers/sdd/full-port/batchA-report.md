# Batch A report — permissions enforcement + tenant scoping + settings admin (2026-10-28)

Scope: audit gaps #1 (permissions hole), #2 (tenant scoping), #10-part (default_settings admin + user_settings; MFA/reset deferred).
TDD: RED established first (broken `cdr.rs` test module → compile fail; `batchA_test` 404s on missing
`/api/v1/default_settings`; 200-instead-of-403 on users/cdr/groups/domains), then GREEN.

## 1. Permission enforcement (gap #1)

- `tenant.rs`: existing `deny_if_missing` kept; added `require_permission` alias (Python `app/auth/deps.py` name parity).
  403 body parity: `{"detail": "Missing permission: {perm}"}`. `admin`/`superadmin`/`*` bypass.
- Wired `deny_if_missing` into 27 routers (~230 handlers): extensions (pre-existing), users, groups,
  permissions (self-gated on `group_permission_view`), domains, contacts, devices (+keys/lines/settings/profiles),
  cdr, dialplans, gateways, sip (+domain/setting sub-resources), vars_fs, fax (+fax_queue), recordings,
  ring_groups, call_centers, ivr_menus, voicemails, conferences, conference_centers (+rooms/sessions),
  email_queue, remaining_a/b/c spillover (call_block, broadcast, bridges, destinations, follow_me, call_forward,
  time_conditions, feature_codes, music_on_hold, phrases, pins, number_translations, streams, access_controls,
  fifo (+members), webrtc, emergency_logs, event_guard), new settings router.
- Python parity notes: reads→`*_view`, writes→`*_add`, patch→`*_edit`/delete→`*_delete` where Python defines them;
  coarse modules (fax, voicemail, call_center, conference, remaining_a) use `*_add` for writes like Python;
  reads that are auth-only in Python are gated on new `*_view` names (tightening, documented in code).
- `permissions.rs` CATALOG extended 24 → ~140 names; lazy-seeded, includes `cdr_add`, `domain_edit/delete`,
  `device_edit/delete`, all spillover families. Full 1000+ PHP catalog remains a follow-up.
- Explicitly NOT gated (documented in code): public provision/bootstrap/auth/health; live ESL state
  (active_calls/conferences, registrations, operator_panel actions, fifo callers, click_to_call) — global
  switch state, no per-domain row; per-action perm gating is batch-B follow-up.

## 2. Tenant scoping (gap #2)

- Verified: all DB-backed list/get/patch/delete filter `domain_id` (extensions, users, groups, cdr, contacts,
  devices, dialplans, ...). Fixed real hole: `groups` bulk `toggle` ignored tenant (now scoped + tested).
- `remaining.rs:415-439` aliases are 307 redirects with no DB query; documented that they inherit target
  scoping + `require_tenant` 401. `event_guard_logs` is global by design (no `domain_id`, Python parity)
  but now requires claims + `event_guard_*` perms (was claim-less).
- Tests: cross-domain extensions isolated (pre-existing), user create + settings GET cross-domain → 404 (new),
  cdr create-A/get-B → 404 (new), groups bulk toggle cross-tenant → per-row `not found` (new).

## 3. Settings admin (gap #10, minus MFA/reset)

- New `settings.rs`: `default_settings` full CRUD (global table, `?_q` search, pagination, required
  category+name → 422, alias keys accepted) gated on `default_setting_{view,add,edit,delete}`.
- `users.rs`: `GET/PUT /api/v1/users/{id}/settings` (Python `app/users/router.py` parity: replace-all PUT,
  self-bypass, else `user_setting_view`/`user_edit`; GET tightened to self-or-view, documented).
- `pbx-db`: exported `entities::user_settings`, added `user_settings` DDL to sqlite `create_schema` and
  postgres migration (+ down-migration drop list).
- Auth hardening deferred with TODO in `auth.rs`: MFA/OTP, password-reset, LDAP.
- Bonus fix: `cdr.rs` unit tests referenced unimplemented `build_cdr_csv`/`extension_summary`/`CdrExportRow`
  (compile break) — implemented minimal versions (header+rows CSV, per-extension count+billsec summary)
  in `cdr.rs`, re-exported from lib root. Full ~50-col CDR parity stays out of scope.

## Verification (fresh, this session)

- `cargo test -p pbx-api --no-run`: 0 errors. `cargo fmt -p pbx-api -p pbx-db`: clean.
  `cargo clippy -p pbx-api -p pbx-db --tests`: No issues found.
- Suites, each run individually: lib 8 ✓, batchA 11 ✓ (5 pre-existing + 6 new), auth_shape 4 ✓, e2e 1 ✓,
  health 1 ✓, plugins 5 ✓, plugins_sandbox 7 ✓, provision 2 ✓, ui_contract 10 ✓, gaps2 11 ✓, routing 10 ✓,
  remaining 15 ✓, system 10 ✓, tenant 3 ✓, telephony 5 ✓ (3× repeat). `cargo test -p pbx-db`: 26 ✓.
- Known RED (out of scope, untouched): `batchb_test` 5 failures — batch-B routes (call_center_active live,
  conference_controls/profiles, per-call hangup, broadcast send/stop) return 404, unimplemented.
- Flake note (pre-existing, untouched code path): `telephony_test::vars_nonempty_after_boot_seed` fails when
  several test binaries run in one `cargo test` invocation (`seed_domains` TOCTOU UNIQUE conflict → seed
  aborts → vars empty); passes consistently alone. `seeds.rs`/`routing_db.rs` not touched by this batch.

## Files (this batch only)

- `rust/crates/pbx-api/src/{tenant,permissions,settings(new),users,groups,domains,contacts,devices,cdr,dialplans,gateways,sip,vars_fs,fax,recordings,ring_groups,call_centers,ivr_menus,voicemails,conferences,conference_centers,email_queue,remaining_a,remaining_b,remaining_c,remaining,auth,lib}.rs`
- `rust/crates/pbx-db/src/{entities.rs,lib.rs,migration.rs}` (user_settings entity export + DDL)
- `rust/crates/pbx-api/tests/batchA_test.rs` (+6 tests)
