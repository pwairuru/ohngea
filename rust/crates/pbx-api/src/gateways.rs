//! Gateways router — mirrors `app/gateways/router.py`.
//!
//! Types → validation → handlers → esl. Password masked `***` on GET
//! (mirrors `mask_secret`); create/update best-effort `sofia_rescan`.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gateway {
    pub gateway_uuid: String,
    pub gateway: String,
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    pub profile: String,
    pub context: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
struct GatewayCreate {
    gateway: Option<String>,
    name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    profile: Option<String>,
    context: Option<String>,
}

static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn store() -> &'static Mutex<HashMap<String, Gateway>> {
    static STORE: OnceLock<Mutex<HashMap<String, Gateway>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn masked(mut g: Gateway) -> Gateway {
    // Mask password *** on read (mirrors mask_secret in router.py).
    if g.password.as_deref().is_some_and(|p| !p.is_empty()) {
        g.password = Some("***".to_owned());
    }
    g
}

async fn create_gateway(Json(payload): Json<GatewayCreate>) -> impl IntoResponse {
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let profile = payload
        .profile
        .unwrap_or_else(|| pbx_db::gateways::default_profile().to_owned());
    let rec = Gateway {
        gateway_uuid: format!("gw-{id}"),
        gateway: payload
            .gateway
            .or(payload.name)
            .unwrap_or_else(|| "gw".to_owned()),
        username: payload.username,
        password: payload.password,
        profile: profile.clone(),
        context: payload
            .context
            .unwrap_or_else(|| pbx_db::gateways::default_context().to_owned()),
        enabled: true,
    };
    store()
        .lock()
        .expect("gw lock")
        .insert(rec.gateway_uuid.clone(), rec.clone());
    // Best-effort ESL sofia profile rescan (legacy gateway_edit.php);
    // mocked transport here, real TCP 8021 prod-side via RealEs.
    let _rescan_reply = pbx_fs::esl::EsClient::sofia_rescan(&pbx_fs::esl::MockEsClient, &profile);
    (StatusCode::CREATED, Json(serde_json::json!(masked(rec))))
}

async fn list_gateways() -> impl IntoResponse {
    let items: Vec<Gateway> = store()
        .lock()
        .expect("gw lock")
        .values()
        .cloned()
        .map(masked)
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn get_gateway(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    let map = store().lock().expect("gw lock");
    match map.get(&id) {
        Some(rec) => (StatusCode::OK, Json(serde_json::json!(masked(rec.clone())))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

pub fn gateways_router() -> Router {
    Router::new()
        .route("/api/v1/gateways", post(create_gateway).get(list_gateways))
        .route("/api/v1/gateways/{id}", axum::routing::get(get_gateway))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
