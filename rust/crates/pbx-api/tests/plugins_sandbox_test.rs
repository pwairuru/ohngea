//! Extism WASM sandbox — TDD RED.
//!
//! Ground truth: `src/plugins.rs` registry (manifest/license/toggles),
//! Extism `Manifest`/`PluginBuilder`/`Plugin::call` docs (bytes-in/out,
//! `extism:host/user` host fns, `extism:host/env` memory funcs).
//!
//! Expected to FAIL until `crate::plugin_sandbox` (extism dep) exists.

use pbx_api::plugin_sandbox::{PluginSandbox, SandboxError};
use pbx_api::plugins::parse_example_manifest;

/// Minimal guest: official Extism WAT pattern — imports host fns
/// (`pbx_log`, `db_get` stub), exports `handle` (fixed `ok:example` output)
/// and `check_db` (traps unless the `db_get` allowlist answers 1/0).
/// Single source of truth: `plugins/example/plugin.wat` (also powers the
/// `example`-route hook via `EXAMPLE_GUEST_WASM`).
const GUEST_WAT: &str = include_str!("../../../../plugins/example/plugin.wat");

/// Untrusted guest: imports WASI — must fail to instantiate (WASI off).
const WASI_WAT: &str = r#"
(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func (param i32 i32 i32 i32) (result i32)))
  (func (export "handle") (result i32) (i32.const 0)))
"#;

/// Runaway guest: infinite loop — must be stopped by fuel/timeout.
const SPIN_WAT: &str = r#"
(module
  (func (export "spin") (result i32)
    (loop $l (br $l))))
"#;

fn guest_wasm() -> Vec<u8> {
    wat::parse_str(GUEST_WAT).expect("guest wat compiles")
}

// ---- 0. provenance: committed plugin.wasm matches plugin.wat ----
#[test]
fn guest_wasm_matches_wat() {
    assert_eq!(
        guest_wasm(),
        pbx_api::plugin_sandbox::EXAMPLE_GUEST_WASM,
        "recompile plugins/example/plugin.wat into plugin.wasm on drift"
    );
}

// ---- 1. guest runs through sandbox, returns output, hits host fn ----
#[test]
fn sandbox_calls_guest_and_returns_output() {
    let mut sb = PluginSandbox::new();
    sb.register(&parse_example_manifest(), guest_wasm())
        .expect("MIT-licensed example guest loads");
    let out = sb
        .call_plugin("example", "handle", b"example")
        .expect("guest runs");
    assert_eq!(out, b"ok:example");
    assert!(
        sb.log_calls() >= 1,
        "pbx_log host fn must observe the guest call"
    );
}

// ---- 2. manifest gate: copyleft license rejected before any exec ----
#[test]
fn sandbox_rejects_disallowed_license() {
    let mut m = parse_example_manifest();
    m.license = "GPL-3.0".to_string();
    let mut sb = PluginSandbox::new();
    let err = sb.register(&m, guest_wasm()).unwrap_err();
    assert!(
        matches!(err, SandboxError::LicenseDenied(_)),
        "expected LicenseDenied, got {err:?}"
    );
}

// ---- 3. isolation: no WASI unless explicitly enabled ----
#[test]
fn sandbox_blocks_wasi_by_default() {
    assert!(!PluginSandbox::new().wasi_enabled());
    let mut sb = PluginSandbox::new();
    sb.register(
        &parse_example_manifest(),
        wat::parse_str(WASI_WAT).expect("wasi wat compiles"),
    )
    .expect("license gate passes; capability gate is at call time");
    assert!(
        sb.call_plugin("example", "handle", b"").is_err(),
        "wasi import must not link with WASI off"
    );
}

// ---- 4. limits: fuel + timeout set; runaway guest stopped fast ----
#[test]
fn sandbox_enforces_fuel_and_timeout() {
    let sb = PluginSandbox::new();
    assert_eq!(sb.timeout_ms(), 1_000);
    assert!(sb.fuel_limit() > 0, "fuel limit must be set");
    assert!(sb.max_memory_pages() > 0, "memory cap must be set");
    let mut sb = sb.with_timeout_ms(250);
    sb.register(
        &parse_example_manifest(),
        wat::parse_str(SPIN_WAT).expect("spin wat compiles"),
    )
    .expect("spin guest loads");
    let start = std::time::Instant::now();
    assert!(
        sb.call_plugin("example", "spin", b"").is_err(),
        "infinite loop must be interrupted"
    );
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "interruption must be fast"
    );
}

// ---- 5b. lookup wiring: caller-supplied db_get resolver is honored ----
#[test]
fn sandbox_db_get_custom_lookup() {
    use std::sync::{Arc, Mutex};
    let mut sb = PluginSandbox::new();
    sb.register(&parse_example_manifest(), guest_wasm())
        .expect("guest loads");
    let seen = Arc::new(Mutex::new(Vec::<i64>::new()));
    let seen_clone = seen.clone();
    let out = sb
        .call_plugin_with_lookup("example", "check_db", b"", move |key| {
            seen_clone.lock().unwrap().push(key);
            match key {
                1 => 1,
                999 => 0,
                _ => 7,
            }
        })
        .expect("guest allowlist shape still holds with custom lookup");
    assert!(out.is_empty(), "check_db sets no output");
    assert_eq!(
        *seen.lock().unwrap(),
        vec![1, 999],
        "resolver must observe the guest's db_get keys in order"
    );
}
// ---- 6. hook: GET /api/v1/example runs the guest via the sandbox ----
#[tokio::test]
async fn example_route_proves_sandbox_path() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    // login (same shape as plugins_test.rs)
    let res = pbx_api::test_app()
        .oneshot(
            Request::post("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"username":"admin","password":"x"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = v
        .get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token");

    let res = pbx_api::test_app()
        .oneshot(
            Request::get("/api/v1/example")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v.get("plugin").and_then(|p| p.as_str()), Some("example"));
    let sb = v.get("sandbox").expect("sandbox hook output");
    assert_eq!(sb.get("ok").and_then(|o| o.as_bool()), Some(true));
    assert_eq!(
        sb.get("output").and_then(|o| o.as_str()),
        Some("ok:example")
    );
}
// ---- 5. host fns: db_get stub allowlist (1 allowed, 999 denied) ----
#[test]
fn sandbox_db_get_allowlist() {
    let mut sb = PluginSandbox::new();
    sb.register(&parse_example_manifest(), guest_wasm())
        .expect("guest loads");
    let out = sb
        .call_plugin("example", "check_db", b"")
        .expect("allowlist answers must match (traps otherwise)");
    assert!(out.is_empty(), "check_db sets no output");
}
