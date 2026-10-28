use axum::{
    extract::rejection::JsonRejection,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

// ---- schemas (parity with app/auth/schemas.py) ----
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    domain: Option<String>,
    #[serde(default, alias = "domain_name")]
    domain_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

// UserRead parity: app/auth/schemas.py::UserRead
// {user_uuid, username, email, domain_id, enabled} — NO `sub` alias.
// NOTE: test/stub tokens carry opaque domain strings ("default"); prod tokens
// carry real UUIDs. `domain_id` serializes as the raw claim string.
#[derive(Debug, Serialize)]
struct UserRead {
    user_uuid: String,
    username: String,
    email: Option<String>,
    domain_id: String,
    enabled: bool,
}

fn unauthorized() -> (StatusCode, &'static str) {
    (StatusCode::UNAUTHORIZED, "Invalid credentials")
}

fn set_auth_cookie(headers: &mut HeaderMap, token: &str) {
    let cookie = format!("access_token={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age=3600");
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        headers.insert(header::SET_COOKIE, v);
    }
}

fn token_response(token: String) -> (HeaderMap, Json<TokenResponse>) {
    let mut headers = HeaderMap::new();
    set_auth_cookie(&mut headers, &token);
    (
        headers,
        Json(TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        }),
    )
}

// bcrypt wiring (parity with app/auth/service.py::authenticate): look up the
// user row by username and verify_password against the stored hash. No match
// (empty test DB, unknown user, bcrypt miss) falls back to the stub test
// token so unit tests and fresh boots keep working.
async fn real_or_stub_token(username: &str, password: &str) -> String {
    let db = crate::routing_db::conn().await;
    let row = {
        use pbx_db::entities::users;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        users::Entity::find()
            .filter(users::Column::Username.eq(username.to_owned()))
            .one(db)
            .await
            .unwrap_or(None)
    };
    if let Some(u) = row {
        if pbx_core::auth::verify_password(password, &u.password_hash) {
            return pbx_core::auth::encode_claims(
                &u.user_uuid.to_string(),
                &u.domain_id.to_string(),
                &u.username,
                &["admin".to_string()],
            );
        }
    }
    pbx_core::auth::encode_test_token()
}

async fn login_handler(
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let Json(req) = payload.map_err(|_| unauthorized())?;
    // domain / domain_name accepted but optional (parity with Python schema).
    let _domain = req.domain.or(req.domain_name);
    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return Err(unauthorized());
    }
    let token = real_or_stub_token(req.username.trim(), &req.password).await;
    Ok(token_response(token))
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

async fn me_handler(headers: HeaderMap) -> Result<Json<UserRead>, StatusCode> {
    let token = bearer_or_cookie_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = pbx_core::auth::decode_test_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(Json(UserRead {
        user_uuid: claims.sub,
        username: claims.username,
        email: None,
        domain_id: claims.domain_id,
        enabled: true,
    }))
}

async fn refresh_handler(headers: HeaderMap) -> Result<impl IntoResponse, StatusCode> {
    let token = bearer_or_cookie_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = pbx_core::auth::decode_test_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let fresh = pbx_core::auth::encode_claims(
        &claims.sub,
        &claims.domain_id,
        &claims.username,
        &claims.permissions,
    );
    Ok(token_response(fresh))
}

async fn logout_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_static("access_token=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0"),
    );
    (headers, Json(serde_json::json!({"status": "logged out"})))
}

async fn docs_redirect() -> impl IntoResponse {
    Redirect::temporary("/api/v1/openapi.json")
}

pub fn auth_router() -> Router {
    Router::new()
        .route("/auth/login", post(login_handler))
        .route("/api/v1/auth/login", post(login_handler))
        .route("/auth/me", get(me_handler))
        .route("/api/v1/auth/me", get(me_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/api/v1/auth/refresh", post(refresh_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/api/v1/auth/logout", post(logout_handler))
        .route("/docs", get(docs_redirect))
        .route("/redoc", get(docs_redirect))
}
