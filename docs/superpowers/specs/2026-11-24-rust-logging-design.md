# Enterprise logging for Rust backend — design

Date: 2026-11-24. Approach picked by human: **A (tracing + tracing-subscriber)**.
B (log/env_logger) rejected: no spans, rewrite later for telemetry.
C (OTel-first now) rejected: collector dependency + per-request network risk before basics work.

## Intent / why

`pbx-api` serves production telephony with zero structured logging: 6 `eprintln!/println!`
sites, no levels, no file output, no request correlation, secrets discipline only by convention.
Enterprise bar: level switching without rebuild, file + stdout sinks, per-request trace context,
telemetry hook that costs nothing until enabled.

## Goals

1. `LOG_LEVEL` (trace|debug|info|warn|error, default info) + full `RUST_LOG` filter override.
2. `LOG_FILE` optional: unset → stdout only; set → non-blocking file appender (daily rotation) + stdout.
3. `LOG_FORMAT=json|pretty` (default json in release, pretty in debug).
4. Request middleware: method, route path, status, latency ms, `x-request-id` (generate UUIDv7 if absent, echo back).
5. One `pbx-api::logging::init()` called first in `main.rs`; OTel OLP layer slot behind `otel` cargo feature (off by default, zero cost off).
6. Replace all 6 `eprintln!/println!` sites (banner stays on stdout by design) with tracing events.
7. Handler instrumentation pass over every `pbx-api/src/*.rs` router: error on 5xx/ESL/migrate/seed failures, warn on 4xx-adjacent + plugin skips, debug for cache/tenant internals. No passwords/tokens/hashes/URLs in fields.
8. Tests: init-once guard, filter-parse unit tests, redaction test, request-id middleware test.

## Non-goals / anti-goals

- No OTel exporter wired now (hook point only). No log shipping agent in repo.
- No React logging changes in this work (backend only; frontend keeps ErrorBoundary + console).
- No rewriting `rust/README.md` history or `docs/` Python provenance notes (intentional history).
- Anti-goal: logging must never block requests (non-blocking writer), never leak secrets, never `panic!` on bad env (fall back + warn).

## Constraints

- Deps: `tracing`, `tracing-subscriber` (env-filter, json), `tracing-appender` — all already in Cargo.lock transitively; network available.
- Keep `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` clean.
- `#[instrument]` must `skip` bodies/connections (no PII, no perf hit on hot XML paths).
- SeaORM/hyper noise default-off: suggested `RUST_LOG=pbx_api=debug,pbx_fs=debug,sea_orm=warn,hyper=warn`.

## Decided parts

- Module: `rust/crates/pbx-api/src/logging.rs`, `pub mod logging` in lib.rs; `init()` returns `Option<WorkerGuard>` held in `main`.
- Env: `LOG_LEVEL` → base filter; `RUST_LOG` wins if set; `LOG_FILE` → file stem in its dir, daily rotation; `LOG_FORMAT` json|pretty.
- Middleware `request_log_middleware` (from_fn, next to existing accept/pagination layers): incoming request-id or UUIDv7, span fields `method path status latency_ms request_id`, error log on 5xx.
- Replacements: main.rs FATAL stays stderr pre-init then `tracing::error`; seed/migrate `eprintln` → `error!`; fs_xml ESL propagate → `warn!` with cmd + redact output; plugins registry skip → `warn!`.
- `.env.example` gains `LOG_LEVEL/LOG_FORMAT/LOG_FILE` + commented `RUST_LOG` example; AGENTS.md + rust/README runbook updated.

## Builder-check resolutions (my calls, 2026-11-24)

1. ESL redaction: log command verb only (e.g. `sofia rescan`), never raw output; helper `redact_esl(cmd)` truncates to 200 chars and masks `password|passwd|secret|token`. SIP URIs (`user/1001@domain`) are fine — user IDs, not secrets.
2. Default filter: `pbx_api=info,pbx_fs=info,pbx_db=info,pbx_core=info,sea_orm=warn,hyper=warn,tower_http=warn`. Bad `LOG_LEVEL`/`RUST_LOG` → stderr warn + fall back to default, never panic.
3. `LOG_FILE` = full file path (e.g. `/var/log/ohngea/pbx-api.log`); split into dir + file-stem for daily rotation (`pbx-api.log.2026-11-24`). Missing parents created with `create_dir_all`; failure → stderr warn + stdout only. File and stdout share format.
4. Middleware: `request_log_middleware` outermost (wraps auth rejections too). Path = `MatchedPath` template when matched else raw `uri.path()` — never query string. `x-request-id` generated (UUIDv7) if absent, echoed on response.
5. Goal 7 scope: error/warn events at every failure site in all routers + `#[instrument(skip(body, conn))]` on mutating handlers (POST/PATCH/DELETE/import); read-only list/get handlers get span from middleware only — no per-handler code. Hot FS XML paths untouched beyond warn-on-error.
6. `otel` feature: optional deps `tracing-opentelemetry` + `opentelemetry-otlp`, `cfg(feature = "otel")` layer in `init()`; default build zero-cost. No CI `--features otel` gate in this work.
7. Guard: `init()` returns `LogGuards { _file: Option<WorkerGuard>, _stdout: WorkerGuard }` held in `main()` to end of `main`; tests use `OnceLock<()>` + `try_init` so parallel tests never double-init. Pre-init failures (bad env) go to stderr before subscriber exists.

## Left to builder

- Exact extra span fields per sensitive router (auth, cdr import) within redaction rule.
- Rotation retention count and file-name prefix.
- Whether `x-request-id` also feeds `tenant` span or stays middleware-local.
- Test file layout (new `logging_test.rs` vs inline `#[cfg(test)]`).
