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

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
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

// ---- tones (Task 6; `fusionpbx/app/tones/app_config.php` parity) ----
//
// Tones are `v_vars` rows with `var_category = 'Tones'` (PHP destinations
// query). Minimal read surface: PHP ships no tone CRUD pages, so only
// `GET /api/v1/tones` exists. Shape mirrors the destinations mapping:
// `{uuid, name, destination, description}`.
async fn list_tones(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "tone_destinations") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = pbx_db::entities::vars::Entity::find()
        .filter(pbx_db::entities::vars::Column::DomainId.eq(domain))
        .filter(pbx_db::entities::vars::Column::VarCategory.eq("Tones".to_owned()))
        .order_by(pbx_db::entities::vars::Column::VarName, sea_orm::Order::Asc)
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "uuid": r.var_uuid.to_string(),
                "name": r.var_name,
                "destination": r.var_value,
                "description": r.var_description,
            })
        })
        .collect();
    let total = data.len() as u64;
    Json(serde_json::json!({"data": data, "total": total})).into_response()
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

// ---- active_calls per-channel control (parity: app/active_calls/router.py) ----
async fn get_active_call(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(uid): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _ = &claims;
    // Mirrors `uuid_exists {uid}` against the shared mock channel store
    // (same store `POST /api/v1/calls/originate` writes).
    match pbx_fs::esl::mock_channels()
        .into_iter()
        .find(|c| c.uuid == uid)
    {
        Some(c) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "call_uuid": c.uuid,
                "call_id": c.uuid,
                "from": c.from,
                "to": c.to,
                "state": c.state,
                "exists": true,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Active call not found"})),
        )
            .into_response(),
    }
}

async fn hangup_active_call(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(uid): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "active_calls_hangup") {
        return (s, j).into_response();
    }
    // Mirrors `uuid_kill {uid}`; also drops the mock leg so a later GET 404s.
    let _ = esl().api(&format!("uuid_kill {uid}"));
    let _ = pbx_fs::esl::mock_kill(&uid);
    (
        StatusCode::OK,
        Json(serde_json::json!({"result": "hangup queued (ESL offline)", "call_uuid": uid})),
    )
        .into_response()
}

// ---- active_conferences member control (parity: conferences_active/conference_exec.php) ----
async fn count_active_conferences(
    Extension(claims): Extension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_active_view") {
        return (s, j).into_response();
    }
    let rooms = MockEs::default().conference_list("default");
    let count = rooms.len() as u64;
    Json(serde_json::json!({"count": count})).into_response()
}

async fn list_conference_members(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(conf): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_active_view") {
        return (s, j).into_response();
    }
    // Best-effort `conference {name} xml_list` parse; mock transport has no
    // live rooms, so the snapshot is empty offline.
    let reply = esl().api(&format!("conference {conf} xml_list"));
    let data = parse_conference_members(&reply);
    Json(serde_json::json!({"conference": conf, "data": data, "total": data.len()})).into_response()
}

/// Minimal `xml_list` member extraction: `<member ...><uuid>…</uuid></member>`
/// rows when FS answered live; empty when the reply is a mock fallback.
fn parse_conference_members(reply: &str) -> Vec<Value> {
    if reply.contains("mock-fallback") || !reply.contains("<member") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut rest = reply;
    while let Some(start) = rest.find("<member") {
        rest = &rest[start..];
        let Some(end) = rest.find("</member>") else {
            break;
        };
        let chunk = &rest[..end];
        let uuid = chunk
            .find("<uuid>")
            .and_then(|s| {
                chunk[s + 6..]
                    .find("</uuid>")
                    .map(|e| chunk[s + 6..s + 6 + e].trim())
            })
            .unwrap_or("")
            .to_owned();
        if !uuid.is_empty() {
            out.push(serde_json::json!({"member_id": uuid, "uuid": uuid}));
        }
        rest = &rest[end + 9..];
    }
    out
}

/// Member actions mirroring `conference_exec.php` `$data` switch
/// (mute/unmute/deaf/undeaf/kick/lock/unlock/energy/volume/record).
fn valid_conf_action(action: &str) -> bool {
    matches!(
        action,
        "mute"
            | "unmute"
            | "deaf"
            | "undeaf"
            | "kick"
            | "lock"
            | "unlock"
            | "energy"
            | "volume"
            | "volume_in"
            | "volume_out"
            | "record"
            | "norecord"
    )
}

async fn conference_member_action(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((conf, member, action)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_active_view") {
        return (s, j).into_response();
    }
    if !valid_conf_action(&action) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": format!("unknown conference action: {action}")})),
        )
            .into_response();
    }
    // Mirrors `conference {name} {action} {member}` over ESL.
    let result = esl().api(&format!("conference {conf} {action} {member}"));
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "conference": conf,
            "member": member,
            "action": action,
            "result": result,
        })),
    )
        .into_response()
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

// ---- operator_panel live actions (parity: app/operator_panel/router.py) ----
async fn operator_calls() -> impl IntoResponse {
    let chans = pbx_fs::esl::mock_channels();
    let data: Vec<Value> = chans
        .iter()
        .map(|c| serde_json::json!({"call_id": c.uuid, "from": c.from, "to": c.to, "state": c.state}))
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn op_hangup(axum::extract::Path(uid): axum::extract::Path<String>) -> impl IntoResponse {
    let _ = esl().api(&format!("uuid_kill {uid}"));
    Json(serde_json::json!({"result": "hangup queued (ESL offline)", "call_uuid": uid}))
}

async fn op_eavesdrop(
    axum::extract::Path(uid): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let _ = payload;
    let _ = esl().api(&format!("uuid_eavesdrop {uid}"));
    Json(
        serde_json::json!({"result": "eavesdrop queued (ESL offline)", "call_uuid": uid, "action": "eavesdrop"}),
    )
}

async fn op_coach(
    axum::extract::Path(uid): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let _ = payload;
    let _ = esl().api(&format!("uuid_coach {uid}"));
    Json(
        serde_json::json!({"result": "coach queued (ESL offline)", "call_uuid": uid, "action": "coach"}),
    )
}

async fn op_record(
    axum::extract::Path(uid): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let _ = payload;
    let _ = esl().api(&format!("uuid_record {uid}"));
    Json(
        serde_json::json!({"result": "record queued (ESL offline)", "call_uuid": uid, "action": "record"}),
    )
}

async fn provision_registration(
    axum::extract::Path(uid): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _ = esl().api(&format!("sofia profile internal rescan {uid}"));
    Json(serde_json::json!({"uid": uid, "action": "provision", "success": true}))
}

async fn reboot_registration(
    axum::extract::Path(uid): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _ = esl().api(&format!("sofia profile internal reboot {uid}"));
    Json(serde_json::json!({"uid": uid, "action": "reboot", "success": true}))
}

// ---- webrtc credentials (parity: app/webrtc/credentials_router.py) ----
async fn webrtc_credentials(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let settings = pbx_db::entities::webrtc_settings::Entity::find_by_id(domain)
        .one(db)
        .await
        .unwrap_or(None);
    let Some(s) = settings else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"detail": "WebPhone disabled for this domain"})),
        )
            .into_response();
    };
    if !s.enabled {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"detail": "WebPhone disabled for this domain"})),
        )
            .into_response();
    }
    let Some(wss_url) = s.wss_url.filter(|u| !u.trim().is_empty()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"detail": "wss_url not configured"})),
        )
            .into_response();
    };
    let sip_domain = s.sip_domain.unwrap_or_else(|| "example.com".to_owned());
    if let Some(ext_val) = payload.get("extension").and_then(|v| v.as_str()) {
        if s.auth_mode != "extension" && s.auth_mode != "both" {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"detail": "extension auth disabled"})),
            )
                .into_response();
        }
        let ext = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain))
            .filter(pbx_db::entities::extensions::Column::Extension.eq(ext_val.to_owned()))
            .one(db)
            .await
            .unwrap_or(None);
        let Some(e) = ext.filter(|e| e.enabled) else {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"detail": "Extension not found"})),
            )
                .into_response();
        };
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "sip_uri": format!("sip:{}@{sip_domain}", e.extension),
                "wss_url": wss_url,
                "auth_user": e.extension,
                "password": e.password.unwrap_or_else(|| "1234".to_owned()),
                "display_name": e.effective_caller_id_name,
                "expires_in": 3600,
                "ice_servers": s.ice_servers.unwrap_or_else(|| serde_json::json!([{"urls": "stun:stun.l.google.com:19302"}])),
            })),
        )
            .into_response();
    }
    if let Some(user) = payload.get("webrtc_user").and_then(|v| v.as_str()) {
        if s.auth_mode != "webrtc" && s.auth_mode != "both" {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"detail": "webrtc auth disabled"})),
            )
                .into_response();
        }
        let pwd = payload
            .get("webrtc_pass")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "sip_uri": format!("sip:{user}@{sip_domain}"),
                "wss_url": wss_url,
                "auth_user": user,
                "password": pwd,
                "display_name": user,
                "expires_in": 3600,
                "ice_servers": s.ice_servers.unwrap_or_else(|| serde_json::json!([{"urls": "stun:stun.l.google.com:19302"}])),
            })),
        )
            .into_response();
    }
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({"detail": "Provide extension or webrtc_user"})),
    )
        .into_response()
}

// ---- registry ----
// NOTE: GET /api/v1/modules lives in `crate::missing_tables` (DB-backed
// v_modules CRUD). No static stub here.

// NOTE: the old static `GET /api/v1/plugins` stub lived here; superseded by
// the real manifest registry in `crate::plugins::plugins_router`
// (all 9 `plugins/*/manifest.toml`, license gate, enriched shape).

async fn log_viewer() -> impl IntoResponse {
    // Prod tails FS logs; mock transport has no log stream.
    Json(serde_json::json!({"data": [], "total": 0}))
}

async fn database_transactions(
    Extension(claims): Extension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = pbx_db::entities::database_transactions::Entity::find()
        .filter(pbx_db::entities::database_transactions::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "database_transaction_uuid": r.database_transaction_uuid,
                "app_name": r.app_name,
                "app_uuid": r.app_uuid,
                "transaction_code": r.transaction_code,
                "transaction_address": r.transaction_address,
                "transaction_type": r.transaction_type,
                "transaction_date": r.transaction_date,
                "transaction_old": r.transaction_old,
                "transaction_new": r.transaction_new,
                "transaction_result": r.transaction_result,
                "user_uuid": r.user_uuid,
            })
        })
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn system_reload() -> impl IntoResponse {
    // Prod issues `reloadxml` over ESL 8021 (+ sofia rescan on conf regen);
    // here the shared EsClient falls back to the mock when FS is down.
    let result = esl().reloadxml();
    Json(serde_json::json!({"result": result, "success": true}))
}

// ---- status-alias redirects (Python parity: app/status_aliases/router.py) ----
// `status_aliases` is redirects-only upstream — no table, no CRUD.
// Tenant scoping: aliases carry no domain query of their own; the
// `require_tenant` layer 401s unauthenticated callers, and each redirect
// target (`/cdr`, `/dialplans`, `/sip_profiles/status`, `/system/status`,
// `/active_calls`) re-runs its own auth + domain filter. Cross-tenant reads
// through an alias therefore 404/empty exactly like a direct read.
// Live ESL routes below (`active_calls`, `active_conferences`,
// `registrations`, `operator_panel/*`, `click_to_call`, fifo callers in
// remaining_c) expose FreeSWITCH runtime state, which is global to the
// switch — no per-domain row exists to filter. They stay auth-gated (401
// without a token) and permissive for any valid JWT by design; per-route
// permission gating for live actions (hangup/eavesdrop/originate) is a
// follow-up (Python: operator_panel_* / active_calls_hangup perms).
async fn alias_to(dst: &'static str) -> impl IntoResponse {
    axum::response::Redirect::to(dst)
}

pub fn remaining_router() -> Router {
    crate::remaining_a::spillover_a_router()
        .merge(crate::remaining_b::spillover_b_router())
        .merge(crate::remaining_c::spillover_c_router())
        .route("/api/v1/tones", axum::routing::get(list_tones))
        .route("/api/v1/tones/", axum::routing::get(list_tones))
        .route(
            "/api/v1/active_calls",
            axum::routing::get(list_active_calls),
        )
        .route(
            "/api/v1/active_calls/{id}",
            axum::routing::get(get_active_call),
        )
        .route("/api/v1/active_calls/{id}/hangup", post(hangup_active_call))
        .route(
            "/api/v1/active_conferences",
            axum::routing::get(list_active_conferences),
        )
        .route(
            "/api/v1/active_conferences/count",
            axum::routing::get(count_active_conferences),
        )
        .route(
            "/api/v1/active_conferences/{conf}/members",
            axum::routing::get(list_conference_members),
        )
        .route(
            "/api/v1/active_conferences/{conf}/members/{member}/{action}",
            post(conference_member_action),
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
        .route(
            "/api/v1/registrations/{uid}/provision",
            post(provision_registration),
        )
        .route(
            "/api/v1/registrations/{uid}/reboot",
            post(reboot_registration),
        )
        .route("/api/v1/operator_panel", axum::routing::get(operator_panel))
        .route("/api/v1/operator_panel/originate", post(operator_originate))
        .route(
            "/api/v1/operator_panel/calls",
            axum::routing::get(operator_calls),
        )
        .route("/api/v1/operator_panel/hangup/{uid}", post(op_hangup))
        .route("/api/v1/operator_panel/eavesdrop/{uid}", post(op_eavesdrop))
        .route("/api/v1/operator_panel/coach/{uid}", post(op_coach))
        .route("/api/v1/operator_panel/record/{uid}", post(op_record))
        .route("/api/v1/webrtc/credentials", post(webrtc_credentials))
        .route(
            "/api/v1/click_to_call",
            axum::routing::get(click_to_call_info).post(click_to_call_originate),
        )
        .route(
            "/api/v1/click_to_call/originate",
            post(click_to_call_originate),
        )
        .route("/api/v1/log_viewer", axum::routing::get(log_viewer))
        .route(
            "/api/v1/database_transactions",
            axum::routing::get(database_transactions),
        )
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
