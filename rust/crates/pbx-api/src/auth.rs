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
    /// TOTP code for the second-factor step (PHP totp.php parity: password
    /// + `authentication_code` in one login when the user already knows it).
    #[serde(default)]
    code: Option<String>,
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

// bcrypt wiring (parity with app/auth/service.py::authenticate): the user row
// is looked up scoped to the resolved domain and verified against the
// stored hash. Debug keeps the historical stub fallback (empty test DB,
// unit tests, fresh boots); release returns None → 401 unless a real user
// verifies.
//
// Domain resolution: explicit `domain`/`domain_name` wins; otherwise a
// `user@domain` suffix is split off. The scope accepts a UUID or a
// domain_name; unknown → None (callers answer 401, never stub).
// No global username-only lookup happens once a scope was requested.
//
// Second factor (PHP `core/authentication/resources/classes/plugins/totp.php`
// parity): when `user_totp_secret` is set the password step returns a
// short-lived `mfa:pending` challenge instead of a session token; the client
// completes login via `POST /auth/mfa/verify` (or single-step with `code`).
async fn resolve_domain_id(
    db: &sea_orm::DatabaseConnection,
    domain: &str,
) -> Option<uuid::Uuid> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let t = domain.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(id) = t.parse::<uuid::Uuid>() {
        // UUID scope must exist, otherwise the domain is unknown → 401.
        if pbx_db::entities::domains::Entity::find_by_id(id)
            .one(db)
            .await
            .unwrap_or(None)
            .is_some()
        {
            return Some(id);
        }
        return None;
    }
    if let Ok(row) = pbx_db::entities::domains::Entity::find()
        .filter(pbx_db::entities::domains::Column::DomainName.eq(t.to_owned()))
        .one(db)
        .await
    {
        if let Some(r) = row {
            return Some(r.domain_id);
        }
    }
    // Legacy opaque test scope ("default"): deterministic UUIDv5 mapping
    // (routing_db::domain parity) so stub logins keep working pre-bootstrap.
    if t == "default" {
        return Some(uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, t.as_bytes()));
    }
    None
}

/// Real permissions for `user` (Python
/// `app/auth/service.py::get_user_permissions` parity): permission names via
/// `user_groups` → `group_permissions` → `permissions`, plus group names so
/// a `superadmin` group grants the bypass.
async fn user_permissions(
    db: &sea_orm::DatabaseConnection,
    user_uuid: uuid::Uuid,
) -> Vec<String> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let mems = pbx_db::entities::user_groups::Entity::find()
        .filter(pbx_db::entities::user_groups::Column::UserUuid.eq(user_uuid))
        .all(db)
        .await
        .unwrap_or_default();
    if mems.is_empty() {
        return Vec::new();
    }
    let gids: Vec<uuid::Uuid> = mems.iter().map(|m| m.group_uuid).collect();
    let groups = pbx_db::entities::groups::Entity::find()
        .filter(pbx_db::entities::groups::Column::GroupUuid.is_in(gids.clone()))
        .all(db)
        .await
        .unwrap_or_default();
    let mut out: Vec<String> = groups.into_iter().map(|g| g.name).collect();
    let gps = pbx_db::entities::group_permissions::Entity::find()
        .filter(pbx_db::entities::group_permissions::Column::GroupUuid.is_in(gids))
        .all(db)
        .await
        .unwrap_or_default();
    if !gps.is_empty() {
        let pids: Vec<uuid::Uuid> = gps.iter().map(|gp| gp.permission_uuid).collect();
        let perms = pbx_db::entities::permissions::Entity::find()
            .filter(pbx_db::entities::permissions::Column::PermissionUuid.is_in(pids))
            .all(db)
            .await
            .unwrap_or_default();
        out.extend(perms.into_iter().map(|p| p.name));
    }
    out.sort();
    out.dedup();
    out
}

async fn find_user_scoped(
    db: &sea_orm::DatabaseConnection,
    bare: &str,
    domain_id: uuid::Uuid,
) -> Option<pbx_db::entities::users::Model> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    pbx_db::entities::users::Entity::find()
        .filter(pbx_db::entities::users::Column::Username.eq(bare.to_owned()))
        .filter(pbx_db::entities::users::Column::DomainId.eq(domain_id))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn user_exists_global(db: &sea_orm::DatabaseConnection, bare: &str) -> bool {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    pbx_db::entities::users::Entity::find()
        .filter(pbx_db::entities::users::Column::Username.eq(bare.to_owned()))
        .one(db)
        .await
        .unwrap_or(None)
        .is_some()
}

fn full_token_for(user: &pbx_db::entities::users::Model, perms: &[String]) -> String {
    pbx_core::auth::encode_claims(
        &user.user_uuid.to_string(),
        &user.domain_id.to_string(),
        &user.username,
        perms,
    )
}

async fn login_handler(
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let Json(req) = payload.map_err(|_| unauthorized())?;
    // domain / domain_name accepted but optional (parity with Python schema).
    let domain = req.domain.or(req.domain_name);
    if req.username.trim().is_empty() || req.password.trim().is_empty() {
        return Err(unauthorized());
    }
    let username_raw = req.username.trim().to_owned();
    // Explicit domain wins; otherwise split `user@domain`. (No split when
    // explicit is present so email-shaped usernames keep working.)
    let (bare, suffix) = match domain
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(_) => (username_raw.clone(), None),
        None => match username_raw.rsplit_once('@') {
            Some((b, s)) if !b.is_empty() && !s.trim().is_empty() => {
                (b.to_owned(), Some(s.trim().to_owned()))
            }
            _ => (username_raw.clone(), None),
        },
    };
    let domain_str: Option<String> = domain
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .or(suffix);
    let db = crate::routing_db::conn().await;
    // Unknown scope → 401 outright (never stub): the caller named a domain
    // that does not exist.
    let scope: Option<uuid::Uuid> = match domain_str {
        Some(ref d) => match resolve_domain_id(db, d).await {
            Some(id) => Some(id),
            None => return Err(unauthorized()),
        },
        None => None,
    };
    // Scoped query once a scope was requested — never a global username-only
    // lookup followed by a post-filter (cross-tenant first-row leak).
    // Without any scope, keep the historical global lookup (single-tenant
    // logins, empty-DB boots).
    let found = match scope {
        Some(id) => find_user_scoped(db, &bare, id).await,
        None => {
            use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
            pbx_db::entities::users::Entity::find()
                .filter(pbx_db::entities::users::Column::Username.eq(bare.clone()))
                .one(db)
                .await
                .unwrap_or(None)
        }
    };
    // Real-user path (enforces enabled + domain scope + TOTP second factor).
    if let Some(u) = found {
        if !u.enabled || !pbx_core::auth::verify_password(&req.password, &u.password_hash) {
            return Err(unauthorized());
        }
        let perms = user_permissions(db, u.user_uuid).await;
        // Second factor enrolled: single-step when `code` verifies,
        // otherwise issue the 202 challenge for `/auth/mfa/verify`.
        if let Some(secret) = u.user_totp_secret.as_deref().filter(|s| !s.is_empty()) {
            if let Some(code) = req.code.as_deref() {
                if pbx_core::mfa::verify_current(secret, code) {
                    return Ok((StatusCode::OK, token_response(full_token_for(&u, &perms)))
                        .into_response());
                }
                return Err(unauthorized());
            }
            let mfa_token = pbx_core::auth::encode_mfa_token(
                &u.user_uuid.to_string(),
                &u.domain_id.to_string(),
                &u.username,
            );
            return Ok((
                StatusCode::ACCEPTED,
                Json(serde_json::json!({
                    "mfa_required": true,
                    "mfa_token": mfa_token,
                    "expires_in": 300,
                })),
            )
                .into_response());
        }
        return Ok((StatusCode::OK, token_response(full_token_for(&u, &perms))).into_response());
    }
    // Scoped miss for a username that lives in another tenant → 401 (no
    // stub leak across tenants). Unscoped misses fall through to stub.
    if scope.is_some() && user_exists_global(db, &bare).await {
        return Err(unauthorized());
    }
    // Stub fallback (debug only): historical empty-DB login.
    #[cfg(debug_assertions)]
    {
        let _ = domain;
        let token = pbx_core::auth::encode_test_token();
        return Ok((StatusCode::OK, token_response(token)).into_response());
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = domain;
        return Err(unauthorized());
    }
}

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

/// Reload the live user row for a session token: deleted or disabled users
/// fail closed (None). Stale cross-tenant tokens (UUID scope mismatch) also
/// fail. Callers fall back to the legacy claim echo only for opaque test
/// scopes, which have no DB row by design.
async fn load_live_user(
    claims: &pbx_core::auth::Claims,
) -> Option<pbx_db::entities::users::Model> {
    use sea_orm::EntityTrait;
    let uid = claims.sub.parse::<uuid::Uuid>().ok()?;
    let db = crate::routing_db::conn().await;
    let u = pbx_db::entities::users::Entity::find_by_id(uid)
        .one(db)
        .await
        .unwrap_or(None)?;
    if !u.enabled {
        return None;
    }
    if let Ok(scope) = claims.domain_id.parse::<uuid::Uuid>() {
        if scope != u.domain_id {
            return None;
        }
    }
    Some(u)
}

async fn me_handler(headers: HeaderMap) -> Result<Json<UserRead>, StatusCode> {
    let token = bearer_or_cookie_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = pbx_core::auth::decode_test_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    // Pre-auth MFA challenge is not a session.
    if pbx_core::auth::is_mfa_pending(&claims) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if let Some(u) = load_live_user(&claims).await {
        return Ok(Json(UserRead {
            user_uuid: u.user_uuid.to_string(),
            username: u.username,
            email: u.email.clone().or(u.user_email.clone()),
            domain_id: u.domain_id.to_string(),
            enabled: u.enabled,
        }));
    }
    // No row: real-UUID scopes are deleted/disabled users → 401. Legacy
    // opaque test tokens (no DB row) keep echoing claims.
    if claims.domain_id.parse::<uuid::Uuid>().is_ok() {
        return Err(StatusCode::UNAUTHORIZED);
    }
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
    if pbx_core::auth::is_mfa_pending(&claims) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // Fresh permissions from the DB (never trust stale claim perms), same
    // fail-closed rule as `me_handler`.
    if let Some(u) = load_live_user(&claims).await {
        let db = crate::routing_db::conn().await;
        let perms = user_permissions(db, u.user_uuid).await;
        let fresh = pbx_core::auth::encode_claims(
            &u.user_uuid.to_string(),
            &u.domain_id.to_string(),
            &u.username,
            &perms,
        );
        return Ok(token_response(fresh));
    }
    if claims.domain_id.parse::<uuid::Uuid>().is_ok() {
        return Err(StatusCode::UNAUTHORIZED);
    }
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

// ---- TOTP enrolment ----
//
// Parity with PHP totp.php: the secret is stored base32 on the user row and
// the client enrols via an `otpauth://totp/...` URL (QR). Self-enrolment is
// always allowed (authenticated); enrolling another user requires
// `user_edit` (same self-or-perm shape as `PUT /users/{id}/settings`).
#[derive(Debug, Deserialize)]
struct EnrollRequest {
    #[serde(default)]
    user_id: Option<uuid::Uuid>,
}

async fn mfa_enroll_handler(
    headers: HeaderMap,
    payload: Result<Json<EnrollRequest>, JsonRejection>,
) -> impl IntoResponse {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    let token = match bearer_or_cookie_token(&headers) {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let claims = match pbx_core::auth::decode_test_token(&token) {
        Ok(c) => c,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if pbx_core::auth::is_mfa_pending(&claims) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Json(body) = match payload {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "invalid request"})),
            )
                .into_response()
        }
    };
    // Resolve target: explicit user_id, else the caller themself.
    let target_id: uuid::Uuid = match body.user_id {
        Some(id) => id,
        None => match claims.sub.parse::<uuid::Uuid>() {
            Ok(id) => id,
            Err(_) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({"detail": "user_id required for this token"})),
                )
                    .into_response()
            }
        },
    };
    // Managing another user requires `user_edit` (admin/superadmin/* bypass).
    if target_id.to_string() != claims.sub {
        if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_edit") {
            return (s, j).into_response();
        }
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let user = pbx_db::entities::users::Entity::find_by_id(target_id)
        .filter(pbx_db::entities::users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(u) = user else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let secret = pbx_core::mfa::generate_secret();
    let issuer = pbx_db::entities::domains::Entity::find_by_id(domain)
        .one(db)
        .await
        .unwrap_or(None)
        .map(|d| d.domain_name)
        .unwrap_or_else(|| claims.domain_id.clone());
    let mut am: pbx_db::entities::users::ActiveModel = u.into();
    am.user_totp_secret = Set(Some(secret.clone()));
    am.updated_at = Set(crate::routing_db::now());
    if am.update(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "enrol failed"})),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "secret": secret,
            "otpauth_url": pbx_core::mfa::otpauth_url(&target_id.to_string(), &issuer, &secret),
        })),
    )
        .into_response()
}

// ---- TOTP second-factor verification ----
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    mfa_token: String,
    code: String,
}

async fn mfa_verify_handler(
    payload: Result<Json<VerifyRequest>, JsonRejection>,
) -> impl IntoResponse {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let Json(body) = match payload {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "invalid request"})),
            )
                .into_response()
        }
    };
    let claims = match pbx_core::auth::decode_test_token(&body.mfa_token) {
        Ok(c) => c,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    if !pbx_core::auth::is_mfa_pending(&claims) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let (Ok(user_id), Ok(domain_id)) = (
        claims.sub.parse::<uuid::Uuid>(),
        claims.domain_id.parse::<uuid::Uuid>(),
    ) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let db = crate::routing_db::conn().await;
    let user = pbx_db::entities::users::Entity::find_by_id(user_id)
        .filter(pbx_db::entities::users::Column::DomainId.eq(domain_id))
        .one(db)
        .await
        .unwrap_or(None);
    let secret = user
        .as_ref()
        .and_then(|u| u.user_totp_secret.clone())
        .unwrap_or_default();
    let user = match user {
        Some(u) if u.enabled && pbx_core::mfa::verify_current(&secret, &body.code) => u,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let perms = user_permissions(db, user.user_uuid).await;
    let token = full_token_for(&user, &perms);
    let mut headers = HeaderMap::new();
    set_auth_cookie(&mut headers, &token);
    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({
            "access_token": token,
            "token_type": "Bearer",
            "expires_in": 3600,
        })),
    )
        .into_response()
}

// ---- password reset ----
//
// PHP ground truth `fusionpbx/resources/login.php`: request by email, link
// carries an encrypted `username|domain|password|timestamp` payload, 60-minute
// expiry, single-use via password-hash comparison, mailed through the
// `password_reset` email template. Rust port: same shape with a signed JWT
// (`sub|domain_id|marker|exp`, marker = SHA1 of the live password hash)
// enqueued through `email_queue` (waiting row, minijinja template render).
// Both endpoints are public; request always answers 200 so addresses cannot
// be enumerated.
#[derive(Debug, Deserialize)]
struct ResetRequest {
    email: String,
    #[serde(default)]
    domain: Option<String>,
}

async fn reset_request_handler(
    payload: Result<Json<ResetRequest>, JsonRejection>,
) -> impl IntoResponse {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    let generic = || {
        (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ok"})),
        )
            .into_response()
    };
    let Json(body) = match payload {
        Ok(b) => b,
        Err(_) => return generic(),
    };
    let email = body.email.trim().to_owned();
    if email.is_empty() || !email.contains('@') {
        return generic();
    }
    let db = crate::routing_db::conn().await;
    // Optional domain scope (tenant.rs pattern: UUID or domain_name).
    let scope: Option<uuid::Uuid> = match body.domain.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(d) => {
            if let Ok(id) = d.parse::<uuid::Uuid>() {
                Some(id)
            } else {
                match pbx_db::entities::domains::Entity::find()
                    .filter(pbx_db::entities::domains::Column::DomainName.eq(d.to_owned()))
                    .one(db)
                    .await
                    .unwrap_or(None)
                {
                    Some(r) => Some(r.domain_id),
                    None => return generic(),
                }
            }
        }
    };
    let mut q = pbx_db::entities::users::Entity::find()
        .filter(pbx_db::entities::users::Column::Email.eq(email.clone()))
        .filter(pbx_db::entities::users::Column::Enabled.eq(true));
    if let Some(d) = scope {
        q = q.filter(pbx_db::entities::users::Column::DomainId.eq(d));
    }
    let user = q.one(db).await.unwrap_or(None);
    let Some(u) = user else {
        return generic();
    };
    let marker = pbx_core::mfa::reset_marker(&u.password_hash);
    let token = pbx_core::mfa::encode_reset_token(
        &u.user_uuid.to_string(),
        &u.domain_id.to_string(),
        &marker,
        3600,
    );
    let domain_name = pbx_db::entities::domains::Entity::find_by_id(u.domain_id)
        .one(db)
        .await
        .unwrap_or(None)
        .map(|d| d.domain_name)
        .unwrap_or_else(|| "localhost".to_owned());
    let reset_link = format!("https://{domain_name}/api/v1/auth/password-reset/confirm?token={token}");
    let (subject, text_body) = crate::email_queue::render_password_reset_email(&reset_link, &domain_name);
    let body_with_link = format!("{text_body}\nReset link: {reset_link}\ntoken={token}\n");
    let am = pbx_db::entities::email_queue::ActiveModel {
        email_queue_uuid: Set(uuid::Uuid::now_v7()),
        domain_id: Set(u.domain_id),
        email_from: Set("pbx@localhost".to_owned()),
        email_to: Set(email),
        email_subject: Set(Some(subject)),
        email_body: Set(Some(body_with_link)),
        email_status: Set("waiting".to_owned()),
        created_at: Set(crate::routing_db::now()),
        ..Default::default()
    };
    let _ = am.insert(db).await;
    generic()
}

#[derive(Debug, Deserialize)]
struct ResetConfirm {
    token: String,
    new_password: String,
}

async fn reset_confirm_handler(
    payload: Result<Json<ResetConfirm>, JsonRejection>,
) -> impl IntoResponse {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    let invalid = || {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"detail": "invalid or expired token"})),
        )
            .into_response()
    };
    let Json(body) = match payload {
        Ok(b) => b,
        Err(_) => return invalid(),
    };
    if body.new_password.len() < 8 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "new_password must be at least 8 characters"})),
        )
            .into_response();
    }
    let rc = match pbx_core::mfa::decode_reset_token(&body.token) {
        Ok(c) => c,
        Err(_) => return invalid(),
    };
    let (Ok(user_id), Ok(domain_id)) = (
        rc.sub.parse::<uuid::Uuid>(),
        rc.domain_id.parse::<uuid::Uuid>(),
    ) else {
        return invalid();
    };
    let db = crate::routing_db::conn().await;
    let user = pbx_db::entities::users::Entity::find_by_id(user_id)
        .filter(pbx_db::entities::users::Column::DomainId.eq(domain_id))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(u) = user else {
        return invalid();
    };
    // Single-use: the marker binds the token to the password hash at request
    // time; any intervening reset rotates the hash and fails closed here.
    if pbx_core::mfa::reset_marker(&u.password_hash) != rc.marker {
        return invalid();
    }
    let mut am: pbx_db::entities::users::ActiveModel = u.into();
    am.password_hash = Set(pbx_core::auth::hash_password(&body.new_password));
    am.updated_at = Set(crate::routing_db::now());
    if am.update(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "reset failed"})),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "password reset"})),
    )
        .into_response()
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
        .route("/auth/mfa/enroll", post(mfa_enroll_handler))
        .route("/api/v1/auth/mfa/enroll", post(mfa_enroll_handler))
        .route("/auth/mfa/verify", post(mfa_verify_handler))
        .route("/api/v1/auth/mfa/verify", post(mfa_verify_handler))
        .route("/auth/password-reset/request", post(reset_request_handler))
        .route(
            "/api/v1/auth/password-reset/request",
            post(reset_request_handler),
        )
        .route("/auth/password-reset/confirm", post(reset_confirm_handler))
        .route(
            "/api/v1/auth/password-reset/confirm",
            post(reset_confirm_handler),
        )
        .route("/docs", get(docs_redirect))
        .route("/redoc", get(docs_redirect))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn req(
        method: &str,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut b = match method {
            "POST" => Request::post(path),
            "GET" => Request::get(path),
            _ => Request::post(path),
        };
        if let Some(t) = token {
            b = b.header("authorization", format!("Bearer {t}"));
        }
        let res = crate::test_app()
            .oneshot(
                b.header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    async fn admin_token() -> String {
        let (s, v) = req(
            "POST",
            "/api/v1/auth/login",
            None,
            serde_json::json!({"username":"admin","password":"x"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "stub admin login: {v}");
        v["access_token"].as_str().unwrap().to_owned()
    }

    fn uniq(prefix: &str) -> String {
        format!("{prefix}_{}", uuid::Uuid::now_v7().simple())
    }

    async fn create_user(token: &str, username: &str, password: &str, email: Option<&str>) -> String {
        let (s, v) = req(
            "POST",
            "/api/v1/users",
            Some(token),
            serde_json::json!({"username": username, "password": password, "email": email}),
        )
        .await;
        assert_eq!(s, StatusCode::CREATED, "create user {username}: {v}");
        v["user_uuid"].as_str().unwrap().to_owned()
    }

    #[tokio::test]
    async fn mfa_enroll_login_verify_roundtrip() {
        let admin = admin_token().await;
        let username = uniq("mfa");
        let uid = create_user(&admin, &username, "Secret123!", None).await;

        // enroll TOTP for the new user (admin holds user_edit via admin bypass)
        let (s, v) = req(
            "POST",
            "/api/v1/auth/mfa/enroll",
            Some(&admin),
            serde_json::json!({"user_id": uid}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "enroll: {v}");
        let secret = v["secret"].as_str().expect("secret").to_owned();
        assert!(v["otpauth_url"].as_str().unwrap().starts_with("otpauth://totp/"));

        // password-only login now challenges with 202 + mfa_token
        let (s, v) = req(
            "POST",
            "/api/v1/auth/login",
            None,
            serde_json::json!({"username": username, "password": "Secret123!"}),
        )
        .await;
        assert_eq!(s, StatusCode::ACCEPTED, "mfa challenge: {v}");
        assert_eq!(v["mfa_required"], serde_json::json!(true));
        let mfa_token = v["mfa_token"].as_str().unwrap().to_owned();

        // pending token must not access /me
        let (s, _) = req("GET", "/api/v1/auth/me", Some(&mfa_token), serde_json::json!({})).await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);

        // wrong code rejected
        let (s, _) = req(
            "POST",
            "/api/v1/auth/mfa/verify",
            None,
            serde_json::json!({"mfa_token": mfa_token, "code": "000000"}),
        )
        .await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);

        // correct code yields a full token
        let code = pbx_core::mfa::current_code(&secret).unwrap();
        let (s, v) = req(
            "POST",
            "/api/v1/auth/mfa/verify",
            None,
            serde_json::json!({"mfa_token": mfa_token, "code": code}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "verify: {v}");
        let full = v["access_token"].as_str().unwrap().to_owned();
        let (s, _) = req("GET", "/api/v1/auth/me", Some(&full), serde_json::json!({})).await;
        assert_eq!(s, StatusCode::OK);
    }

    #[tokio::test]
    async fn mfa_login_single_step_with_code() {
        let admin = admin_token().await;
        let username = uniq("mfa1");
        let uid = create_user(&admin, &username, "Secret123!", None).await;
        let (s, v) = req(
            "POST",
            "/api/v1/auth/mfa/enroll",
            Some(&admin),
            serde_json::json!({"user_id": uid}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "enroll: {v}");
        let secret = v["secret"].as_str().unwrap().to_owned();

        let code = pbx_core::mfa::current_code(&secret).unwrap();
        let (s, v) = req(
            "POST",
            "/api/v1/auth/login",
            None,
            serde_json::json!({"username": username, "password": "Secret123!", "code": code}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "single-step login: {v}");
        assert!(v["access_token"].as_str().is_some());
    }

    #[tokio::test]
    async fn password_reset_request_confirm_single_use() {
        let admin = admin_token().await;
        let username = uniq("rst");
        let email = format!("{username}@example.com");
        create_user(&admin, &username, "OldPass123!", Some(&email)).await;

        // unknown email still returns generic ok (no enumeration oracle)
        let (s, v) = req(
            "POST",
            "/api/v1/auth/password-reset/request",
            None,
            serde_json::json!({"email": "nobody@example.com"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "unknown email: {v}");

        let (s, _) = req(
            "POST",
            "/api/v1/auth/password-reset/request",
            None,
            serde_json::json!({"email": email}),
        )
        .await;
        assert_eq!(s, StatusCode::OK);

        // token arrives via the email_queue row body
        let (s, v) = req("GET", "/api/v1/email_queue", Some(&admin), serde_json::json!({})).await;
        assert_eq!(s, StatusCode::OK, "list queue: {v}");
        let token = v["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["email_to"] == email)
            .filter_map(|r| r["email_body"].as_str())
            .find_map(extract_reset_token)
            .expect("reset email with token");
        assert!(v["data"].as_array().unwrap().iter().any(|r| r["email_to"] == email
            && r["email_subject"].as_str().unwrap().contains("Password Reset")));

        // confirm sets the new password
        let (s, v) = req(
            "POST",
            "/api/v1/auth/password-reset/confirm",
            None,
            serde_json::json!({"token": token, "new_password": "NewPass123!"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "confirm: {v}");

        // new password logs in, old does not
        let (s, _) = req(
            "POST",
            "/api/v1/auth/login",
            None,
            serde_json::json!({"username": username, "password": "NewPass123!"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = req(
            "POST",
            "/api/v1/auth/login",
            None,
            serde_json::json!({"username": username, "password": "OldPass123!"}),
        )
        .await;
        assert_eq!(s, StatusCode::UNAUTHORIZED);

        // single-use: replay rejected
        let (s, _) = req(
            "POST",
            "/api/v1/auth/password-reset/confirm",
            None,
            serde_json::json!({"token": token, "new_password": "NewPass123!"}),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST);

        // garbage token rejected
        let (s, _) = req(
            "POST",
            "/api/v1/auth/password-reset/confirm",
            None,
            serde_json::json!({"token": "garbage.token.here", "new_password": "NewPass123!"}),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn password_reset_expired_token_rejected() {
        // craft an already-expired token bound to a dummy marker
        let token = pbx_core::mfa::encode_reset_token(
            "00000000-0000-0000-0000-000000000000",
            "default",
            "deadbeef",
            -10,
        );
        let (s, _) = req(
            "POST",
            "/api/v1/auth/password-reset/confirm",
            None,
            serde_json::json!({"token": token, "new_password": "NewPass123!"}),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
    }

    fn extract_reset_token(body: &str) -> Option<String> {
        let key = "token=";
        let i = body.find(key)?;
        let rest = &body[i + key.len()..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '<')
            .unwrap_or(rest.len());
        let tok = rest[..end].trim_end_matches(['.', ')']);
        if tok.contains('.') {
            Some(tok.to_owned())
        } else {
            None
        }
    }
}
