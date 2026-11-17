# Task 6 Report — App→dialplan fan-out + tones

**Status: DONE.** All 6 new `fanout_test.rs` tests pass, `telephony_test` (9)
passes, full `cargo test -p pbx-api` green (0 failures), `pbx-fs` green.
Tasks 1–5 commits (52e5081, 4fe0556, 3e011ef, e22f8e3, cc224d8) untouched.

## Scope (only these files)
- `rust/crates/pbx-api/src/ring_groups.rs` — shared `sync_app_dialplan` /
  `delete_app_dialplan` / `resolve_domain_name` helpers + RG spec + hooks.
- `rust/crates/pbx-api/src/ivr_menus.rs` — IVR spec + hooks (menu CRUD only;
  options affect `ivr.conf` from Task 5, dialplan untouched by them).
- `rust/crates/pbx-api/src/conferences.rs` — conference spec + hooks.
- `rust/crates/pbx-api/src/call_centers.rs` — queue spec + hooks (fan-out
  only; no `callcenter.conf` builder changes).
- `rust/crates/pbx-api/src/voicemails.rs` — `invalidate_cache()` on
  create/patch/delete (join already done in Task 3).
- `rust/crates/pbx-api/src/remaining.rs` — `GET /api/v1/tones` (+ `/`).
- `rust/crates/pbx-api/tests/fanout_test.rs` — new, 6 tests.

NOT touched: `fs_xml.rs` renderer, `config.rs`, auth, domains, `dialplans.rs`.

## Fan-out contract (PHP-exact)
| app | app_uuid | order | glue |
|---|---|---|---|
| ring group | `1d61fb65-…` | 101 | `lua app.lua ring_groups` (+ring_ready, ring_group_uuid, record_stereo) |
| IVR | `a5788e9b-…` | 101 | `lua ivr_menu.lua` (+ring_ready, sleep, hangup_after_bridge, ivr_menu_uuid) |
| conference | `b81412e8-…` | **333** | `conference ext@domain@profile+pin+flags{…}` (+answer, inline sets) |
| call center | `95788e50-…` | 230 | `callcenter ext@domain` (limit/caller-id groups, sound_prefix via `$${sounds_dir}`) |

Writes: upsert `dialplans` row (PHP-format `dialplan_xml` stored too) +
replace `dialplan_details`; delete removes both; every write ends with
`invalidate_cache()` + `reloadxml` (loud ESL, non-fatal — Task 5 pattern).
Create/patch mint a `dialplan_uuid` when the row link is absent (PHP parity)
and persist it back; delete drops the linked row.

## Deviations from the task shorthand (all PHP-grounded)
- Conference order is **333**, not 101 (`conference_edit.php:250`); RG/IVR
  are 101. Used PHP values.
- Extension-less IVR menus / queues skip the dialplan sync (an empty
  `^$`/`^(callcenter\+)?$` condition would be junk); link stays empty.
- `cc_exit_keys` emitted when `Some` (PHP `!== null`); timeout app skipped
  (no `timeout_data` field in our model).
- Sync DB errors return loud 500 (row may exist without dialplan), never
  silent; ESL `reloadxml` failures log + continue (DB already committed).

## Tests
- TDD: `fanout_test.rs` written first — 5/6 failed pre-impl (red), all pass
  post-impl: create→mint (app_uuid/order/glue asserted on the row AND on
  served `/api/internal/freeswitch/xml?section=dialplan`), update→re-point,
  delete→row 404; tones round-trip via seeded `Tones` var; voicemail CRUD.
- `cargo test -p pbx-api --test telephony_test` — 9/9 ok.
- Full `cargo test -p pbx-api` — 0 failures. `cargo test -p pbx-fs` — ok.
- Pre-existing, out of scope: `cargo test -p pbx-db` fails to compile
  (`SofiaProfileInput` missing fields in `pbx-db/tests`) — untouched crate,
  cannot be caused by this change (pbx-db does not depend on pbx-api).

## Commit note
`5fd9f32` stages whole files, so it also captures pre-existing uncommitted
working-tree edits to these same 7 source files (the tree was already dirty
at session start — e.g. `dialplans.rs`/`lib.rs` remain dirty and were NOT
staged). Tasks 1–5 commits are intact ancestors; nothing outside the Task 6
file scope was staged. Tests above ran on the exact committed content.
