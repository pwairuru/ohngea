//! Calls router — ESL originate/kill over the mocked transport.
//!
//! Types → validation → handlers → esl. Uses `RealEs::from_env` with
//! graceful fallback to the mock when FS is down.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use pbx_fs::esl::{EsClient, RealEs};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OriginateReq {
    from: String,
    to: String,
}

fn client() -> RealEs {
    RealEs::from_env()
}

async fn originate(Json(payload): Json<OriginateReq>) -> impl IntoResponse {
    if payload.from.trim().is_empty() || payload.to.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "from/to required"})),
        );
    }
    let call_id = client().originate(&payload.from, &payload.to);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"call_id": call_id, "from": payload.from, "to": payload.to})),
    )
}

async fn channels() -> impl IntoResponse {
    let chans = pbx_fs::esl::mock_channels();
    let data: Vec<serde_json::Value> = chans
        .iter()
        .map(|c| serde_json::json!({"call_id": c.uuid, "from": c.from, "to": c.to, "state": c.state}))
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn kill(axum::extract::Path(call_id): axum::extract::Path<String>) -> impl IntoResponse {
    if client().kill(&call_id) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "call not found"})),
        )
            .into_response()
    }
}

pub fn calls_router() -> Router {
    Router::new()
        .route("/api/v1/calls/originate", post(originate))
        .route("/api/v1/calls/channels", axum::routing::get(channels))
        .route("/api/v1/calls/{id}", axum::routing::delete(kill))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
