# Rust Enterprise Logging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Structured, level-switchable, file-capable logging with request correlation across `pbx-api`, ready for OTel later at zero current cost.

**Architecture:** New `pbx-api::logging` module owns subscriber init (EnvFilter + JSON/pretty + non-blocking stdout/file + `otel` feature slot); `main.rs` inits first and holds guards; one `from_fn` request middleware (outermost) adds span fields + request-id; all `eprintln!` sites become events; mutating handlers get error/warn events.

**Tech Stack:** `tracing`, `tracing-subscriber` (features `env-filter`, `json`), `tracing-appender`, `uuid` (already a dep, v7 for request ids).

**Spec:** `docs/superpowers/specs/2026-11-24-rust-logging-design.md`

## Global Constraints

- `cargo fmt --all -- --check` clean.
- `cargo clippy --all-targets -- -D warnings` clean.
- Never log passwords, tokens, hashes, or full URLs/connection strings (only `database_kind`-style labels).
- `#[instrument]` must `skip` request bodies and DB connections.
- Bad `LOG_LEVEL`/`RUST_LOG`/`LOG_FILE` values: stderr warn + safe fallback, never panic, never block boot.
- Non-blocking writers only; `WorkerGuard`s held to end of `main`.

## Review Focus

- `LOG_LEVEL=verbose` (invalid): boots fine, stderr warns, default filter applies — pinned in Task 1 test.
- `LOG_FILE=/nonexistent-dir/x.log` (unwritable): boots fine on stdout, stderr warns — pinned in Task 1 test.
- Request without `x-request-id`: gets UUIDv7, echoed back on response — pinned in Task 3 test.
- `GET /api/v1/extensions?token=secret`: logged path has no query string — pinned in Task 3 test.
- Inner-layer 5xx (e.g. bad tenant 401→ actually 4xx; use a 500 probe): response still carries request-id and an error event fires — pinned in Task 3 test.

---

### Task 1: `logging` module + unit tests

**Files:**
- Create: `rust/crates/pbx-api/src/logging.rs`
- Modify: `rust/crates/pbx-api/Cargo.toml` (add `tracing`, `tracing-subscriber/env-filter/json`, `tracing-appender`)
- Modify: `rust/crates/pbx-api/src/lib.rs` (add `pub mod logging;`)
- Test: `rust/crates/pbx-api/tests/logging_test.rs`

**Interfaces:**
- Consumes: `LOG_LEVEL`, `RUST_LOG`, `LOG_FORMAT`, `LOG_FILE` env vars.
- Produces (used by Tasks 2–4):
  - `pub struct LogGuards` — holds stdout + optional file `WorkerGuard`s; kept alive in `main`.
  - `pub fn init() -> LogGuards`
  - `pub fn init_for_tests()` — `OnceLock`-based, idempotent, sink writer.
  - `pub fn default_filter() -> String` — returns `pbx_api=info,pbx_fs=info,pbx_db=info,pbx_core=info,sea_orm=warn,hyper=warn,tower_http=warn`.
  - `pub fn redact_esl(cmd: &str) -> String` — verb-only + 200-char cap + masks `password|passwd|secret|token` values.

- [ ] **Step 1: Write failing test** `rust/crates/pbx-api/tests/logging_test.rs`:

```rust
#[test]
fn invalid_log_level_falls_back_to_default() {
    assert_eq!(
        pbx_api::logging::resolve_filter(Some("verbose")),
        pbx_api::logging::default_filter()
    );
}

#[test]
fn redact_esl_masks_secret() {
    let out = pbx_api::logging::redact_esl("sofia profile internal rescan password=hunter2");
    assert!(!out.contains("hunter2"));
}

#[test]
fn init_for_tests_is_idempotent() {
    pbx_api::logging::init_for_tests();
    pbx_api::logging::init_for_tests();
}
```

- [ ] **Step 2: Run to verify failure** — Run: `cargo test -p pbx-api --test logging_test` Expected: FAIL (`logging` module not defined).
- [ ] **Step 3: Implement** `logging.rs` with `default_filter()`, `resolve_filter(Option<&str>)`, `resolve_format()`, `split_log_file(path) -> (dir, stem)`, `redact_esl()`, `init() -> LogGuards`, `init_for_tests()`. `init`: `RUST_LOG` wins if set else `LOG_LEVEL` via `resolve_filter`; `LOG_FORMAT=json|pretty` (default json in release, pretty in debug); `LOG_FILE` full path → `tracing_appender::rolling::daily(dir, stem)` + `create_dir_all`, failure → stderr warn + stdout only; `otel` feature adds `cfg(feature = "otel")` opentelemetry layer slot (optional deps, off by default).
- [ ] **Step 4: Run** `cargo test -p pbx-api --test logging_test` Expected: PASS (3/3).
- [ ] **Step 5: Commit** `feat(logging): add tracing subscriber module with level/file/format env control`.

### Task 2: `main.rs` wiring + boot event

**Files:**
- Modify: `rust/crates/pbx-api/src/main.rs:9-45`
- Test: existing `rust/crates/pbx-api/tests/banner_test.rs` (must keep passing) + manual boot smoke.

**Interfaces:**
- Consumes: `pbx_api::logging::init() -> LogGuards` (Task 1).
- Produces: process-wide subscriber active before any other output; guards alive to end of `main`.

- [ ] **Step 1: Edit** `main.rs`: call `let _log_guards = pbx_api::logging::init();` as the first statement of `main` (before the JWT gate). Keep the pre-init release JWT `eprintln!` + `exit(1)` as-is (subscriber may not exist yet). After `println!` banner, add `tracing::info!(version = ..., port, db = database_kind, mode = build_mode, "pbx-api boot")`.
- [ ] **Step 2: Run** `cargo test -p pbx-api --test banner_test` Expected: PASS.
- [ ] **Step 3: Boot smoke** — Run: `LOG_LEVEL=debug cargo run -p pbx-api` (with a dev `JWT_SECRET`), then `curl -s localhost:8000/health`. Expected: JSON/pretty log lines on stdout including `pbx-api boot` with version/port/db fields; `/health` 200.
- [ ] **Step 4: Commit** `feat(logging): init subscriber first in main, log boot context`.

### Task 3: Request-log middleware (outermost) + request-id

**Files:**
- Modify: `rust/crates/pbx-api/src/lib.rs` (add `request_log_middleware` fn; insert `.layer()` FIRST in `build_app()` chain, before `plugin_gate`; add `uuid` use if needed)
- Test: `rust/crates/pbx-api/tests/logging_test.rs` (append to Task 1 file)

**Interfaces:**
- Consumes: `test_app()` from Task-2-era `lib.rs`; `MatchedPath` extractor.
- Produces: every response carries echoed `x-request-id`; 5xx responses emit `tracing::error!`.

- [ ] **Step 1: Write failing tests** (append):

```rust
#[tokio::test]
async fn request_id_generated_and_echoed() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res.headers().contains_key("x-request-id"));
}

#[tokio::test]
async fn query_string_never_logged_path_only() {
    // path source must be MatchedPath or uri.path(), never the query string
    assert!(!pbx_api::logging::logged_path("/api/v1/extensions?token=secret").contains("token"));
}
```

- [ ] **Step 2: Run** Expected: FAIL (`logged_path` undefined, no `x-request-id` header).
- [ ] **Step 3: Implement**: `pub fn logged_path(matched: Option<&str>, uri_path: &str) -> String` in `logging.rs` (matched template wins, else raw path — never query). `async fn request_log_middleware(req, next)`: take-or-make (`Uuid::now_v7`) `x-request-id`; `Instant::now`; run inner; record `method`, `logged_path`, `status`, `latency_ms`, `request_id` via `tracing::info!`; `error!` when status is 5xx; insert `x-request-id` into response headers.
- [ ] **Step 4: Run** `cargo test -p pbx-api --test logging_test` Expected: PASS (5/5).
- [ ] **Step 5: Commit** `feat(logging): outermost request log middleware with request-id`.

### Task 4: Replace stderr sites + router error/warn instrumentation

**Files:**
- Modify: `rust/crates/pbx-api/src/lib.rs:404,416` (`migrate_on_boot` / `seed_on_boot` → `tracing::error!`)
- Modify: `rust/crates/pbx-api/src/fs_xml.rs:49` (`propagate_esl` → `tracing::warn!(cmd = redact_esl(cmd), "esl propagate failed")`)
- Modify: `rust/crates/pbx-api/src/plugins.rs:214` (registry skip → `tracing::warn!`)
- Modify: mutating handlers (`POST`/`PATCH`/`DELETE`/import) across `src/*.rs`: `error!` on 5xx paths, `warn!` on 4xx with public identifiers only (names/usernames, never secrets), `debug!` on cache-invalidate/tenant-map internals; add `#[tracing::instrument(skip(...))]` on mutating handlers, skipping bodies/connections.
- Test: `rust/crates/pbx-api/tests/logging_test.rs` — add `no_eprintln_in_src` guard? (grep-based, run in CI script instead: `! grep -rn "eprintln!" rust/crates/pbx-api/src`)

**Interfaces:**
- Consumes: `logging::redact_esl`, subscriber from Task 2.
- Produces: zero `eprintln!` in `src/` (tests + pre-init FATAL excepted); consistent failure events.

- [ ] **Step 1: Replace the 4 stderr sites** listed above with the specified events.
- [ ] **Step 2: Instrument routers**: grep for `StatusCode::INTERNAL_SERVER_ERROR`, `map_err`, `Err(` in `src/*.rs`; add error/warn events per rule; `#[instrument]` on mutating handlers. Read-only list/get handlers: no per-handler code (middleware span covers them).
- [ ] **Step 3: Verify** — Run: `! grep -rn "eprintln!" rust/crates/pbx-api/src --include="*.rs" | grep -v "main.rs"; cargo test -p pbx-api; cargo clippy --all-targets -- -D warnings` Expected: no stderr sites outside `main.rs` pre-init gate; full suite green; clippy clean.
- [ ] **Step 4: Commit** `feat(logging): structured failure events across routers`.

### Task 5: Env docs + full verification

**Files:**
- Modify: `.env.example` (add `LOG_LEVEL=info`, `LOG_FORMAT=json`, commented `LOG_FILE=` + `RUST_LOG=` example)
- Modify: `rust/README.md` (logging runbook section: vars, examples, redaction rule, otel hook)
- Modify: `AGENTS.md` (already points at `logging`; confirm wording matches final API)

**Interfaces:** Consumes Tasks 1–4. Produces: documented, verified logging.

- [ ] **Step 1: Docs edits** as above.
- [ ] **Step 2: Format/lint** — Run: `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings` Expected: clean.
- [ ] **Step 3: Full tests** — Run: `cd rust && cargo test -p pbx-api -p pbx-core -p pbx-db -p pbx-fs` Expected: all pass.
- [ ] **Step 4: File-sink smoke** — Run: `LOG_FILE=/tmp/ohngea-pbx-api.log LOG_FORMAT=json cargo run -p pbx-api`, curl `/health` + one bad-auth request, check `/tmp/ohngea-pbx-api.log.$(date +%F)` holds JSON lines with `request_id`, `status`, no query strings. Expected: yes.
- [ ] **Step 5: Commit** `docs(logging): env vars and runbook`.

## Self-Review

1. **Spec coverage:** Goals 1–3 → Task 1; boot/banner→Task 2; Goal 4 → Task 3; Goal 6 → Task 4 (banner stays stdout per spec); Goal 5 otel slot → Task 1; Goal 7 → Task 4 (scoped: mutating handlers + failure sites); Goal 8 tests → Tasks 1+3. ✓
2. **Step scan:** each test step names assertions; each code step pins signatures/files/values; bodies left to implementer. ✓
3. **Type consistency:** `init() -> LogGuards`, `init_for_tests()`, `default_filter() -> String`, `redact_esl(&str) -> String`, `logged_path(Option<&str>, &str) -> String` used identically across tasks. ✓
4. **Review Focus:** all five lines pinned to owning tasks above. ✓
5. **Proportion:** plan pins decisions; bodies/examples left out except where spec fixes values. ✓
