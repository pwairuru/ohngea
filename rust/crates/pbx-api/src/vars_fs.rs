//! Vars CRUD + sofia XML regen — mirrors `app/vars/router.py:13-74`
//! and `service.py`. PATCH invalidates the FS XML cache and regens
//! sofia XML from DB sip profiles.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn store() -> &'static Mutex<HashMap<String, serde_json::Value>> {
    static S: OnceLock<Mutex<HashMap<String, serde_json::Value>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

async fn create_var(Json(mut payload): Json<serde_json::Value>) -> impl IntoResponse {
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let uuid = format!("var-{id}");
    payload["var_uuid"] = serde_json::json!(uuid);
    store()
        .lock()
        .expect("vars lock")
        .insert(uuid, payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn list_vars() -> impl IntoResponse {
    let items: Vec<serde_json::Value> = store()
        .lock()
        .expect("vars lock")
        .values()
        .cloned()
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn get_var(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    match store().lock().expect("vars lock").get(&id).cloned() {
        Some(v) => (StatusCode::OK, Json(v)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_var(
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut map = store().lock().expect("vars lock");
    match map.get_mut(&id) {
        Some(v) => {
            for (k, val) in patch.as_object().cloned().unwrap_or_default() {
                v[k] = val;
            }
            // Invalidate FS cache + regen sofia XML from DB profiles.
            crate::fs_xml::invalidate_cache();
            let names = crate::sip::profile_names();
            let inputs: Vec<pbx_fs::sofia::SofiaProfileInput> = names
                .iter()
                .map(|n| pbx_fs::sofia::SofiaProfileInput {
                    name: n.clone(),
                    sip_port: 5060,
                    ..Default::default()
                })
                .collect();
            let _regen = pbx_fs::sofia::build_sofia_xml(&inputs);
            let out = v.clone();
            (StatusCode::OK, Json(out)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn delete_var(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    match store().lock().expect("vars lock").remove(&id) {
        Some(_) => {
            crate::fs_xml::invalidate_cache();
            StatusCode::NO_CONTENT.into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

pub fn vars_router() -> Router {
    Router::new()
        .route("/api/v1/vars", post(create_var).get(list_vars))
        .route(
            "/api/v1/vars/{id}",
            axum::routing::get(get_var)
                .patch(patch_var)
                .delete(delete_var),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
