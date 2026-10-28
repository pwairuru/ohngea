//! Extensions router — mirrors `app/extensions/router.py`.
//!
//! Types → validation → handlers. Validation + password auto-gen reuse
//! `pbx_db::extensions` (mirrors `models.py` + `service.py`).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extension {
    pub extension: String,
    pub password: String,
    pub domain: String,
}

#[derive(Debug, Deserialize)]
struct ExtensionCreate {
    extension: String,
    password: Option<String>,
    domain: Option<String>,
}

/// SWAP POINT: process-global in-memory store (tests + local dev only).
/// Prod wiring is SeaORM/postgres via `pbx_db::entities::extensions`
/// (see `crate::routing_db::conn`); `find_extension` keeps its signature
/// so the FS XML directory handler needs no change at swap time.
fn store() -> &'static Mutex<HashMap<String, Extension>> {
    static STORE: OnceLock<Mutex<HashMap<String, Extension>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn key(ext: &str, domain: &str) -> String {
    format!("{ext}@{domain}")
}

fn validate(ext: &str) -> Result<(), String> {
    pbx_db::extensions::validate_extension(ext)
}

async fn create_extension(Json(payload): Json<ExtensionCreate>) -> impl IntoResponse {
    let ext = payload.extension.trim().to_owned();
    if let Err(msg) = validate(&ext) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": msg})),
        );
    }
    let domain = payload.domain.unwrap_or_else(|| "example.com".to_owned());
    let password = pbx_db::extensions::ensure_password(payload.password.as_deref());
    let rec = Extension {
        extension: ext.clone(),
        password,
        domain: domain.clone(),
    };
    store()
        .lock()
        .expect("ext lock")
        .insert(key(&ext, &domain), rec.clone());
    (StatusCode::CREATED, Json(serde_json::json!(rec)))
}

async fn list_extensions() -> impl IntoResponse {
    let items: Vec<Extension> = store()
        .lock()
        .expect("ext lock")
        .values()
        .cloned()
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

/// Lookup used by the FS XML directory handler (DB-backed in prod,
/// in-memory here; sqlite-test / postgres-runtime wiring is the
/// follow-up once SeaORM is wired into pbx-api).
pub fn find_extension(user: &str, domain: &str) -> Option<Extension> {
    // Exact domain match first, then IP-fallback: any domain for user.
    let map = store().lock().expect("ext lock");
    if let Some(rec) = map.get(&key(user, domain)) {
        return Some(rec.clone());
    }
    map.values().find(|e| e.extension == user).cloned()
}

pub fn extensions_router() -> Router {
    Router::new()
        .route(
            "/api/v1/extensions",
            post(create_extension).get(list_extensions),
        )
        // Trailing-slash alias (Python router accepts both).
        .route(
            "/api/v1/extensions/",
            post(create_extension).get(list_extensions),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
