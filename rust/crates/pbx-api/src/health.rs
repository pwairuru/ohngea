use axum::{routing::get, Json, Router};
use serde_json::json;

pub fn health_router() -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(json!({"status":"ok","version":"0.1.0"})) }),
        )
        .route(
            "/api/v1/health",
            get(|| async { Json(json!({"status":"ok","version":"0.1.0"})) }),
        )
}
