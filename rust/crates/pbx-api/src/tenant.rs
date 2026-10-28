use axum::http::Request;
use axum::{
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

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
    // Cookie fallback (matches Python app/auth/deps.py accepting httpOnly access_token cookie).
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

pub async fn require_tenant(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = bearer_or_cookie_token(req.headers()).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = pbx_core::auth::decode_test_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    // Per-request RLS scoping (`app/db.py::set_tenant` parity): best-effort
    // `SET LOCAL app.domain_uuid` on the shared pool + stash claims for
    // handler-level tenant txns (FORCE RLS requires txn-scoped GUC).
    // No-op on sqlite unit tests; never fails the request.
    let db = crate::routing_db::conn().await;
    crate::routing_db::set_tenant(db, &claims).await;
    // Stash verified claims for downstream handlers / db layer (SET LOCAL app.domain_uuid).
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
