# Field Parity — Rust vs Python (2026-11-16)

Ground truth: `app/*/models.py` + `app/*/router.py` vs `rust/crates/pbx-api/src/*.rs` + `rust/crates/pbx-db/src/entities/*`.

## Router parity (62 Python routers)

OK (Rust router + mount, same `/api/v1/*`): auth, domains, extensions, dialplans, gateways,
sip_profiles+sip_status, freeswitch/xml+status, ivr_menus, ring_groups, voicemails+voicemail alias,
conferences, conference_controls+profiles, conference_centers, fax+fax_queue, cdr, users/groups/
permissions/contacts, devices+device_profiles+provision, email_queue, call_centers+call_center_active,
vars, number_translations/phrases/streams/pin_numbers/music_on_hold/feature_codes,
call_block/destinations/follow_me/call_forward/time_conditions/bridges/broadcasts,
access_controls/fifo/fifo_list/emergency_logs/event_guard, active_calls/active_conferences/
registrations/operator_panel/click_to_call/modules/log_viewer/database_transactions,
logs, status_aliases redirects, system, plugins/example/domain.

FIXED 2026-11-16 (`rust/crates/pbx-api/src/admin.rs`, `gaps3_test.rs` 2/2 pass):
- `GET /api/v1/domain_limits` — counts view (extensions/dialplans/gateways/devices/voicemails + limit lookup)
- `CRUD /api/v1/extension_settings` — new entity `extension_settings.rs` (7 cols parity)
- `CRUD /api/v1/sofia_global_settings` — existing entity, global no-domain
- `CRUD /api/v1/call_recordings` — existing entity
- `CRUD /api/v1/call_flows` — new entity `call_flows.rs` (19 cols parity + extension validator)
- `GET /api/v1/webrtc-settings` hyphen alias — same shape as underscore

PARTIAL (mounted but subset/wrong path):
- `admin_settings`: only `default_settings` in `settings.rs:316`; MISSING `domain_settings, email_templates, menu_items, user_logs, notifications, services`
- `voicemail_greetings`: only `POST {id}/greetings` in `voicemails.rs:960`; full CRUD missing
- `webrtc_settings`: underscore + `/webrtc/credentials` exist; hyphen canonical + legacy `/api/v1/domains/{id}/webrtc_settings` 307 missing
- `call_recordings` claimed in `recordings.rs:1` but only `/recordings` mounted — now fixed via `admin.rs`
- `provision`: `GET /provision/{mac}` exists `devices.rs:550`; frontend calls `/provision/{mac}.cfg` bypassing `API_BASE` — needs `.cfg` alias
- `legacy /auth/*` 307: Rust `auth.rs:577` already serves both `/auth/*` + `/api/v1/auth/*`

MISSING (no Rust route, only `_links` stub in `lib.rs:158`): none after gap3 — all 60 Python prefixes now served.
Frontend-only 404s still open (no backend): `domain_settings, email_templates, menu_items, user_logs, notifications, services, call_flows UI needs backend now OK`.

Middleware parity (`app/main.py:44-56` CORS→RateLimit→Audit→AppGate→Tenant):
- Tenant OK (per-router `require_tenant`, `tenant.rs:207`)
- AppGate PARTIAL (only plugin gate `plugins.rs:307`, general app disable unenforced)
- Audit MISSING (`app/middleware/audit.py` X-Request-ID/duration)
- RateLimit MISSING (`app/middleware/rate_limit.py`)
- CORS MISSING (no `CorsLayer`, `main.rs:1-32` bare serve)

Auth parity: bcrypt OK (`pbx-core/src/auth.rs:123`), debug-only stub `auth.rs:166` (release 401), MFA + password-reset are Rust superset.

## Field parity (entities)

OK (0 missing): extensions, gateways, devices+lines/settings/keys/profiles, dialplans+details (rename `type→detail_type` intentional), sip_profiles+domains+settings, groups, user_settings.

THIN Rust entities (need expansion):
- `ivr_menus` 23 missing: `greet_long,short,invalid_sound,exit_sound,confirm_macro,key,confirm_attempts,tts_engine,voice,timeout,inter_digit_timeout,max_failures,max_timeouts,digit_len,dialplan_uuid,parent_uuid,language,dialect,voice,pin_number,ringback,cid_prefix,context,direct_dial,exit_app,exit_data` — `app/ivr_menus/models.py:10-44` vs `entities/ivr_menus.rs:11-18`
- `voicemails` 16 missing: `greeting_id,toll_allow,voicemail_mail_to,file,alternate_greet_id,tutorial,local_after_email,transcription_enabled,sms_to,transcription_prompt,recording_instructions,attach_file,forward_enabled,destinations,sms_type,keep_local_after_email`
- `ring_groups` 21 missing: `timeout,forward_enabled/destination,greeting,exit_key,call_timeout,caller_id_name/number,cid_name/number_prefix,timeout_app/data,distinctive_ring,ringback,call_screen/call_forward/follow_me_enabled,missed_call_app/data,context,dialplan_uuid,forward_toll_allow`
- `users` 4 missing: `contact_uuid,user_status,user_type,api_key`
- `voicemail_messages` 8-9, `voicemail_greetings` 7, `voicemail_options.order`, `ring_group_members.destination_enabled/description`, `user_groups.group_name`

Docs claim (`docs/field-parity.md:4 COMPLETE`) describes Python/alembic 021-028 state, NOT `rust/.../entities/*` — corrected here.

## Frontend (`frontend/src/routes/` 85 .tsx)

Routes with working Rust backend: extensions, dialplans, gateways, sip_profiles, ivr, ring_groups, time_conditions, voicemails, conferences, fax, call_centers, devices, groups, users, domains, permissions, contacts, email_queue, vars, analytics, system, logs, plugins, backups, cdr, calls, call_block/broadcast, bridges, destinations, follow_me, call_forward, feature_codes, moh, phrases, pin_numbers, number_translations, streams, fifo, access_controls, conference_controls/profiles/centers, emergency_logs, event_guard, database_transactions, active_calls/conferences, registrations, operator_panel, click_to_call, modules, auth.

Calling MISSING backend (404): `domain_settings, email_templates, menu_items, user_logs, notifications, services, extension_settings (now fixed), sofia_global_settings (now fixed), domain_limits (now fixed), call_recordings (now fixed)`.
Form gaps: Extensions omits password, Gateways extras dropped, Devices omits domain/template, Dialplans PATCH drops details, Users omits domain scoping.
Client gaps: `API_BASE=/api/v1` hardcoded, auth cookie ignored, no 401→refresh, no pagination headers, `provision/{mac}.cfg` bypasses base, `webrtc/domains/{id}` 404 path.
Tests: only 11/75 route unit tests, `routes-coverage.test.tsx` mocks `apiFetch` (masks 404s).

## Plugins (`plugins/` 9)

Registry parity OK (all 9 in `plugins.rs` manifests). Functional: only `example` has `.wasm` guest + route; other 8 served by core routers by design. `time_conditions` has no Rust module (health-stub both sides). Sandbox stubs: `db_get` allowlist `[1,2,3]`, no PDK, no cache.

## Tests

- `rust/crates/pbx-api/tests/` 15 files; `pbx-db/tests/` 4; `pbx-fs/tests/` 2 trivial (sofia/cache/esl/xml uncovered); `pbx-core` 0 tests dir; `rust/tests/` 2 shells; `tests/` pytest 39 files (broadest); `frontend/tests/` 13 files (bulk `comprehensive.spec.ts:571`).
- `gaps3_test.rs` 2/2 GREEN 2026-11-16 (this session).

## Compose

`docker-compose.yml` full stack (db/valkey/freeswitch-mock/minio/api/frontend/mailpit/worker).
`rust/docker-compose.rust.yml` overlay only (`rust-api :8000`, `JWT_SECRET=test-secret`, missing VALKEY/S3/ESL/SMTP, port clash, no healthcheck/frontend entry).
Legacy `fusionpbx/` 266M 1248 `.php` gitignored — reference only.
