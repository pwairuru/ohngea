# Task 7 Report — Rust+FS compose overlay + live-gated test + ESL loudness

**Status: DONE.** Overlay + domain-qualified originate + live channels +
real `uuid_kill` + ignored live test implemented. Required suites green:
`fanout_test` 6/6, `telephony_test` 9/9, `pbx-fs` all suites green,
`e2e_call_test` (mock flow) green. Live test skipped by default, fails
loudly with `--ignored` when FS absent. Tasks 1–6 commits untouched.

## Scope (only these files)
- `rust/docker-compose.fs.yml` — NEW overlay: `freeswitch` (real, pinned
  image, stock `docker/freeswitch/conf`) + `rust-api` with `api` network
  alias (no `gateway-url`/CDR URL retargeting), Postgres via base `db`,
  `FS_ESL_HOST=freeswitch:8021`, `FS_XML_USER/PASSWORD`, host
  `${RUST_API_PORT:-8001}:8000` (no 8000/host collision with base
  `api ${API_PORT:-8010}:8000`), FS ESL host `:8022`.
- `rust/crates/pbx-fs/src/esl.rs` — `originate_command(from,to,domain)`
  (`user/from@domain &bridge(user/to@domain)`, `@`-legs untouched),
  `RealEs::originate_with_domain` (live returns the REAL FS uuid parsed
  from `+OK <uuid>` via `parse_originate_uuid`; down falls back to mock
  with qualified legs), `parse_show_channels_json` (tolerant
  `rows`/`data`/bare-array → `{uuid,from,to,state}`), `live_channels`
  (`Some` incl. empty when FS answers, `None` when down), `kill` uses
  `uuid_kill <fs-uuid>` for real ids (honest `+OK` check, `false` when FS
  unreachable) and the mock only for `call-…` ids.
- `rust/crates/pbx-api/src/calls.rs` — threads tenant domain:
  optional `domain` field, else `domains.domain_name` via
  `ring_groups::resolve_domain_name` (fallback claims display name);
  `GET channels` serves live `show channels as json` when FS answers,
  else mock (same `{data,total}` shape).
- `rust/crates/pbx-api/tests/live_fs_ignored_test.rs` — NEW, single
  `#[ignore]` test: ESL reachable (loud fail otherwise) → seed 1001/1002
  → Basic directory fetch 1001@default (REGISTER path) → dialplan fetch →
  `sofia status` RUNNING → originate 1001→1002 → channels contains call →
  kill → CDR `NORMAL_CLEARING` billsec≥1 poll.
- `rust/README.md` — overlay usage + live-test command documented.

NOT touched: directory/dialplan renderers, config builders, auth, domains,
fan-out (`remaining.rs` click-to-call keeps legacy `originate(from,to)`).

## Behavior notes
- `originate` response shape unchanged (`{call_id,from,to}`, 201); live
  `call_id` is now the real FS uuid (was synthetic `real-…`), so DELETE
  `uuid_kill`s a real id. Mock ids (`call-…`) still kill from the mock.
- `channels` shape unchanged; live rows map `{uuid→call_id, cid→from,
  dest→to, state}`. Unit-test envs (FS down) still see the mock.
- `kill` on a real id with FS down now 404s (loud) instead of fake-204 —
  no existing test covers that path (all mock ids).

## Tests (exact commands, fresh runs)
- `cargo test -p pbx-api --test fanout_test --test telephony_test` —
  6/6 + 9/9 pass.
- `cargo test -p pbx-fs` — all suites pass (config/esl_body/esl_live/xml).
- `cargo test -p pbx-api --test e2e_call_test` — mock flow passes
  (domain-threaded originate/kill intact).
- `cargo test -p pbx-api --test live_fs_ignored_test` — 1 ignored (default).
- `cargo test -p pbx-api --test live_fs_ignored_test -- --ignored` —
  FAILS loudly with FS down (`live FS required: ESL 127.0.0.1:8021
  unreachable (start rust/docker-compose.fs.yml)`); passes only with real
  8021 (no live FS in this env to green it — same constraint as the plan's
  step 2; helpers verified via throwaway unit run, then removed).
- `docker compose -f docker-compose.yml -f rust/docker-compose.fs.yml config`
  validates: `rust-api` context=root, `api` alias, `8001→8000`, FS `8022→8021`.

## Commit note
Staged ONLY the 5 Task-7 paths above (plus this report). The working tree
was already dirty at session start (app/, alembic/, frontend/, pbx-db…);
none of that was staged. Tasks 1–6 commits (`52e5081`…`7deebae`,
`fee7910`) are intact ancestors.
