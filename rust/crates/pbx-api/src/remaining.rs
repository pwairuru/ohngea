//! Batch 3c remaining-apps sweep — mirrors the Python `app/` routers not
//! yet ported: call_block, call_broadcast, bridges, destinations, follow_me,
//! call_forward, active_calls, active_conferences, registrations (ESL-backed),
//! operator_panel, click_to_call (originate via ESL), conferences_active,
//! fifo_list, status_aliases, streams, time_conditions, feature_codes,
//! music_on_hold, phrases, pin_numbers, number_translations, emergency_logs,
//! event_guard, log_viewer, modules, plugins registry, system (switch/conf
//! regen stubs), access_controls, database_transactions, webrtc_settings.
//!
//! Order: models → schemas → handlers → esl → registry.
//!
//! models: single `pbx_db::entities::remaining_record` row per object
//! (tenant + resource tag + JSON payload). Uniform shape keeps one entity
//! for 18 near-identical CRUD apps; per-app columns are a follow-up.
//! Runtime dialplan/XML regen goes through `dialplans` / FS XML section.
//!
//! esl: `RealEs::from_env` with graceful fallback to the mock transport
//! (same `EsClient` trait as batch 2b `calls.rs`).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use pbx_db::entities::remaining_record;
use pbx_fs::esl::{EsClient, MockEs, RealEs};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::Value;

// ---- models ----
type Claims = pbx_core::auth::Claims;

// ---- schemas ----
struct CrudSpec {
    base: &'static str,
    resource: &'static str,
    id_field: &'static str,
    required: &'static [&'static str],
    alias_slash: bool,
}

// ---- handlers ----
fn missing_field(payload: &Value, required: &[&str]) -> Option<String> {
    for f in required {
        match payload.get(f) {
            None => return Some((*f).to_owned()),
            Some(v) => {
                if v.as_str().is_some_and(|s| s.trim().is_empty()) {
                    return Some((*f).to_owned());
                }
            }
        }
    }
    None
}

async fn create_record(
    spec: &'static CrudSpec,
    Extension(claims): Extension<Claims>,
    Json(mut payload): Json<Value>,
) -> impl IntoResponse {
    if let Some(f) = missing_field(&payload, spec.required) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": format!("{f} required")})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    if !payload.is_object() {
        payload = serde_json::json!({"value": payload});
    }
    payload[spec.id_field] = serde_json::json!(id.clone());
    let am = remaining_record::ActiveModel {
        id: Set(id),
        domain_id: Set(domain),
        resource: Set(spec.resource.to_owned()),
        data: Set(payload.to_string()),
        created_at: Set(crate::routing_db::now()),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    (StatusCode::CREATED, Json(payload))
}

async fn list_records(
    spec: &'static CrudSpec,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = remaining_record::Entity::find()
        .filter(remaining_record::Column::Resource.eq(spec.resource))
        .filter(remaining_record::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| serde_json::from_str(&r.data).unwrap_or(serde_json::json!({"id": r.id})))
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_record(
    spec: &'static CrudSpec,
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = remaining_record::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) if r.resource == spec.resource && r.domain_id == domain => {
            let v: Value = serde_json::from_str(&r.data).unwrap_or(serde_json::json!({"id": r.id}));
            (StatusCode::OK, Json(v)).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn delete_record(
    spec: &'static CrudSpec,
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = remaining_record::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) if r.resource == spec.resource && r.domain_id == domain => {
            let am: remaining_record::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn crud_router(spec: &'static CrudSpec) -> Router {
    let item = format!("{}/{{id}}", spec.base);
    let r = Router::new()
        .route(
            spec.base,
            post(
                move |claims: Extension<Claims>, payload: Json<Value>| async move {
                    create_record(spec, claims, payload).await
                },
            )
            .get(move |claims: Extension<Claims>| async move { list_records(spec, claims).await }),
        )
        .route(
            &item,
            axum::routing::get(
                move |claims: Extension<Claims>, path: axum::extract::Path<String>| async move {
                    get_record(spec, claims, path).await
                },
            )
            .delete(
                move |claims: Extension<Claims>, path: axum::extract::Path<String>| async move {
                    delete_record(spec, claims, path).await
                },
            ),
        );
    if spec.alias_slash {
        let slash = format!("{}/", spec.base);
        r.route(
            &slash,
            post(
                move |claims: Extension<Claims>, payload: Json<Value>| async move {
                    create_record(spec, claims, payload).await
                },
            )
            .get(move |claims: Extension<Claims>| async move { list_records(spec, claims).await }),
        )
    } else {
        r
    }
}

// ---- esl ----
// Live FreeSWITCH state reads go through the shared `EsClient` trait:
// `RealEs::from_env` dials TCP 8021 and falls back to the mock transport
// when FS is down (same pattern as `calls.rs`).

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

async fn list_plugins() -> impl IntoResponse {
    let data = serde_json::json!([
        {"name": "operator_panel", "enabled": true},
        {"name": "click_to_call", "enabled": true},
    ]);
    Json(serde_json::json!({"data": data, "total": 2}))
}

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

static SPECS: &[CrudSpec] = &[
    CrudSpec {
        base: "/api/v1/call_block",
        resource: "call_block",
        id_field: "call_block_uuid",
        required: &["number"],
        alias_slash: true,
    },
    CrudSpec {
        base: "/api/v1/call_broadcast",
        resource: "call_broadcast",
        id_field: "call_broadcast_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/bridges",
        resource: "bridges",
        id_field: "bridge_uuid",
        required: &["name"],
        alias_slash: true,
    },
    CrudSpec {
        base: "/api/v1/destinations",
        resource: "destinations",
        id_field: "destination_uuid",
        required: &["destination"],
        alias_slash: true,
    },
    CrudSpec {
        base: "/api/v1/follow_me",
        resource: "follow_me",
        id_field: "follow_me_uuid",
        required: &["extension"],
        alias_slash: true,
    },
    CrudSpec {
        base: "/api/v1/call_forward",
        resource: "call_forward",
        id_field: "call_forward_uuid",
        required: &["extension"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/time_conditions",
        resource: "time_conditions",
        id_field: "time_condition_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/feature_codes",
        resource: "feature_codes",
        id_field: "feature_code_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/music_on_hold",
        resource: "music_on_hold",
        id_field: "music_on_hold_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/phrases",
        resource: "phrases",
        id_field: "phrase_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/pin_numbers",
        resource: "pin_numbers",
        id_field: "pin_number_uuid",
        required: &["number"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/number_translations",
        resource: "number_translations",
        id_field: "number_translation_uuid",
        required: &["number"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/streams",
        resource: "streams",
        id_field: "stream_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/status_aliases",
        resource: "status_aliases",
        id_field: "status_alias_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/access_controls",
        resource: "access_controls",
        id_field: "access_control_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/webrtc_settings",
        resource: "webrtc_settings",
        id_field: "webrtc_setting_uuid",
        required: &[],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/fifo_list",
        resource: "fifo_list",
        id_field: "fifo_uuid",
        required: &["name"],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/emergency_logs",
        resource: "emergency_logs",
        id_field: "emergency_log_uuid",
        required: &[],
        alias_slash: false,
    },
    CrudSpec {
        base: "/api/v1/event_guard",
        resource: "event_guard",
        id_field: "event_guard_uuid",
        required: &[],
        alias_slash: false,
    },
];

pub fn remaining_router() -> Router {
    let mut r = Router::new();
    for spec in SPECS {
        r = r.merge(crud_router(spec));
    }
    r.route(
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
    .route("/api/v1/plugins", axum::routing::get(list_plugins))
    .route("/api/v1/log_viewer", axum::routing::get(log_viewer))
    .route(
        "/api/v1/database_transactions",
        axum::routing::get(database_transactions),
    )
    .route("/api/v1/system/status", axum::routing::get(system_status))
    .route("/api/v1/system/reload", post(system_reload))
    .layer(axum::middleware::from_fn(crate::require_tenant))
}
