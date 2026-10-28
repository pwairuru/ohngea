//! Dialplans router — mirrors `app/dialplans/router.py`.
//!
//! Types → validation → handlers. Envelope `{data, total}`.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialplan {
    pub dialplan_uuid: String,
    pub name: String,
    pub context: String,
    pub domain: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
struct DialplanCreate {
    name: Option<String>,
    context: Option<String>,
    domain: Option<String>,
    enabled: Option<bool>,
}

static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn store() -> &'static Mutex<HashMap<String, Dialplan>> {
    static STORE: OnceLock<Mutex<HashMap<String, Dialplan>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn create_dialplan(Json(payload): Json<DialplanCreate>) -> impl IntoResponse {
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let rec = Dialplan {
        dialplan_uuid: format!("dp-{id}"),
        name: payload.name.unwrap_or_else(|| "local".to_owned()),
        context: payload.context.unwrap_or_else(|| "default".to_owned()),
        domain: payload.domain.unwrap_or_else(|| "example.com".to_owned()),
        enabled: payload.enabled.unwrap_or(true),
    };
    store()
        .lock()
        .expect("dp lock")
        .insert(rec.dialplan_uuid.clone(), rec.clone());
    (StatusCode::CREATED, Json(serde_json::json!(rec)))
}

async fn list_dialplans() -> impl IntoResponse {
    let items: Vec<Dialplan> = store().lock().expect("dp lock").values().cloned().collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn get_dialplan(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    let map = store().lock().expect("dp lock");
    match map.get(&id) {
        Some(rec) => (StatusCode::OK, Json(serde_json::json!(rec))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

pub fn dialplans_router() -> Router {
    Router::new()
        .route(
            "/api/v1/dialplans",
            post(create_dialplan).get(list_dialplans),
        )
        .route("/api/v1/dialplans/{id}", axum::routing::get(get_dialplan))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
