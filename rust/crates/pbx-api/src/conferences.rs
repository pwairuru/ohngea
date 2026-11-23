//! Conferences router — mirrors `app/conferences/router.py`.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;

// ---- models ----
use pbx_db::entities::{conference_users, conferences};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct ConferenceCreate {
    conference_name: String,
    conference_extension: String,
    #[serde(default = "default_profile")]
    conference_profile: String,
    #[serde(default)]
    conference_enabled: Option<bool>,
    #[serde(default)]
    dialplan_uuid: Option<String>,
    #[serde(default)]
    pin_number: Option<String>,
    #[serde(default)]
    conference_pin_number: Option<String>,
    #[serde(default)]
    email_address: Option<String>,
    #[serde(default)]
    conference_email_address: Option<String>,
    #[serde(default)]
    account_code: Option<String>,
    #[serde(default)]
    conference_account_code: Option<String>,
    #[serde(default)]
    flags: Option<String>,
    #[serde(default)]
    conference_flags: Option<String>,
    #[serde(default)]
    conference_order: Option<i32>,
    #[serde(default)]
    conference_description: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    conference_context: Option<String>,
}

fn default_profile() -> String {
    "default".to_owned()
}

// ---- handlers ----
/// PHP ground truth (`conference_edit.php`): `empty($conference_extension)`
/// is the only server-side rule — no digit/length regex.
fn valid_extension(ext: &str) -> bool {
    !ext.trim().is_empty()
}

fn ser_conf(r: &conferences::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "conference_uuid": r.conference_uuid,
        "conference_name": r.conference_name,
        "conference_extension": r.conference_extension,
        "conference_profile": r.conference_profile,
        "conference_enabled": r.conference_enabled,
        "dialplan_uuid": r.dialplan_uuid,
        "pin_number": r.pin_number,
        "email_address": r.email_address,
        "account_code": r.account_code,
        "flags": r.flags,
        "conference_order": r.conference_order,
        "conference_description": r.conference_description,
        "context": r.context,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn conf_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<conferences::Model> {
    // PHP parity (conference_edit.php): detail loads are tenant-scoped.
    conferences::Entity::find_by_id(id.to_owned())
        .filter(conferences::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn create_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<ConferenceCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_add") {
        return (s, j).into_response();
    }
    if payload.conference_name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "conference_name required"})),
        )
            .into_response();
    }
    if !valid_extension(&payload.conference_extension) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "conference_extension required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    // PHP mints the dialplan row alongside the conference.
    let dp_id = crate::ring_groups::parse_dialplan_id(payload.dialplan_uuid.as_deref())
        .unwrap_or_else(crate::ring_groups::new_dialplan_id);
    let am = conferences::ActiveModel {
        conference_uuid: Set(id.clone()),
        domain_id: Set(Some(domain)),
        conference_name: Set(payload.conference_name.trim().to_owned()),
        conference_extension: Set(payload.conference_extension.trim().to_owned()),
        conference_profile: Set(payload.conference_profile),
        conference_enabled: Set(payload.conference_enabled.unwrap_or(true)),
        dialplan_uuid: Set(Some(dp_id.to_string())),
        pin_number: Set(payload.pin_number.or(payload.conference_pin_number)),
        email_address: Set(payload.email_address.or(payload.conference_email_address)),
        account_code: Set(payload.account_code.or(payload.conference_account_code)),
        flags: Set(payload.flags.or(payload.conference_flags)),
        conference_order: Set(payload.conference_order),
        conference_description: Set(payload.conference_description.or(payload.description)),
        context: Set(payload.context.or(payload.conference_context)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    }
    let row = conf_or_404(db, domain, &id).await;
    match row {
        Some(r) => {
            // PHP `conference_edit.php` dialplan: conference ext@domain@profile.
            let domain_name = crate::ring_groups::resolve_domain_name(
                db,
                domain,
                &crate::routing_db::tenant_display_name(&claims),
            )
            .await;
            let spec = conf_spec(
                &r.conference_name,
                &r.conference_extension,
                &r.conference_uuid,
                &dp_id,
                &domain_name,
                &r.conference_profile,
                r.pin_number.as_deref(),
                r.flags.as_deref(),
                r.context.clone(),
                r.conference_enabled,
                r.conference_description.clone(),
            );
            if let Err(e) =
                crate::ring_groups::sync_app_dialplan(db, Some(domain), dp_id, &spec).await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": e})),
                )
                    .into_response();
            }
            (StatusCode::CREATED, Json(ser_conf(&r))).into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response(),
    }
}

async fn list_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "conference_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/conferences", page, per_page, 0),
                Json(crate::paged_body(
                    Vec::<serde_json::Value>::new(),
                    0,
                    page,
                    per_page,
                )),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let mut sel = conferences::Entity::find();
    if !show_all {
        sel = sel.filter(
            Condition::any()
                .add(conferences::Column::DomainId.eq(domain))
                .add(conferences::Column::DomainId.is_null()),
        );
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_conf).collect();
    (
        crate::paged_headers("/api/v1/conferences", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn health_conf() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "conferences"})).into_response()
}

async fn get_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match conf_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_conf(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference not found"})),
        )
            .into_response(),
    }
}

async fn patch_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = conf_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference not found"})),
        )
            .into_response();
    };
    let mut am: conferences::ActiveModel = r.into();
    if let Some(v) = payload.get("conference_name").and_then(|v| v.as_str()) {
        am.conference_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("conference_extension").and_then(|v| v.as_str()) {
        if !valid_extension(v) {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "conference_extension required"})),
            )
                .into_response();
        }
        am.conference_extension = Set(v.to_owned());
    }
    if let Some(v) = payload.get("conference_profile").and_then(|v| v.as_str()) {
        am.conference_profile = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("conference_enabled")
        .or_else(|| payload.get("enabled"))
        .and_then(|v| v.as_bool())
    {
        am.conference_enabled = Set(v);
    }
    macro_rules! set_str {
        ($f:ident) => {
            if payload.get(stringify!($f)).is_some() {
                am.$f = Set(payload
                    .get(stringify!($f))
                    .and_then(|v| v.as_str())
                    .map(str::to_owned));
            }
        };
    }
    macro_rules! set_int {
        ($f:ident) => {
            if payload.get(stringify!($f)).is_some() {
                am.$f = Set(payload
                    .get(stringify!($f))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32));
            }
        };
    }
    set_str!(dialplan_uuid);
    set_str!(pin_number);
    set_str!(email_address);
    set_str!(account_code);
    set_str!(flags);
    set_int!(conference_order);
    set_str!(conference_description);
    set_str!(context);
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            // Refresh the linked dialplan (PHP parity); extension/profile
            // changes re-point the conference data string.
            let dp_id = crate::ring_groups::parse_dialplan_id(
                payload.get("dialplan_uuid").and_then(|v| v.as_str()),
            )
            .or_else(|| crate::ring_groups::parse_dialplan_id(r.dialplan_uuid.as_deref()))
            .unwrap_or_else(crate::ring_groups::new_dialplan_id);
            let domain_name = crate::ring_groups::resolve_domain_name(
                db,
                domain,
                &crate::routing_db::tenant_display_name(&claims),
            )
            .await;
            let spec = conf_spec(
                &r.conference_name,
                &r.conference_extension,
                &r.conference_uuid,
                &dp_id,
                &domain_name,
                &r.conference_profile,
                r.pin_number.as_deref(),
                r.flags.as_deref(),
                r.context.clone(),
                r.conference_enabled,
                r.conference_description.clone(),
            );
            if let Err(e) =
                crate::ring_groups::sync_app_dialplan(db, Some(domain), dp_id, &spec).await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": e})),
                )
                    .into_response();
            }
            if r.dialplan_uuid.as_deref() != Some(dp_id.to_string().as_str()) {
                use sea_orm::{ActiveModelTrait, EntityTrait};
                if let Ok(Some(row)) = conferences::Entity::find_by_id(r.conference_uuid.clone())
                    .one(db)
                    .await
                {
                    let mut fix: conferences::ActiveModel = row.into();
                    fix.dialplan_uuid = Set(Some(dp_id.to_string()));
                    let _ = fix.update(db).await;
                }
            }
            (StatusCode::OK, Json(ser_conf(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    for u in conference_users::Entity::find()
        .filter(conference_users::Column::ConferenceUuid.eq(id.clone()))
        .filter(conference_users::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let uam: conference_users::ActiveModel = u.into();
        let _ = uam.delete(db).await;
    }
    let Some(r) = conf_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference not found"})),
        )
            .into_response();
    };
    let dp_link = crate::ring_groups::parse_dialplan_id(r.dialplan_uuid.as_deref());
    let am: conferences::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => {
            if let Some(dp) = dp_link {
                crate::ring_groups::delete_app_dialplan(db, domain, dp).await;
            } else {
                crate::fs_xml::invalidate_cache();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if conf_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = conference_users::Entity::find()
        .filter(conference_users::Column::ConferenceUuid.eq(id.clone()))
        .filter(conference_users::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "conference_user_uuid": r.conference_user_uuid,
                "conference_uuid": r.conference_uuid,
                "user_uuid": r.user_uuid,
            })
        })
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/conferences/{id}/users"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn add_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if conf_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference not found"})),
        )
            .into_response();
    }
    let Some(uid) = payload
        .get("user_uuid")
        .or_else(|| payload.get("userUuid"))
        .or_else(|| payload.get("uuid"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "user_uuid required"})),
        )
            .into_response();
    };
    let now = crate::routing_db::now();
    let cuid = crate::routing_db::new_uuid();
    let am = conference_users::ActiveModel {
        conference_user_uuid: Set(cuid.clone()),
        domain_id: Set(domain),
        conference_uuid: Set(id.clone()),
        user_uuid: Set(uid.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "conference_user_uuid": cuid,
                "conference_uuid": id,
                "user_uuid": uid,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- dialplan fan-out (Task 6; PHP conference_edit.php:234-252 parity) ----
//
// `conference ext@domain@profile+pin`, order 333, app `b81412e8-…`.
// Shared row/detail helpers in `crate::ring_groups`; only the spec
// builder + CRUD hooks live here.

fn conf_spec(
    name: &str,
    extension: &str,
    conf_uuid: &str,
    dialplan_uuid: &uuid::Uuid,
    domain_name: &str,
    profile: &str,
    pin: Option<&str>,
    flags: Option<&str>,
    context: Option<String>,
    enabled: bool,
    description: Option<String>,
) -> crate::ring_groups::AppDialplan {
    use crate::ring_groups::{
        fanout_action, fanout_action_inline, AppDialplan, FanoutCondition, CONF_APP_UUID,
    };
    let esc = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    };
    let pin_suffix = pin
        .filter(|p| !p.trim().is_empty())
        .map(|p| format!("+{}", p.trim()))
        .unwrap_or_default();
    let flags = flags.unwrap_or("").trim();
    let data = format!("{extension}@{domain_name}@{profile}{pin_suffix}+flags{{'{flags}'}}");
    let xml = format!(
        "<extension name=\"{}\" continue=\"\" uuid=\"{}\">\n\t<condition field=\"destination_number\" expression=\"^{}$\">\n\t\t<action application=\"answer\" data=\"\"/>\n\t\t<action application=\"set\" data=\"conference_uuid={}\" inline=\"true\"/>\n\t\t<action application=\"set\" data=\"conference_extension={}\" inline=\"true\"/>\n\t\t<action application=\"conference\" data=\"{}\"/>\n\t</condition>\n</extension>\n",
        esc(name),
        dialplan_uuid,
        esc(extension),
        esc(conf_uuid),
        esc(extension),
        esc(&data),
    );
    AppDialplan {
        name: name.to_owned(),
        number: extension.to_owned(),
        context,
        enabled,
        description,
        app_uuid: CONF_APP_UUID,
        order: 333,
        xml,
        conditions: vec![FanoutCondition {
            expression: format!("^{extension}$"),
            break_attr: None,
            group: 0,
            actions: vec![
                fanout_action("answer", ""),
                fanout_action_inline("set", &format!("conference_uuid={conf_uuid}")),
                fanout_action_inline("set", &format!("conference_extension={extension}")),
                fanout_action("conference", &data),
            ],
        }],
    }
}

// ---- xml ----
pub fn build_conference_dialplan_xml(name: &str, extension: &str) -> String {
    format!(
        "<extension name=\"{name}\"><condition field=\"destination_number\" expression=\"^{extension}$\"><action application=\"conference\" data=\"{name}@default\"/></condition></extension>"
    )
}

pub fn conferences_router() -> Router {
    Router::new()
        .route(
            "/api/v1/conferences/health",
            axum::routing::get(health_conf),
        )
        .route("/api/v1/conferences", post(create_conf).get(list_conf))
        .route("/api/v1/conferences/", post(create_conf).get(list_conf))
        .route(
            "/api/v1/conferences/{id}",
            axum::routing::get(get_conf)
                .patch(patch_conf)
                .delete(delete_conf),
        )
        .route(
            "/api/v1/conferences/{id}/users",
            axum::routing::get(list_users).post(add_user),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

#[cfg(test)]
mod tests {
    use super::build_conference_dialplan_xml;

    #[test]
    fn conference_xml_bridges_into_room() {
        let xml = build_conference_dialplan_xml("team", "3000");
        assert!(xml.contains("team") && xml.contains("3000") && xml.contains("conference"));
    }
}
