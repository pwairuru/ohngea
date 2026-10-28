# Batch B report — live callcenter, broadcast exec, call/conf controls

Date: 2026-10-28. TDD: stub `rust/crates/pbx-api/tests/batchb_test.rs` read first,
ran RED (5 failed, all 404), implemented GREEN, verified.

## Scope delivered (all 5 stub tests pass)

1. **call_center_active live view** — `GET /api/v1/call_center_active`
   (`rust/crates/pbx-api/src/call_centers.rs`). Tenant DB queues + agents snapshot,
   ESL `callcenter_config queue list` poll; `source: esl|db` (`db` on mock
   fallback). Perm: `call_center_active_view`. Contract: `{data, total, source}`.
2. **Broadcast send/stop** — `POST /api/v1/call_broadcasts/{id}/send|/stop`
   (+ legacy `/call_broadcast` aliases) (`remaining_a.rs`). Parity with
   `app/call_broadcast/router.py::send_ep/stop_ep`: send enables row, bgapi-originates
   to each number (cap 100, 300-char replies), returns `send: {attempted, results}`;
   stop disables row, returns `broadcast_enabled: false` + `send.stopped`.
   Perm `call_broadcast_add` both. Also: create/ser/patch now persist
   `broadcast_phone_numbers`, `broadcast_destination_type/data` (send needs them).
3. **Active calls** — `GET /api/v1/active_calls/{id}` (auth-only, mirrors Python
   `uuid_exists` against shared mock store; 404 when absent),
   `POST /api/v1/active_calls/{id}/hangup` (perm `active_calls_hangup`,
   `uuid_kill` + mock drop so later GET 404s) (`remaining.rs`).
4. **Active conferences** — `GET /api/v1/active_conferences/count` (`{count}`),
   `GET /api/v1/active_conferences/{conf}/members` (`{conference, data, total}`,
   best-effort `xml_list` parse, empty offline),
   `POST .../members/{member}/{action}` (perm `conference_active_view` all three;
   PHP `conference_exec.php` action set: mute/unmute/deaf/undeaf/kick/lock/unlock/
   energy/volume[_in|_out]/record/norecord; unknown → 422) (`remaining.rs`).
5. **conference_controls + conference_profiles** — new SeaORM entities
   (`pbx-db/src/entities/conference_{controls,profiles}.rs`, Python-model columns),
   sqlite DDL + postgres migration, full CRUD routers
   (`pbx-api/src/conference_controls.rs`): reads gated `*_view`, writes `*_add`,
   tenant-scoped, paged lists. Detail/param child tables deferred (no test/route
   needs them yet).

## Permission gates (batch-A pattern)

Every new route carries `deny_if_missing` gates (`tenant.rs`) + `domain_id`
filters, except `GET /active_calls/{id}` which mirrors Python's auth-only
`get_call`. Test JWT (`admin` perm) passes all gates.

## Verify evidence

- `cargo test -p pbx-api -p pbx-fs`: **113 passed (22 suites)** incl. batchb 5/5.
- `cargo fmt --all -- --check`: clean (also fixed pre-existing drift in
  `pbx-fs/src/esl.rs`).
- `cargo clippy -p pbx-api -p pbx-fs -p pbx-db --all-targets -- -D warnings`:
  no issues.
- Built/tested only `pbx-api`, `pbx-fs`, `pbx-db` (no workspace build).

## Ground truth refs

- `fusionpbx/app/conferences_active/conference_exec.php` (action set),
- `fusionpbx/app/calls_active/` (hangup), `app/active_calls/router.py`,
  `app/call_broadcast/router.py`, `app/conference_{controls,profiles}/router.py`.
- Python has no `call_center_active` router; PHP `call_center_active_inc.php`
  uses `callcenter_config` ESL — matched.

## Follow-ups (not in stub contract)

- `conference_control_details` / `conference_profile_params` child CRUD.
- Live `xml_list` member parsing is minimal (uuid only); enrich when FS present.
- `source: esl` path untested (no live FS in CI) — DB snapshot covers contract.
