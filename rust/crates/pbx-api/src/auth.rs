use axum::{
    http::{header, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct LoginRequest {
    #[allow(dead_code)]
    username: Option<String>,
    #[allow(dead_code)]
    password: Option<String>,
    #[allow(dead_code)]
    domain: Option<String>,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    access_token: String,
}

async fn login_handler(Json(_req): Json<LoginRequest>) -> Json<LoginResponse> {
    Json(LoginResponse {
        access_token: pbx_core::auth::encode_test_token(),
    })
}

async fn me_handler(headers: axum::http::HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = auth
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = pbx_core::auth::decode_test_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(Json(
        json!({"sub": claims.sub, "domain_id": claims.domain_id, "perms": claims.perms}),
    ))
}

pub fn auth_router() -> Router {
    Router::new()
        .route("/auth/login", post(login_handler))
        .route("/api/v1/auth/login", post(login_handler))
        .route("/auth/me", get(me_handler))
        .route("/api/v1/auth/me", get(me_handler))
}
