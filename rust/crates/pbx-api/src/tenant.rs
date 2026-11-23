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
/// Only `superadmin` and `*` bypass every check; otherwise the exact
/// permission name must be present in the JWT `permissions` claim.
/// (`admin` is a group name / catalog entry, never a bypass.)
#[must_use]
pub fn has_permission(claims: &pbx_core::auth::Claims, perm: &str) -> bool {
    claims
        .permissions
        .iter()
        .any(|p| p == "superadmin" || p == "*" || p == perm)
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

/// Any-of variant of [`deny_if_missing`]: allows shared endpoints that serve
/// several PHP apps (e.g. `/api/v1/dialplans` backing dialplans +
/// inbound/outbound routes, `/api/v1/cdr` backing xml_cdr views) to accept
/// any of the PHP-equivalent permission names. Error names the first perm.
#[must_use]
pub fn deny_if_missing_any(
    claims: &pbx_core::auth::Claims,
    perms: &[&str],
) -> Option<(StatusCode, axum::Json<serde_json::Value>)> {
    if perms.iter().any(|p| has_permission(claims, p)) {
        None
    } else {
        Some(forbidden(perms.first().copied().unwrap_or("unknown")))
    }
}

/// `?show=all` cross-domain scope (PHP `show == 'all'` + `X_all` parity).
/// True ONLY when the caller explicitly requests `show=all` AND holds the
/// resource's `_all` permission. Default (no param, other values, or perm
/// missing) stays tenant-scoped. Callers skip the `domain_id` query filter
/// and set transaction-local `app.domain_all = '1'` (unlocks the
/// `tenant_all_*` RLS policies) on this path only.
#[must_use]
pub fn wants_all(claims: &pbx_core::auth::Claims, show: Option<&str>, all_perm: &str) -> bool {
    matches!(show.map(str::trim), Some("all")) && has_permission(claims, all_perm)
}

/// `{base}_domain` scope (PHP `gateway_domain` parity): holder additionally
/// sees global (NULL-domain) rows alongside their own tenant's rows.
#[must_use]
pub fn sees_domain_nulls(claims: &pbx_core::auth::Claims, domain_perm: &str) -> bool {
    has_permission(claims, domain_perm)
}

/// [`wants_all`] accepting any of several `_all` permissions (shared
/// endpoints serving several PHP apps, e.g. `/api/v1/recordings` backing
/// both `recording_all` and `call_recording_all` pages).
#[must_use]
pub fn wants_all_any(
    claims: &pbx_core::auth::Claims,
    show: Option<&str>,
    all_perms: &[&str],
) -> bool {
    matches!(show.map(str::trim), Some("all"))
        && all_perms.iter().any(|p| has_permission(claims, p))
}

/// Minimal standard-alphabet base64 decode (Basic-auth only, no new deps).
fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut acc: u32 = 0;
    let mut bits = 0u8;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ if c.is_ascii_whitespace() => continue,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8 & 0xff);
        }
    }
    Some(out)
}

/// Resolve the FS-requested domain (`directory`: `user@domain` in
/// `key_value`; `dialplan`/`configuration`: context name) to its UUID.
/// Unknown → nil UUID (RLS then matches nothing; handler renders fallback).
async fn fs_domain_id(query: &str) -> uuid::Uuid {
    let kv = query.split('&').find_map(|p| {
        let (k, v) = p.split_once('=')?;
        (k == "key_value").then(|| v.to_owned())
    });
    let domain = kv
        .as_deref()
        .map(|v| v.rsplit('@').next().unwrap_or(v))
        .unwrap_or("");
    // URL-decode the domain (`%40` etc.) with a tiny in-place decoder.
    let mut decoded = Vec::with_capacity(domain.len());
    let b = domain.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                decoded.push(h << 4 | l);
                i += 3;
                continue;
            }
        }
        decoded.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    let name = String::from_utf8_lossy(&decoded);
    let db = crate::routing_db::conn().await;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    if let Ok(Some(row)) = pbx_db::entities::domains::Entity::find()
        .filter(pbx_db::entities::domains::Column::DomainName.eq(name.as_ref()))
        .one(db)
        .await
    {
        return row.domain_id;
    }
    uuid::Uuid::nil()
}

fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// FS internal guard: Bearer JWT (normal tenant path) OR Basic
/// `FS_XML_USER`:`FS_XML_PASSWORD` (FreeSWITCH mod_xml_curl sends no JWT;
/// Python parity `app/freeswitch/xml_router.py::check_fs_auth`).
pub async fn require_fs_or_tenant(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(token) = bearer_or_cookie_token(req.headers()) {
        let claims =
            pbx_core::auth::decode_test_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
        let db = crate::routing_db::conn().await;
        crate::routing_db::set_tenant(db, &claims).await;
        req.extensions_mut().insert(claims);
        return Ok(next.run(req).await);
    }
    let raw = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // Scheme is case-insensitive per RFC 7235.
    let b64 = if raw.len() > 6 && raw[..6].eq_ignore_ascii_case("basic ") {
        raw[6..].trim()
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let user = std::env::var("FS_XML_USER").unwrap_or_else(|_| "freeswitch".to_owned());
    let pass = std::env::var("FS_XML_PASSWORD").unwrap_or_else(|_| "ClueCon".to_owned());
    let want = format!("{user}:{pass}");
    let ok = b64_decode(b64)
        .map(|b| b == want.as_bytes())
        .unwrap_or(false);
    // Constant-time-ish: never short-circuit on env presence.
    if !ok {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let query = req.uri().query().unwrap_or("").to_owned();
    let domain_id = fs_domain_id(&query).await.to_string();
    let claims = pbx_core::auth::Claims {
        sub: "freeswitch".to_string(),
        domain_id: domain_id.clone(),
        username: "freeswitch".to_string(),
        permissions: vec!["*".to_string()],
        exp: 0,
        iat: 0,
        jti: None,
    };
    let db = crate::routing_db::conn().await;
    crate::routing_db::set_tenant(db, &claims).await;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
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
    // Write-path parent ensure (Task 2 companion): every tenant FK target
    // references `domains`, so POST/PUT/PATCH/DELETE materialize the caller's
    // own row (insert-or-ignore) to satisfy FK integrity. Reads (GET/HEAD)
    // NEVER insert — unknown tenants resolve to empty/404, never conjured
    // rows. Scoped to the caller's own tenant; no cross-tenant effect.
    // `has_permission`/`deny_if_missing` logic untouched (Task 1 preserved).
    if matches!(
        req.method(),
        &axum::http::Method::POST
            | &axum::http::Method::PUT
            | &axum::http::Method::PATCH
            | &axum::http::Method::DELETE
    ) {
        crate::routing_db::ensure_tenant_domain(db, &claims).await;
    }
    // Stash verified claims for downstream handlers / db layer (SET LOCAL app.domain_uuid).
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
