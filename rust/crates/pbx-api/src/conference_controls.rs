//! Conference controls + profiles routers — mirrors
//! `app/conference_controls/router.py` and `app/conference_profiles/router.py`.
//!
//! Order: models → schemas → handlers → registry.
//!
//! Tenant scoping: every query filters `domain_id` from verified JWT claims
//! (batch-A `tenant.rs` pattern). Permission gates mirror Python
//! `require_permission(...)`: reads need `*_view`, writes need `*_add`.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

use pbx_db::entities::{
    conference_control_details, conference_controls, conference_profile_params, conference_profiles,
};

// ---- schemas ----
fn ser_ctrl(r: &conference_controls::Model) -> serde_json::Value {
    serde_json::json!({
        "conference_control_uuid": r.conference_control_uuid,
        "domain_id": r.domain_id.to_string(),
        "control_name": r.control_name,
        "control_enabled": r.control_enabled,
        "control_description": r.control_description,
    })
}

fn ser_prof(r: &conference_profiles::Model) -> serde_json::Value {
    serde_json::json!({
        "conference_profile_uuid": r.conference_profile_uuid,
        "domain_id": r.domain_id.to_string(),
        "profile_name": r.profile_name,
        "profile_enabled": r.profile_enabled,
        "profile_description": r.profile_description,
    })
}

fn not_found(detail: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"detail": detail})),
    )
}

fn propagate_conference() {
    crate::fs_xml::invalidate_key("configuration:conference.conf");
    crate::fs_xml::propagate_esl(&["reloadxml"]);
}

// ---- handlers: conference_controls ----
async fn create_ctrl(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_add") {
        return (s, j).into_response();
    }
    let Some(name) = payload
        .get("control_name")
        .or_else(|| payload.get("name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "control_name required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = conference_controls::ActiveModel {
        conference_control_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        control_name: Set(name.to_owned()),
        control_enabled: Set(payload
            .get("control_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)),
        control_description: Set(payload
            .get("control_description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::CREATED, Json(ser_ctrl(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_ctrl(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = conference_controls::Entity::find()
        .filter(conference_controls::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_ctrl).collect();
    (
        crate::paged_headers("/api/v1/conference_controls", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_ctrl(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match conference_controls::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_ctrl(&r))).into_response(),
        _ => not_found("Conference control not found").into_response(),
    }
}

async fn patch_ctrl(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = conference_controls::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return not_found("Conference control not found").into_response();
    };
    if r.domain_id != domain {
        return not_found("Conference control not found").into_response();
    }
    let mut am: conference_controls::ActiveModel = r.into();
    if let Some(v) = payload
        .get("control_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        am.control_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("control_enabled").and_then(|v| v.as_bool()) {
        am.control_enabled = Set(v);
    }
    if let Some(v) = payload.get("control_description").and_then(|v| v.as_str()) {
        am.control_description = Set(Some(v.to_owned()));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::OK, Json(ser_ctrl(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_ctrl(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match conference_controls::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: conference_controls::ActiveModel = r.into();
            match am.delete(db).await {
                Ok(_) => {
                    propagate_conference();
                    StatusCode::NO_CONTENT.into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
                )
                    .into_response(),
            }
        }
        _ => not_found("Conference control not found").into_response(),
    }
}

// ---- handlers: conference_profiles ----
async fn create_prof(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_add") {
        return (s, j).into_response();
    }
    let Some(name) = payload
        .get("profile_name")
        .or_else(|| payload.get("name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "profile_name required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = conference_profiles::ActiveModel {
        conference_profile_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        profile_name: Set(name.to_owned()),
        profile_enabled: Set(payload
            .get("profile_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)),
        profile_description: Set(payload
            .get("profile_description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::CREATED, Json(ser_prof(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_prof(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = conference_profiles::Entity::find()
        .filter(conference_profiles::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_prof).collect();
    (
        crate::paged_headers("/api/v1/conference_profiles", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_prof(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match conference_profiles::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_prof(&r))).into_response(),
        _ => not_found("Conference profile not found").into_response(),
    }
}

async fn patch_prof(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = conference_profiles::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return not_found("Conference profile not found").into_response();
    };
    if r.domain_id != domain {
        return not_found("Conference profile not found").into_response();
    }
    let mut am: conference_profiles::ActiveModel = r.into();
    if let Some(v) = payload
        .get("profile_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        am.profile_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("profile_enabled").and_then(|v| v.as_bool()) {
        am.profile_enabled = Set(v);
    }
    if let Some(v) = payload.get("profile_description").and_then(|v| v.as_str()) {
        am.profile_description = Set(Some(v.to_owned()));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::OK, Json(ser_prof(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_prof(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match conference_profiles::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: conference_profiles::ActiveModel = r.into();
            match am.delete(db).await {
                Ok(_) => {
                    propagate_conference();
                    StatusCode::NO_CONTENT.into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
                )
                    .into_response(),
            }
        }
        _ => not_found("Conference profile not found").into_response(),
    }
}

// ---- registry ----
pub fn conference_controls_router() -> Router {
    Router::new()
        .route(
            "/api/v1/conference_controls",
            post(create_ctrl).get(list_ctrl),
        )
        .route(
            "/api/v1/conference_controls/{id}",
            axum::routing::get(get_ctrl)
                .patch(patch_ctrl)
                .delete(delete_ctrl),
        )
        .route(
            "/api/v1/conference_profiles",
            post(create_prof).get(list_prof),
        )
        .route(
            "/api/v1/conference_profiles/{id}",
            axum::routing::get(get_prof)
                .patch(patch_prof)
                .delete(delete_prof),
        )
        .route(
            "/api/v1/conference_control_details",
            post(create_ctrl_detail).get(list_ctrl_detail),
        )
        .route(
            "/api/v1/conference_control_details/{id}",
            axum::routing::get(get_ctrl_detail)
                .patch(patch_ctrl_detail)
                .delete(delete_ctrl_detail),
        )
        .route(
            "/api/v1/conference_profile_params",
            post(create_prof_param).get(list_prof_param),
        )
        .route(
            "/api/v1/conference_profile_params/{id}",
            axum::routing::get(get_prof_param)
                .patch(patch_prof_param)
                .delete(delete_prof_param),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

// ---- conference_control_details (PHP v_conference_control_details) ----
fn ser_ctrl_detail(r: &conference_control_details::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "conference_control_detail_uuid": r.conference_control_detail_uuid,
        "conference_control_uuid": r.conference_control_uuid,
        "control_digits": r.control_digits,
        "control_action": r.control_action,
        "control_data": r.control_data,
        "control_enabled": r.control_enabled,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn create_ctrl_detail(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = conference_control_details::ActiveModel {
        conference_control_detail_uuid: Set(crate::routing_db::new_uuid()),
        conference_control_uuid: Set(payload
            .get("conference_control_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        domain_id: Set(domain),
        control_digits: Set(payload
            .get("control_digits")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        control_action: Set(payload
            .get("control_action")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        control_data: Set(payload
            .get("control_data")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        control_enabled: Set(payload.get("control_enabled").and_then(|v| v.as_bool())),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::CREATED, Json(ser_ctrl_detail(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_ctrl_detail(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = conference_control_details::Entity::find()
        .filter(conference_control_details::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_ctrl_detail).collect();
    (
        crate::paged_headers("/api/v1/conference_control_details", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_ctrl_detail(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match conference_control_details::Entity::find_by_id(id)
        .filter(conference_control_details::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser_ctrl_detail(&r))).into_response(),
        None => not_found("Conference control detail not found").into_response(),
    }
}

async fn patch_ctrl_detail(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = conference_control_details::Entity::find_by_id(id)
        .filter(conference_control_details::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return not_found("Conference control detail not found").into_response();
    };
    let mut am: conference_control_details::ActiveModel = r.into();
    if payload.get("control_digits").is_some() {
        am.control_digits = Set(payload
            .get("control_digits")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("control_action").is_some() {
        am.control_action = Set(payload
            .get("control_action")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("control_data").is_some() {
        am.control_data = Set(payload
            .get("control_data")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("control_enabled").is_some() {
        am.control_enabled = Set(payload.get("control_enabled").and_then(|v| v.as_bool()));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::OK, Json(ser_ctrl_detail(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_ctrl_detail(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_control_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = conference_control_details::Entity::find_by_id(id)
        .filter(conference_control_details::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return not_found("Conference control detail not found").into_response();
    };
    let am: conference_control_details::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => {
            propagate_conference();
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- conference_profile_params (PHP v_conference_profile_params) ----
fn ser_prof_param(r: &conference_profile_params::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "conference_profile_param_uuid": r.conference_profile_param_uuid,
        "conference_profile_uuid": r.conference_profile_uuid,
        "profile_param_name": r.profile_param_name,
        "profile_param_value": r.profile_param_value,
        "profile_param_enabled": r.profile_param_enabled,
        "profile_param_description": r.profile_param_description,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn create_prof_param(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = conference_profile_params::ActiveModel {
        conference_profile_param_uuid: Set(crate::routing_db::new_uuid()),
        conference_profile_uuid: Set(payload
            .get("conference_profile_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        domain_id: Set(domain),
        profile_param_name: Set(payload
            .get("profile_param_name")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        profile_param_value: Set(payload
            .get("profile_param_value")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        profile_param_enabled: Set(payload
            .get("profile_param_enabled")
            .and_then(|v| v.as_bool())),
        profile_param_description: Set(payload
            .get("profile_param_description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::CREATED, Json(ser_prof_param(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_prof_param(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = conference_profile_params::Entity::find()
        .filter(conference_profile_params::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_prof_param).collect();
    (
        crate::paged_headers("/api/v1/conference_profile_params", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_prof_param(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match conference_profile_params::Entity::find_by_id(id)
        .filter(conference_profile_params::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser_prof_param(&r))).into_response(),
        None => not_found("Conference profile param not found").into_response(),
    }
}

async fn patch_prof_param(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = conference_profile_params::Entity::find_by_id(id)
        .filter(conference_profile_params::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return not_found("Conference profile param not found").into_response();
    };
    let mut am: conference_profile_params::ActiveModel = r.into();
    if payload.get("profile_param_name").is_some() {
        am.profile_param_name = Set(payload
            .get("profile_param_name")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("profile_param_value").is_some() {
        am.profile_param_value = Set(payload
            .get("profile_param_value")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("profile_param_enabled").is_some() {
        am.profile_param_enabled = Set(payload
            .get("profile_param_enabled")
            .and_then(|v| v.as_bool()));
    }
    if payload.get("profile_param_description").is_some() {
        am.profile_param_description = Set(payload
            .get("profile_param_description")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            propagate_conference();
            (StatusCode::OK, Json(ser_prof_param(&r))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_prof_param(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_profile_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = conference_profile_params::Entity::find_by_id(id)
        .filter(conference_profile_params::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return not_found("Conference profile param not found").into_response();
    };
    let am: conference_profile_params::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => {
            propagate_conference();
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
