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
        // Scheme is case-insensitive per RFC 7235; accept `bearer`/`Bearer`.
        if auth.len() > 7 && auth[..7].eq_ignore_ascii_case("bearer ") {
            let token = auth[7..].trim();
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

/// Permission check parity with Python `app/auth/deps.py::require_permission`.
/// `admin`, `superadmin` and `*` bypass every check; otherwise the exact
/// permission name must be present in the JWT `permissions` claim.
#[must_use]
pub fn has_permission(claims: &pbx_core::auth::Claims, perm: &str) -> bool {
    claims
        .permissions
        .iter()
        .any(|p| p == "admin" || p == "superadmin" || p == "*" || p == perm)
}

/// 403 body parity with Python `HTTPException(403, "Missing permission: {perm}")`.
pub fn forbidden(perm: &str) -> (StatusCode, axum::Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        axum::Json(serde_json::json!({"detail": format!("Missing permission: {perm}")})),
    )
}

/// Handler guard: `Some(403)` when the claims lack `perm`, `None` when allowed.
/// Mirror of Python `Depends(require_permission(perm))` for axum handlers.
///
/// Naming parity: `require_permission(perm)` is the Python `app/auth/deps.py`
/// dependency factory; in axum the same check runs inline at the top of each
/// handler via `deny_if_missing` (middleware can't vary the perm per route).
/// Prefer `require_permission` in new code for grep parity with Python.
#[must_use]
pub fn deny_if_missing(
    claims: &pbx_core::auth::Claims,
    perm: &str,
) -> Option<(StatusCode, axum::Json<serde_json::Value>)> {
    if has_permission(claims, perm) {
        None
    } else {
        Some(forbidden(perm))
    }
}

/// Alias of [`deny_if_missing`] with the Python `require_permission` name.
#[must_use]
pub fn require_permission(
    claims: &pbx_core::auth::Claims,
    perm: &str,
) -> Option<(StatusCode, axum::Json<serde_json::Value>)> {
    deny_if_missing(claims, perm)
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
