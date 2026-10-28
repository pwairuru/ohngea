use axum::{
    extract::rejection::JsonRejection,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    domain: Option<String>,
    #[serde(default, alias = "domain_name")]
    domain_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

fn unauthorized() -> (StatusCode, &'static str) {
    (StatusCode::UNAUTHORIZED, "Invalid credentials")
}

async fn login_handler(
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let Json(req) = payload.map_err(|_| unauthorized())?;
    // domain / domain_name accepted but optional (parity with Python schema); unused in stub.
    let _domain = req.domain.or(req.domain_name);
    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return Err(unauthorized());
    }
    let token = pbx_core::auth::encode_test_token();
    let cookie = format!("access_token={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age=3600");
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| unauthorized())?,
    );
    let body = Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    });
    Ok((headers, body))
}

fn bearer_or_cookie_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("access_token=") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

async fn me_handler(headers: HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = bearer_or_cookie_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = pbx_core::auth::decode_test_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(Json(
        serde_json::json!({"sub": claims.sub, "domain_id": claims.domain_id, "perms": claims.perms}),
    ))
}

pub fn auth_router() -> Router {
    Router::new()
        .route("/auth/login", post(login_handler))
        .route("/api/v1/auth/login", post(login_handler))
        .route("/auth/me", get(me_handler))
        .route("/api/v1/auth/me", get(me_handler))
}
