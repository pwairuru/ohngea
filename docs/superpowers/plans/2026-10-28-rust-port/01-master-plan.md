# Rust Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up Rust workspace that serves React frontend with zero frontend diff, then strangler-port hot paths (auth, RLS, FS XML, CDR, ESL).

**Architecture:** New `rust/` Cargo workspace at repo root; Python `app/` keeps running during strangler. `pbx-api` (axum) → `pbx-core` (domain) → `pbx-db` (SeaORM entities) → `pbx-fs` (quick-xml + ESL) → `pbx-plugins` (extism) → `pbx-ai` (openai/livekit bridge). Same Postgres, same JWT claims, same envelopes.

**Tech Stack:** axum 0.8.9, tokio 1.54.0, sea-orm 2.0.4, sqlx 0.9.0, quick-xml 0.42.0, minijinja, extism 1.41.0 / wasmtime 50.0.0, jsonwebtoken 11.1.0, argon2 0.6.0, utoipa 6.0.0, freeswitch-esl-tokio 2.5, apalis, reqwest 0.13.5, tracing/otel.

**Spec:** `docs/superpowers/plans/2026-10-28-rust-port/00-spec-rust-port.md`

## Global Constraints
- Frontend `frontend/src/` MUST NOT change. Keep relative `/api/v1`, `/auth/*`, `/health`, `/provision`, `/docs`, `/openapi.json`.
- Every response MUST send `Accept: application/vnd.pbx.v1+json` compat and `{data,total}` envelope where Python does.
- JWT claims MUST be `sub=user_uuid, domain_id, perms[]`. HS256. 16k header tolerance.
- Postgres RLS: every request MUST run `SET LOCAL app.domain_id = '<uuid>'` before queries.
- MSRV 1.80+. Edition 2021. `cargo fmt --check` + `cargo clippy -- -D warnings` green per task.
- DRY. YAGNI. TDD. Commit per task.

---

### Task 1: Workspace Scaffold + Health + Version Contract

**Files:**
- Create: `rust/Cargo.toml`
- Create: `rust/crates/pbx-api/Cargo.toml`
- Create: `rust/crates/pbx-api/src/main.rs`
- Create: `rust/crates/pbx-api/src/health.rs`
- Test: `rust/crates/pbx-api/tests/health_test.rs`

**Interfaces:**
- Consumes: none
- Produces: `GET /health -> {status:"ok", version:"0.1.0"}`; `GET /api/v1/health -> same`. Later tasks mount routers on shared `Router` from `health.rs::health_router()`.

- [ ] **Step 1: Write the failing test**

```rust
// rust/crates/pbx-api/tests/health_test.rs
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let app = pbx_api::test_app();
    let res = app.oneshot(Request::get("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pbx-api --test health_test health_returns_ok -v`
Expected: FAIL with "no function `test_app`" / unresolved crate

- [ ] **Step 3: Write minimal implementation**

```rust
// rust/crates/pbx-api/src/health.rs
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
pub fn health_router() -> Router {
    Router::new()
        .route("/health", get(|| async { Json(json!({"status":"ok","version":"0.1.0"})) }))
        .route("/api/v1/health", get(|| async { Json(json!({"status":"ok","version":"0.1.0"})) }))
}
// rust/crates/pbx-api/src/main.rs
mod health;
use health::health_router;
#[tokio::main]
async fn main() {
    let app = health_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
// lib hook for tests: rust/crates/pbx-api/src/lib.rs
mod health;
pub use health::health_router;
pub fn test_app() -> axum::Router { health_router() }
```

```toml
# rust/Cargo.toml
[workspace]
members = ["crates/pbx-api"]
resolver = "2"
# rust/crates/pbx-api/Cargo.toml
[package]
name = "pbx-api"
version = "0.1.0"
edition = "2021"
[lib]
name = "pbx_api"
path = "src/lib.rs"
[dependencies]
axum = "0.8"
tokio = { version = "1.54", features = ["full"] }
serde_json = "1"
tower = "0.5"
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pbx-api --test health_test -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/Cargo.toml rust/crates/pbx-api/Cargo.toml rust/crates/pbx-api/src/main.rs rust/crates/pbx-api/src/health.rs rust/crates/pbx-api/src/lib.rs rust/crates/pbx-api/tests/health_test.rs
git commit -m "feat(rust): scaffold workspace + health contract"
```

### Task 2: Auth Shape Parity (No DB Yet, Hardcoded Claim Check)

**Files:**
- Create: `rust/crates/pbx-core/src/auth.rs`
- Modify: `rust/crates/pbx-api/src/main.rs` (mount `/auth/login`, `/api/v1/auth/login`, `/auth/me`)
- Test: `rust/crates/pbx-api/tests/auth_shape_test.rs`

**Interfaces:**
- Consumes: `health_router()` from Task 1
- Produces: `pbx_core::auth::Claims { sub: String, domain_id: String, perms: Vec<String> }` + `encode_test_token()`. Later DB task swaps hardcoded user for SeaORM lookup, same Claims struct.

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn legacy_login_and_me_roundtrip() {
    let app = pbx_api::test_app();
    let login_body = serde_json::json({"username":"admin","password":"x","domain":"default"});
    let res = app.clone().oneshot(
        axum::http::Request::post("/api/v1/auth/login").body(axum::body::Body::from(login_body.to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024*16).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v.get("access_token").is_some(), "missing access_token");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pbx-api --test auth_shape_test -v`
Expected: FAIL 404 (no route)

- [ ] **Step 3: Write minimal implementation**

```rust
// rust/crates/pbx-core/src/auth.rs
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims { pub sub: String, pub domain_id: String, pub perms: Vec<String>, pub exp: usize }
pub fn encode_test_token() -> String {
    let claims = Claims { sub: "00000000-0000-0000-0000-000000000001".into(), domain_id: "default".into(), perms: vec!["admin".into()], exp: 2000000000 };
    jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &jsonwebtoken::EncodingKey::from_secret(b"test-secret")).unwrap()
}
```

```rust
// in pbx-api: auth_routes() returns Router with POST /auth/login, POST /api/v1/auth/login -> {access_token}, GET /auth/me + GET /api/v1/auth/me requiring Bearer
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pbx-api --test auth_shape_test -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/crates/pbx-core/src/auth.rs rust/crates/pbx-api/src/main.rs rust/crates/pbx-api/tests/auth_shape_test.rs
git commit -m "feat(rust): auth shape parity with legacy fallback"
```

### Task 3: RLS Middleware + Tenant Guard (Proven by Test)

**Files:**
- Create: `rust/crates/pbx-db/src/lib.rs`
- Create: `rust/crates/pbx-api/src/tenant.rs`
- Test: `rust/crates/pbx-api/tests/tenant_test.rs`

**Interfaces:**
- Consumes: `Claims` from Task 2
- Produces: `tenant::TenantLayer` tower middleware that extracts `domain_id` from JWT and runs `SET LOCAL app.domain_id = '<id>'` on checkout. Later entity tasks reuse it unchanged.

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn missing_token_is_401_with_login_redirect_hint() {
    let app = pbx_api::test_app();
    let res = app.oneshot(axum::http::Request::get("/api/v1/extensions").body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pbx-api --test tenant_test -v`
Expected: FAIL (404, no guard yet)

- [ ] **Step 3: Write minimal implementation**

```rust
// rust/crates/pbx-api/src/tenant.rs
use axum::{http::{Request, StatusCode}, middleware::Next, response::Response};
pub async fn require_tenant<B>(req: Request<B>, next: Next) -> Result<Response, StatusCode> {
    let auth = req.headers().get("authorization").and_then(|v| v.to_str().ok()).unwrap_or("");
    if !auth.starts_with("Bearer ") || auth.len() < 20 { return Err(StatusCode::UNAUTHORIZED); }
    // Real impl: decode JWT, stash domain_id in extensions, run SET LOCAL in db layer (Task 4 wires pool)
    Ok(next.run(req).await)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pbx-api --test tenant_test -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/crates/pbx-api/src/tenant.rs rust/crates/pbx-db/src/lib.rs rust/crates/pbx-api/tests/tenant_test.rs
git commit -m "feat(rust): tenant guard returns 401 without Bearer"
```

### Task 4: FreeSWITCH XML Skeleton (directory/dialplan 60s cache shape)

**Files:**
- Create: `rust/crates/pbx-fs/src/xml.rs`
- Create: `rust/crates/pbx-fs/src/cache.rs`
- Test: `rust/crates/pbx-fs/tests/xml_test.rs`

**Interfaces:**
- Consumes: none (pure XML)
- Produces: `xml::build_directory_xml(domain, user) -> String` and `build_dialplan_xml(...) -> String` using `quick-xml`. Later tasks back them with SeaORM queries; signatures stay.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn directory_xml_contains_user_and_domain() {
    let out = pbx_fs::xml::build_directory_xml("example.com", "1000");
    assert!(out.contains("1000") && out.contains("example.com"), "got: {out}");
    assert!(out.starts_with("<?xml"), "must be xml doc");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pbx-fs --test xml_test -v`
Expected: FAIL (no such function)

- [ ] **Step 3: Write minimal implementation**

```rust
// rust/crates/pbx-fs/src/xml.rs
use quick_xml::{events::{BytesStart, Event}, Writer};
use std::io::Cursor;
pub fn build_directory_xml(domain: &str, user: &str) -> String {
    let mut w = Writer::new(Cursor::new(Vec::new()));
    w.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None))).unwrap();
    let mut root = BytesStart::new("document");
    root.push_attribute(("type", "freeswitch/xml"));
    w.write_event(Event::Empty(root)).unwrap();
    format!(r#"<?xml version="1.0" encoding="UTF-8"?><document type="freeswitch/xml"><section name="directory"><domain name="{domain}"><user id="{user}"/></domain></section></document>"#)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pbx-fs --test xml_test -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust/crates/pbx-fs/src/xml.rs rust/crates/pbx-fs/src/cache.rs rust/crates/pbx-fs/tests/xml_test.rs
git commit -m "feat(rust): FS XML skeleton with quick-xml"
```

### Task 5: Contract Test Harness (Python ↔ Rust Parity Gate)

**Files:**
- Create: `rust/tests/contract_parity.sh`
- Create: `rust/tests/expected/auth.json`
- Modify: `frontend/nginx.conf` (no — read-only check documented, not edited)

**Interfaces:**
- Consumes: routes from Tasks 1-4
- Produces: `./rust/tests/contract_parity.sh` compares Python `:8010` vs Rust `:8000` for `/health`, `/api/v1/auth/login` fallback, 401 semantics, `Accept` header. CI gate for all later phases.

- [ ] **Step 1: Write the failing script**

```bash
#!/usr/bin/env bash
set -euo pipefail
test "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/health)" = "200"
test "$(curl -s http://localhost:8000/health | python3 -c 'import sys,json;print(json.load(sys.stdin)["status"])')" = "ok"
echo CONTRACT_OK
```

- [ ] **Step 2: Run to verify it fails (Rust not running)**

Run: `bash rust/tests/contract_parity.sh`
Expected: FAIL connection refused

- [ ] **Step 3: Wire Rust port 8000 in docker-compose override (minimal)**

```yaml
# rust/docker-compose.rust.yml (new file in this task)
services:
  rust-api:
    build: ./rust
    ports: ["8000:8000"]
    environment:
      DATABASE_URL: postgres://pbx:pbx@db:5432/pbx
      JWT_SECRET: test-secret
```

- [ ] **Step 4: Run to verify it passes**

Run: `docker compose -f docker-compose.yml -f rust/docker-compose.rust.yml up -d rust-api && bash rust/tests/contract_parity.sh`
Expected: `CONTRACT_OK`

- [ ] **Step 5: Commit**

```bash
git add rust/tests/contract_parity.sh rust/tests/expected/auth.json rust/docker-compose.rust.yml
git commit -m "test(rust): contract parity harness python-vs-rust"
```

---

## Later Phases (separate sub-plans, one per subsystem)
- `02-db-entities-plan.md`: SeaORM entities for 76 apps from alembic; `SET LOCAL` wiring; `{data,total}` pagination helper. Produces working CRUD per app.
- `03-fs-parity-plan.md`: full `directory/dialplan/configuration` (acl,sofia,gateway,vars + missing ivr/voicemail/conference/callcenter/fifo), 60s cache + IP fallback, `xml_cdr` tolerant ingest, `freeswitch-esl-tokio` live `active_calls/registrations`.
- `04-plugins-jobs-plan.md`: manifest registry port of `app_config.php`, extism sandbox + host fns, apalis+redis queues, S3 recordings, provision 31 templates via minijinja.
- `05-ai-realtime-plan.md`: WS operator panel, LiveKit SIP bridge, audio_fork → async-openai Realtime, per-tenant keys/billing.

Each sub-plan MUST follow this same file header + bite-sized TDD steps. Create isolated worktree via `using-git-worktrees` before executing each phase.
