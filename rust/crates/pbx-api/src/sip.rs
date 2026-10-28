//! SIP profiles router — mirrors `app/sip_profiles/router.py`
//! (11 endpoints): profiles CRUD (4) + domains list/create (2) +
//! domains patch/delete (2) + settings list/create (2) + settings
//! patch (1). Generic JSON stores keep this minimal; envelope
//! `{data, total}` on collections.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn profiles() -> &'static Mutex<HashMap<String, serde_json::Value>> {
    static S: OnceLock<Mutex<HashMap<String, serde_json::Value>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn domains() -> &'static Mutex<HashMap<String, serde_json::Value>> {
    static S: OnceLock<Mutex<HashMap<String, serde_json::Value>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn settings() -> &'static Mutex<HashMap<String, serde_json::Value>> {
    static S: OnceLock<Mutex<HashMap<String, serde_json::Value>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn new_id(prefix: &str) -> String {
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("{prefix}-{id}")
}

async fn create_profile(Json(mut payload): Json<serde_json::Value>) -> impl IntoResponse {
    let id = new_id("sip");
    payload["sip_profile_uuid"] = serde_json::json!(id);
    if payload.get("sip_port").is_none() {
        payload["sip_port"] = serde_json::json!(5060);
    }
    profiles()
        .lock()
        .expect("sip lock")
        .insert(id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn list_profiles() -> impl IntoResponse {
    let items: Vec<serde_json::Value> = profiles()
        .lock()
        .expect("sip lock")
        .values()
        .cloned()
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn get_profile(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    match profiles().lock().expect("sip lock").get(&id).cloned() {
        Some(v) => (StatusCode::OK, Json(v)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_profile(
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut map = profiles().lock().expect("sip lock");
    match map.get_mut(&id) {
        Some(v) => {
            for (k, val) in patch.as_object().cloned().unwrap_or_default() {
                v[k] = val;
            }
            // Vars/sofia XML regen trigger would go here (DB-backed prod).
            (StatusCode::OK, Json(v.clone())).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn delete_profile(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    match profiles().lock().expect("sip lock").remove(&id) {
        Some(_) => StatusCode::NO_CONTENT.into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn list_profile_domains(
    axum::extract::Path(pid): axum::extract::Path<String>,
) -> impl IntoResponse {
    let items: Vec<serde_json::Value> = domains()
        .lock()
        .expect("dom lock")
        .values()
        .filter(|v| v.get("sip_profile_uuid").and_then(|p| p.as_str()) == Some(pid.as_str()))
        .cloned()
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn create_profile_domain(
    axum::extract::Path(pid): axum::extract::Path<String>,
    Json(mut payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = new_id("spd");
    payload["sip_profile_domain_uuid"] = serde_json::json!(id);
    payload["sip_profile_uuid"] = serde_json::json!(pid);
    domains()
        .lock()
        .expect("dom lock")
        .insert(id, payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn patch_profile_domain(
    axum::extract::Path((pid, did)): axum::extract::Path<(String, String)>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut map = domains().lock().expect("dom lock");
    match map.get_mut(&did) {
        Some(v) if v.get("sip_profile_uuid").and_then(|p| p.as_str()) == Some(pid.as_str()) => {
            for (k, val) in patch.as_object().cloned().unwrap_or_default() {
                v[k] = val;
            }
            (StatusCode::OK, Json(v.clone())).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn delete_profile_domain(
    axum::extract::Path((pid, did)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let mut map = domains().lock().expect("dom lock");
    let owned = map.get(&did).cloned();
    match owned {
        Some(v) if v.get("sip_profile_uuid").and_then(|p| p.as_str()) == Some(pid.as_str()) => {
            map.remove(&did);
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn list_profile_settings(
    axum::extract::Path(pid): axum::extract::Path<String>,
) -> impl IntoResponse {
    let items: Vec<serde_json::Value> = settings()
        .lock()
        .expect("set lock")
        .values()
        .filter(|v| v.get("sip_profile_uuid").and_then(|p| p.as_str()) == Some(pid.as_str()))
        .cloned()
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn create_profile_setting(
    axum::extract::Path(pid): axum::extract::Path<String>,
    Json(mut payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = new_id("sps");
    payload["sip_profile_setting_uuid"] = serde_json::json!(id);
    payload["sip_profile_uuid"] = serde_json::json!(pid);
    settings()
        .lock()
        .expect("set lock")
        .insert(id, payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn patch_profile_setting(
    axum::extract::Path((pid, sid)): axum::extract::Path<(String, String)>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut map = settings().lock().expect("set lock");
    match map.get_mut(&sid) {
        Some(v) if v.get("sip_profile_uuid").and_then(|p| p.as_str()) == Some(pid.as_str()) => {
            for (k, val) in patch.as_object().cloned().unwrap_or_default() {
                v[k] = val;
            }
            (StatusCode::OK, Json(v.clone())).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

/// Names of enabled profiles for sofia XML regen from DB.
pub fn profile_names() -> Vec<String> {
    profiles()
        .lock()
        .expect("sip lock")
        .values()
        .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(str::to_owned))
        .collect()
}

pub fn sip_router() -> Router {
    Router::new()
        .route(
            "/api/v1/sip_profiles",
            post(create_profile).get(list_profiles),
        )
        .route(
            "/api/v1/sip_profiles/{id}",
            axum::routing::get(get_profile)
                .patch(patch_profile)
                .delete(delete_profile),
        )
        .route(
            "/api/v1/sip_profiles/{id}/domains",
            get(list_profile_domains).post(create_profile_domain),
        )
        .route(
            "/api/v1/sip_profiles/{id}/domains/{did}",
            axum::routing::patch(patch_profile_domain).delete(delete_profile_domain),
        )
        .route(
            "/api/v1/sip_profiles/{id}/settings",
            get(list_profile_settings).post(create_profile_setting),
        )
        .route(
            "/api/v1/sip_profiles/{id}/settings/{sid}",
            axum::routing::patch(patch_profile_setting),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
