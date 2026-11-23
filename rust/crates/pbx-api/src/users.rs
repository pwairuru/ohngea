//! Users router — mirrors `app/auth/models.py` User + users parity.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{users}`.
//! Tenant-guarded (domain_id = routing_db::domain); list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::users;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct UserCreate {
    username: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct UserPatch {
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
}

// ---- handlers ----
fn sstr(p: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = p.get(*k) {
            if let Some(s) = v.as_str() {
                let t = s.trim();
                if !t.is_empty() {
                    return Some(t.to_owned());
                }
            }
        }
    }
    None
}

fn ser_user(r: &users::Model) -> serde_json::Value {
    serde_json::json!({
        "user_uuid": r.user_uuid,
        "username": r.username,
        "email": r.email,
        "user_email": r.user_email.clone().or_else(|| r.email.clone()),
        "domain_id": r.domain_id,
        "enabled": r.enabled,
        "user_enabled": r.user_enabled.unwrap_or(r.enabled),
        "user_status": r.user_status,
        "user_time_zone": r.user_time_zone,
        "user_language": r.user_language,
        "user_type": r.user_type,
        "api_key": r.api_key,
        "contact_uuid": r.contact_uuid,
        "contact_name_given": r.contact_name_given,
        "contact_name_family": r.contact_name_family,
        "contact_organization": r.contact_organization,
        "user_totp_secret": r.user_totp_secret,
        "password_confirm": null,
        "username_old": r.username,
    })
}

async fn create_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_add") {
        return (s, j).into_response();
    }
    let username = sstr(&payload, &["username"]).unwrap_or_default();
    if username.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "username required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let tenant = crate::routing_db::tenant_domain(db, &claims).await;
    // PHP `user_domain` parity: holders may assign another tenant
    // (valid UUID only — users are never global).
    let domain: uuid::Uuid = match payload
        .get("domain_uuid")
        .or_else(|| payload.get("domain_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
    {
        Some(v) if !v.is_empty() && crate::tenant::has_permission(&claims, "user_domain") => {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => id,
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_uuid must be a UUID"})),
                    )
                        .into_response();
                }
            }
        }
        _ => tenant,
    };
    // duplicate username within tenant → 409 (parity with Python unique index).
    let exists = users::Entity::find()
        .filter(users::Column::DomainId.eq(domain))
        .filter(users::Column::Username.eq(username.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": "username exists"})),
        )
            .into_response();
    }
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    // Real bcrypt hash (parity with app/auth/service.py hash_password).
    // Blank password hashes a random id so verify never passes on empty.
    // password_confirm validated to match when both present (PHP parity).
    let pw = sstr(&payload, &["password"]).unwrap_or_default();
    if let Some(pc) = sstr(&payload, &["password_confirm"]) {
        if !pw.is_empty() && pw != pc {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "passwords do not match"})),
            )
                .into_response();
        }
    }
    let pw_hash = if pw.is_empty() {
        pbx_core::auth::hash_password(&id.to_string())
    } else {
        pbx_core::auth::hash_password(&pw)
    };
    let email_val = sstr(&payload, &["user_email", "email"]);
    let enabled_val = payload.get("user_enabled").or_else(|| payload.get("enabled")).and_then(|v| v.as_bool()).unwrap_or(true);
    let am = users::ActiveModel {
        user_uuid: Set(id),
        domain_id: Set(domain),
        username: Set(username.clone()),
        email: Set(email_val.clone()),
        user_email: Set(email_val.clone()),
        password_hash: Set(pw_hash),
        enabled: Set(enabled_val),
        user_enabled: Set(Some(enabled_val)),
        user_status: Set(sstr(&payload, &["user_status"])),
        user_time_zone: Set(sstr(&payload, &["user_time_zone"])),
        user_language: Set(sstr(&payload, &["user_language"])),
        user_type: Set(sstr(&payload, &["user_type"])),
        api_key: Set(sstr(&payload, &["api_key"])),
        contact_uuid: Set(None),
        contact_name_given: Set(sstr(&payload, &["contact_name_given"])),
        contact_name_family: Set(sstr(&payload, &["contact_name_family"])),
        contact_organization: Set(sstr(&payload, &["contact_organization"])),
        user_totp_secret: Set(sstr(&payload, &["user_totp_secret"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    // Linked contact parity (PHP creates v_contacts + v_contact_emails on user_add).
    // Best-effort: create contact row so contact_* fields are durable cross-table.
    let contact_given = sstr(&payload, &["contact_name_given"]);
    let contact_family = sstr(&payload, &["contact_name_family"]);
    let contact_org = sstr(&payload, &["contact_organization"]);
    let contact_email = email_val.clone();
    if contact_given.is_some() || contact_family.is_some() || contact_org.is_some() {
        use pbx_db::entities::contacts;
        let cid = uuid::Uuid::now_v7();
        let cname = format!("{} {}", contact_given.clone().unwrap_or_default(), contact_family.clone().unwrap_or_default()).trim().to_owned();
        let _ = contacts::ActiveModel {
            contact_uuid: Set(cid),
            domain_id: Set(domain),
            name: Set(if cname.is_empty() { username.clone() } else { cname }),
            number: Set(String::new()),
            email: Set(contact_email.clone()),
            organization: Set(contact_org.clone()),
            contact_organization: Set(contact_org.clone()),
            contact_type: Set(Some("user".to_owned())),
            contact_parent_uuid: Set(None),
            contact_name_prefix: Set(None),
            contact_name_given: Set(contact_given.clone()),
            contact_name_middle: Set(None),
            contact_name_family: Set(contact_family.clone()),
            contact_name_suffix: Set(None),
            contact_nickname: Set(Some(username.clone())),
            contact_title: Set(None),
            contact_role: Set(None),
            contact_category: Set(None),
            contact_url: Set(None),
            contact_time_zone: Set(None),
            contact_note: Set(None),
            created_at: Set(now),
        }
        .insert(db)
        .await;
    }
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_user(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "user_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/users", page, per_page, 0),
                Json(crate::paged_body(Vec::<serde_json::Value>::new(), 0, page, per_page)),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let mut sel = users::Entity::find();
    if !show_all {
        sel = sel.filter(users::Column::DomainId.eq(domain));
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_user).collect();
    (
        crate::paged_headers("/api/v1/users", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (StatusCode::OK, Json(ser_user(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: users::ActiveModel = r.into();
    if let Some(u) = sstr(&payload, &["username"]) {
        if u.trim().is_empty() {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "username required"})),
            )
                .into_response();
        }
        am.username = Set(u);
    }
    if let Some(e) = sstr(&payload, &["user_email", "email"]) {
        am.email = Set(Some(e.clone()));
        am.user_email = Set(Some(e));
    }
    if let Some(en) = payload.get("user_enabled").or_else(|| payload.get("enabled")).and_then(|v| v.as_bool()) {
        am.enabled = Set(en);
        am.user_enabled = Set(Some(en));
    }
    for (keys, slot) in [
        (&["user_status"][..], &mut am.user_status),
        (&["user_time_zone"][..], &mut am.user_time_zone),
        (&["user_language"][..], &mut am.user_language),
        (&["user_type"][..], &mut am.user_type),
        (&["api_key"][..], &mut am.api_key),
        (&["contact_name_given"][..], &mut am.contact_name_given),
        (&["contact_name_family"][..], &mut am.contact_name_family),
        (&["contact_organization"][..], &mut am.contact_organization),
        (&["user_totp_secret"][..], &mut am.user_totp_secret),
    ] {
        if let Some(v) = sstr(&payload, keys) {
            *slot = Set(Some(v));
        }
    }
    // password change with confirm check
    if let Some(pw) = sstr(&payload, &["password"]) {
        if let Some(pc) = sstr(&payload, &["password_confirm"]) {
            if pw != pc {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({"detail": "passwords do not match"})),
                )
                    .into_response();
            }
        }
        if !pw.is_empty() {
            am.password_hash = Set(pbx_core::auth::hash_password(&pw));
        }
    }
    am.updated_at = Set(crate::routing_db::now());
    // PHP `user_domain` parity: holders may move the user to another
    // tenant (valid UUID only). Group memberships follow the user
    // (PHP user_edit adjusts user_groups rows on domain change).
    let mut move_to: Option<uuid::Uuid> = None;
    if let Some(v) = payload
        .get("domain_uuid")
        .or_else(|| payload.get("domain_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if crate::tenant::has_permission(&claims, "user_domain") {
            match v.parse::<uuid::Uuid>() {
                Ok(tid) if tid != domain => {
                    am.domain_id = Set(tid);
                    move_to = Some(tid);
                }
                Ok(_) => {}
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_uuid must be a UUID"})),
                    )
                        .into_response();
                }
            }
        }
    }
    match am.update(db).await {
        Ok(r) => {
            if let Some(tid) = move_to {
                use sea_orm::EntityTrait as _;
                let _ = pbx_db::entities::user_groups::Entity::update_many()
                    .col_expr(
                        pbx_db::entities::user_groups::Column::DomainId,
                        sea_orm::sea_query::Expr::value(tid),
                    )
                    .filter(pbx_db::entities::user_groups::Column::UserUuid.eq(r.user_uuid))
                    .exec(db)
                    .await;
            }
            (StatusCode::OK, Json(ser_user(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: users::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn users_router() -> Router {
    Router::new()
        .route("/api/v1/users", post(create_user).get(list_users))
        .route("/api/v1/users/", post(create_user).get(list_users))
        .route("/api/v1/users/bulk", post(bulk_users))
        .route("/api/v1/users/import", post(import_users))
        .route(
            "/api/v1/users/{id}",
            axum::routing::get(get_user)
                .patch(patch_user)
                .delete(delete_user),
        )
        .route(
            "/api/v1/users/{id}/groups",
            axum::routing::get(list_user_groups),
        )
        .route(
            "/api/v1/users/{id}/settings",
            axum::routing::get(get_user_settings).put(put_user_settings),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

// ---- bulk (parity with app/users/router.py::bulk_users: copy/toggle/delete) ----
#[derive(Debug, serde::Deserialize)]
struct BulkRequest {
    #[serde(default)]
    action: String,
    #[serde(default)]
    ids: Vec<serde_json::Value>,
}

fn bulk_id(v: &serde_json::Value) -> Option<uuid::Uuid> {
    match v {
        serde_json::Value::String(s) => s.parse().ok(),
        serde_json::Value::Object(m) => m
            .get("uuid")
            .or_else(|| m.get("user_uuid"))
            .and_then(|u| u.as_str())
            .and_then(|s| s.parse().ok()),
        _ => None,
    }
}

async fn bulk_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<BulkRequest>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut results = Vec::new();
    for item in &payload.ids {
        let Some(uid) = bulk_id(item) else {
            results.push(serde_json::json!({"error": "invalid uuid"}));
            continue;
        };
        let row = users::Entity::find_by_id(uid)
            .filter(users::Column::DomainId.eq(domain))
            .one(db)
            .await
            .unwrap_or(None);
        let Some(r) = row else {
            results.push(serde_json::json!({"user_uuid": uid, "error": "not found"}));
            continue;
        };
        match payload.action.as_str() {
            "toggle" => {
                let enabled = !r.enabled;
                let mut am: users::ActiveModel = r.into();
                am.enabled = Set(enabled);
                am.updated_at = Set(crate::routing_db::now());
                let _ = am.update(db).await;
                results.push(
                    serde_json::json!({"user_uuid": uid, "action": "toggled", "enabled": enabled}),
                );
            }
            "copy" => {
                let c = r;
                let nid = uuid::Uuid::now_v7();
                let now = crate::routing_db::now();
                let am = users::ActiveModel {
                    user_uuid: Set(nid),
                    domain_id: Set(c.domain_id),
                    username: Set(format!("{} (copy)", c.username)),
                    email: Set(c.email.clone()),
                    user_email: Set(c.user_email.clone()),
                    password_hash: Set(c.password_hash.clone()),
                    enabled: Set(c.enabled),
                    user_enabled: Set(c.user_enabled),
                    user_status: Set(c.user_status.clone()),
                    user_time_zone: Set(c.user_time_zone.clone()),
                    user_language: Set(c.user_language.clone()),
                    user_type: Set(c.user_type.clone()),
                    api_key: Set(None),
                    contact_uuid: Set(c.contact_uuid),
                    contact_name_given: Set(c.contact_name_given.clone()),
                    contact_name_family: Set(c.contact_name_family.clone()),
                    contact_organization: Set(c.contact_organization.clone()),
                    user_totp_secret: Set(None),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                match am.insert(db).await {
                    Ok(_) => {
                        results.push(serde_json::json!({"user_uuid": nid, "action": "copied"}))
                    }
                    Err(e) => {
                        results.push(serde_json::json!({"user_uuid": uid, "error": format!("{e}")}))
                    }
                }
            }
            "delete" => {
                let am: users::ActiveModel = r.into();
                match am.delete(db).await {
                    Ok(_) => {
                        results.push(serde_json::json!({"user_uuid": uid, "action": "deleted"}))
                    }
                    Err(e) => {
                        results.push(serde_json::json!({"user_uuid": uid, "error": format!("{e}")}))
                    }
                }
            }
            _ => results.push(serde_json::json!({"user_uuid": uid, "error": "Invalid action"})),
        }
    }
    let total = results.len();
    Json(serde_json::json!({"data": results, "total": total})).into_response()
}

// ---- import (parity shape: POST /users/import {"csv"} or {"users": [...]}) ----
async fn import_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut imported = 0u64;
    let mut errors = Vec::new();
    let mut created = Vec::new();
    if let Some(arr) = payload.get("users").and_then(|u| u.as_array()) {
        for (i, row) in arr.iter().enumerate() {
            let username = row
                .get("username")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .trim()
                .to_owned();
            if username.is_empty() {
                errors.push(serde_json::json!({"row": i + 1, "error": "username required"}));
                continue;
            }
            let email = row
                .get("email")
                .and_then(|e| e.as_str())
                .map(|s| s.to_owned());
            let pw = row
                .get("password")
                .and_then(|p| p.as_str())
                .unwrap_or("Stub123!xxxx");
            let id = uuid::Uuid::now_v7();
            let now = crate::routing_db::now();
            let am = users::ActiveModel {
                user_uuid: Set(id),
                domain_id: Set(domain),
                username: Set(username.clone()),
                email: Set(email.clone()),
                user_email: Set(email.clone()),
                password_hash: Set(pbx_core::auth::hash_password(pw)),
                enabled: Set(true),
                user_enabled: Set(Some(true)),
                user_status: Set(None),
                user_time_zone: Set(None),
                user_language: Set(None),
                user_type: Set(Some("default".to_owned())),
                api_key: Set(None),
                contact_uuid: Set(None),
                contact_name_given: Set(None),
                contact_name_family: Set(None),
                contact_organization: Set(None),
                user_totp_secret: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => {
                    imported += 1;
                    created.push(serde_json::json!({"user_uuid": id, "username": username}));
                }
                Err(e) => errors.push(serde_json::json!({"row": i + 1, "error": format!("{e}")})),
            }
        }
    } else if let Some(csv) = payload.get("csv").and_then(|c| c.as_str()) {
        for (i, line) in csv.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue; // header
            }
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            let username = parts.first().unwrap_or(&"").to_string();
            if username.is_empty() {
                errors.push(serde_json::json!({"row": i, "error": "username required"}));
                continue;
            }
            let email = parts
                .get(1)
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            let pw = parts
                .get(2)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Stub123!xxxx".to_string());
            let id = uuid::Uuid::now_v7();
            let now = crate::routing_db::now();
            let am = users::ActiveModel {
                user_uuid: Set(id),
                domain_id: Set(domain),
                username: Set(username.clone()),
                email: Set(email.clone()),
                user_email: Set(email.clone()),
                password_hash: Set(pbx_core::auth::hash_password(&pw)),
                enabled: Set(true),
                user_enabled: Set(Some(true)),
                user_status: Set(None),
                user_time_zone: Set(None),
                user_language: Set(None),
                user_type: Set(Some("default".to_owned())),
                api_key: Set(None),
                contact_uuid: Set(None),
                contact_name_given: Set(None),
                contact_name_family: Set(None),
                contact_organization: Set(None),
                user_totp_secret: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => {
                    imported += 1;
                    created.push(serde_json::json!({"user_uuid": id, "username": username}));
                }
                Err(e) => errors.push(serde_json::json!({"row": i, "error": format!("{e}")})),
            }
        }
    } else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "CSV content required"})),
        )
            .into_response();
    }
    Json(serde_json::json!({"imported": imported, "errors": errors, "data": created, "total": imported})).into_response()
}

// ---- user_settings (parity with app/users/router.py::{get,put}_user_settings) ----
fn user_setting_json(r: &pbx_db::entities::user_settings::Model) -> serde_json::Value {
    serde_json::json!({
        "user_setting_uuid": r.user_setting_uuid,
        "user_uuid": r.user_uuid,
        "domain_id": r.domain_id,
        "category": r.user_setting_category,
        "user_setting_category": r.user_setting_category,
        "subcategory": r.user_setting_subcategory,
        "user_setting_subcategory": r.user_setting_subcategory,
        "name": r.user_setting_name,
        "user_setting_name": r.user_setting_name,
        "value": r.user_setting_value,
        "user_setting_value": r.user_setting_value,
        "order": r.user_setting_order,
        "user_setting_order": r.user_setting_order,
        "enabled": r.user_setting_enabled,
        "user_setting_enabled": r.user_setting_enabled,
        "description": r.user_setting_description,
        "user_setting_description": r.user_setting_description,
    })
}

/// Self-or-perm gate (Python `put_user_settings` parity): the user themself
/// always passes; anyone else needs `user_setting_view`/`user_edit`/`admin`.
fn self_or(
    claims: &pbx_core::auth::Claims,
    target: &uuid::Uuid,
    perm: &str,
) -> Option<(StatusCode, Json<serde_json::Value>)> {
    if claims.sub == target.to_string() {
        return None;
    }
    crate::tenant::deny_if_missing(claims, perm)
}

async fn get_user_settings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    // Python GET is auth-only; we tighten to self-or-view and note parity.
    if let Some((s, j)) = self_or(&claims, &id, "user_setting_view") {
        // Fall back to user_view so profile readers (not settings admins)
        // can still see a user's settings row.
        if crate::tenant::deny_if_missing(&claims, "user_view").is_some() {
            return (s, j).into_response();
        }
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    if user.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "User not found"})),
        )
            .into_response();
    }
    let rows = pbx_db::entities::user_settings::Entity::find()
        .filter(pbx_db::entities::user_settings::Column::UserUuid.eq(id))
        .filter(pbx_db::entities::user_settings::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(user_setting_json).collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn put_user_settings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Python parity: self bypasses; others need user_setting_view/user_edit.
    if claims.sub != id.to_string()
        && crate::tenant::deny_if_missing(&claims, "user_setting_view").is_some()
        && crate::tenant::deny_if_missing(&claims, "user_edit").is_some()
    {
        let (s, j) = crate::tenant::forbidden("user_setting_view");
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(u) = user else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "User not found"})),
        )
            .into_response();
    };
    let settings = payload
        .get("settings")
        .or_else(|| payload.get("data"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    // Replace-all (Python parity): delete existing, recreate from payload.
    use sea_orm::EntityTrait as _;
    let _ = pbx_db::entities::user_settings::Entity::delete_many()
        .filter(pbx_db::entities::user_settings::Column::UserUuid.eq(id))
        .exec(db)
        .await;
    for s in &settings {
        let am = pbx_db::entities::user_settings::ActiveModel {
            user_setting_uuid: Set(uuid::Uuid::now_v7()),
            user_uuid: Set(id),
            domain_id: Set(u.domain_id),
            user_setting_category: Set(s
                .get("category")
                .or_else(|| s.get("user_setting_category"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned()),
            user_setting_subcategory: Set(s
                .get("subcategory")
                .or_else(|| s.get("user_setting_subcategory"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
            user_setting_name: Set(s
                .get("name")
                .or_else(|| s.get("user_setting_name"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
            user_setting_value: Set(s
                .get("value")
                .or_else(|| s.get("user_setting_value"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
            user_setting_order: Set(s
                .get("order")
                .or_else(|| s.get("user_setting_order"))
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)),
            user_setting_enabled: Set(s
                .get("enabled")
                .or_else(|| s.get("user_setting_enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true)),
            user_setting_description: Set(s
                .get("description")
                .or_else(|| s.get("user_setting_description"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
        };
        let _ = am.insert(db).await;
    }
    Json(serde_json::json!({"status": "updated"})).into_response()
}

// ---- user → groups (parity with GET /users/{id}/groups) ----
async fn list_user_groups(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_view") {
        return (s, j).into_response();
    }
    use pbx_db::entities::{groups, user_groups};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    // Tenant check: user must belong to caller's domain, else 404 (no cross-tenant leak).
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    if user.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mems = user_groups::Entity::find()
        .filter(user_groups::Column::UserUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    let mut data = Vec::new();
    for m in mems {
        if let Ok(Some(g)) = groups::Entity::find_by_id(m.group_uuid)
            .filter(groups::Column::DomainId.eq(domain))
            .one(db)
            .await
        {
            data.push(serde_json::json!({
                "group_uuid": g.group_uuid,
                "name": g.name,
                "description": g.description,
                "domain_id": g.domain_id,
                "group_level": g.group_level,
            }));
        }
    }
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}
