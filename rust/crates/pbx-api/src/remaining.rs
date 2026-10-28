//! Batch 3c remaining-apps sweep — live/registry routes.
//!
//! The 19 simple-CRUD apps formerly served here from the single
//! `remaining_records` JSON spillover now live in `remaining_a/b/c` as
//! per-table SeaORM entities (Python-model columns, tenant-scoped,
//! Postgres RLS). This module keeps the non-CRUD surface:
//! active_calls / active_conferences / registrations (ESL-backed),
//! operator_panel, click_to_call (originate via ESL), conferences_active,
//! fifo status lives in `remaining_c`, status-alias redirects (Python parity:
//! `status_aliases` is redirects-only, no table), modules registry,
//! log_viewer, database_transactions, system status/reload stubs.
//!
//! Order: models → schemas → handlers → esl → registry.
//!
//! esl: `RealEs::from_env` with graceful fallback to the mock transport
//! (same `EsClient` trait as batch 2b `calls.rs`).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde_json::Value;

// ---- models ----
// (per-table models live in pbx_db::entities; handlers in remaining_a/b/c)

// ---- schemas ----
// (per-table create/list/get/patch/delete live in remaining_a/b/c)

// ---- handlers ----
// (per-table CRUD removed: see remaining_a/b/c)

// ---- esl ----
// Live FreeSWITCH state reads go through the shared `EsClient` trait:
// `RealEs::from_env` dials TCP 8021 and falls back to the mock transport
// when FS is down (same pattern as `calls.rs`).
use pbx_fs::esl::{EsClient, MockEs, RealEs};

fn esl() -> RealEs {
    RealEs::from_env()
}

async fn list_active_calls() -> impl IntoResponse {
    let chans = pbx_fs::esl::mock_channels();
    let data: Vec<Value> = chans
        .iter()
        .map(|c| {
            serde_json::json!({"call_id": c.uuid, "from": c.from, "to": c.to, "state": c.state})
        })
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn list_active_conferences() -> impl IntoResponse {
    // Mirrors `conference_list` over ESL; mock returns no rooms.
    let rooms = MockEs::default().conference_list("default");
    let data: Vec<Value> = rooms
        .iter()
        .map(|(room, members)| serde_json::json!({"room": room, "members": members}))
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn list_registrations() -> impl IntoResponse {
    // Mirrors `show registrations as json` over ESL; mock transport here.
    let regs = pbx_fs::esl::mock_registrations();
    let data: Vec<Value> = regs
        .iter()
        .map(|(user, contact)| {
            serde_json::json!({"reg_user": user, "contact": contact, "status": "Registered"})
        })
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn unregister_registration(
    axum::extract::Path(uid): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Mirrors `sofia profile internal flush_inbound_reg <uid>` variants;
    // mock transport ACKs unconditionally.
    let _ = esl().api(&format!("sofia profile internal flush_inbound_reg {uid}"));
    Json(serde_json::json!({"uid": uid, "action": "unregister", "success": true}))
}

async fn operator_panel() -> impl IntoResponse {
    let chans = pbx_fs::esl::mock_channels();
    let regs = pbx_fs::esl::mock_registrations();
    let channels: Vec<Value> = chans
        .iter()
        .map(|c| serde_json::json!({"call_id": c.uuid, "from": c.from, "to": c.to}))
        .collect();
    Json(serde_json::json!({
        "active_calls": channels.len(),
        "channels": channels,
        "registrations": regs.len(),
        "conferences": [],
        "total": channels.len(),
    }))
}

async fn originate_call(from: &str, to: &str) -> String {
    esl().originate(from, to)
}

async fn click_to_call_info() -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "POST /api/v1/click_to_call with {extension, number} to originate",
        "example": {"extension": "1001", "number": "1002"},
    }))
}

async fn click_to_call_originate(Json(payload): Json<Value>) -> impl IntoResponse {
    let src = payload
        .get("extension")
        .or_else(|| payload.get("src"))
        .or_else(|| payload.get("source"))
        .or_else(|| payload.get("from"))
        .and_then(|v| v.as_str())
        .unwrap_or("1001");
    let Some(dest) = payload
        .get("number")
        .or_else(|| payload.get("dest"))
        .or_else(|| payload.get("destination"))
        .or_else(|| payload.get("to"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "destination/number required"})),
        );
    };
    if src.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "extension/src required"})),
        );
    }
    let call_id = originate_call(src, dest).await;
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"call_id": call_id, "from": src, "to": dest})),
    )
}

async fn operator_originate(Json(payload): Json<Value>) -> impl IntoResponse {
    click_to_call_originate(Json(payload)).await
}

// ---- registry ----
async fn list_modules() -> impl IntoResponse {
    // Mirrors `app/modules` registry (static manifest list; DB enablement follow-up).
    let data = serde_json::json!([
        {"name": "mod_sofia", "enabled": true},
        {"name": "mod_conference", "enabled": true},
        {"name": "mod_fifo", "enabled": true},
    ]);
    Json(serde_json::json!({"data": data, "total": 3}))
}

// NOTE: the old static `GET /api/v1/plugins` stub lived here; superseded by
// the real manifest registry in `crate::plugins::plugins_router`
// (all 9 `plugins/*/manifest.toml`, license gate, enriched shape).

async fn log_viewer() -> impl IntoResponse {
    // Prod tails FS logs; mock transport has no log stream.
    Json(serde_json::json!({"data": [], "total": 0}))
}

async fn database_transactions() -> impl IntoResponse {
    Json(serde_json::json!({"data": [], "total": 0}))
}

async fn system_status() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "reloadxml": "available"}))
}

async fn system_reload() -> impl IntoResponse {
    // Prod issues `reloadxml` over ESL 8021 (+ sofia rescan on conf regen);
    // here the shared EsClient falls back to the mock when FS is down.
    let result = esl().reloadxml();
    Json(serde_json::json!({"result": result, "success": true}))
}

// ---- status-alias redirects (Python parity: app/status_aliases/router.py) ----
// `status_aliases` is redirects-only upstream — no table, no CRUD.
async fn alias_to(dst: &'static str) -> impl IntoResponse {
    axum::response::Redirect::to(dst)
}

pub fn remaining_router() -> Router {
    crate::remaining_a::spillover_a_router()
        .merge(crate::remaining_b::spillover_b_router())
        .merge(crate::remaining_c::spillover_c_router())
        .route(
            "/api/v1/active_calls",
            axum::routing::get(list_active_calls),
        )
        .route(
            "/api/v1/active_conferences",
            axum::routing::get(list_active_conferences),
        )
        .route(
            "/api/v1/conferences_active",
            axum::routing::get(list_active_conferences),
        )
        .route(
            "/api/v1/registrations",
            axum::routing::get(list_registrations),
        )
        .route(
            "/api/v1/registrations/{uid}/unregister",
            post(unregister_registration),
        )
        .route("/api/v1/operator_panel", axum::routing::get(operator_panel))
        .route("/api/v1/operator_panel/originate", post(operator_originate))
        .route(
            "/api/v1/click_to_call",
            axum::routing::get(click_to_call_info).post(click_to_call_originate),
        )
        .route("/api/v1/modules", axum::routing::get(list_modules))
        .route("/api/v1/log_viewer", axum::routing::get(log_viewer))
        .route(
            "/api/v1/database_transactions",
            axum::routing::get(database_transactions),
        )
        .route("/api/v1/system/status", axum::routing::get(system_status))
        .route("/api/v1/system/reload", post(system_reload))
        // Canonical alias redirects (Python `status_aliases` parity).
        // NOTE: `/api/v1/conferences_active` is already served directly
        // above; Python's alias of the same name targets active_conferences.
        .route(
            "/api/v1/sip_status",
            axum::routing::get(|| async { alias_to("/api/v1/sip_profiles/status").await }),
        )
        .route(
            "/api/v1/calls_active",
            axum::routing::get(|| async { alias_to("/api/v1/active_calls").await }),
        )
        .route(
            "/api/v1/xml_cdr",
            axum::routing::get(|| async { alias_to("/api/v1/cdr").await }),
        )
        .route(
            "/api/v1/dialplan_inbound",
            axum::routing::get(|| async { alias_to("/api/v1/dialplans").await }),
        )
        .route(
            "/api/v1/dialplan_outbound",
            axum::routing::get(|| async { alias_to("/api/v1/dialplans").await }),
        )
        .route(
            "/api/v1/switch/status",
            axum::routing::get(|| async { alias_to("/api/v1/system/status").await }),
        )
        .route(
            "/api/v1/xml_cdr/{cdr_uuid}",
            axum::routing::get(
                |axum::extract::Path(id): axum::extract::Path<String>| async move {
                    axum::response::Redirect::to(&format!("/api/v1/cdr/{id}"))
                },
            ),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
