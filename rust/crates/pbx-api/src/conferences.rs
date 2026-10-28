//! Conferences router — mirrors `app/conferences/router.py`.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{conference_users, conferences};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct ConferenceCreate {
    conference_name: String,
    conference_extension: String,
    #[serde(default = "default_profile")]
    conference_profile: String,
}

fn default_profile() -> String {
    "default".to_owned()
}

#[derive(Debug, Serialize)]
struct ConferenceOut {
    conference_uuid: String,
    conference_name: String,
    conference_extension: String,
}

// ---- handlers ----
fn valid_extension(ext: &str) -> bool {
    let s = ext.trim();
    (2..=7).contains(&s.len()) && s.chars().all(|c| c.is_ascii_digit())
}

fn ser_conf(r: &conferences::Model) -> serde_json::Value {
    serde_json::json!({
        "conference_uuid": r.conference_uuid,
        "conference_name": r.conference_name,
        "conference_extension": r.conference_extension,
        "conference_profile": r.conference_profile,
        "conference_enabled": r.conference_enabled,
    })
}

async fn conf_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<conferences::Model> {
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
            Json(serde_json::json!({"detail": "conference_extension must be 2-7 digits"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = conferences::ActiveModel {
        conference_uuid: Set(id.clone()),
        domain_id: Set(domain),
        conference_name: Set(payload.conference_name.trim().to_owned()),
        conference_extension: Set(payload.conference_extension.trim().to_owned()),
        conference_profile: Set(payload.conference_profile),
        conference_enabled: Set(true),
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
    (
        StatusCode::CREATED,
        Json(serde_json::json!(ConferenceOut {
            conference_uuid: id,
            conference_name: payload.conference_name,
            conference_extension: payload.conference_extension,
        })),
    )
        .into_response()
}

async fn list_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = conferences::Entity::find()
        .filter(conferences::Column::DomainId.eq(domain))
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
                "conference_uuid": r.conference_uuid,
                "conference_name": r.conference_name,
                "conference_extension": r.conference_extension,
            })
        })
        .collect();
    (
        crate::paged_headers("/api/v1/conferences", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn health_conf() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "conferences"}))
}

async fn get_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
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
                Json(serde_json::json!({"detail": "conference_extension must be 2-7 digits"})),
            )
                .into_response();
        }
        am.conference_extension = Set(v.to_owned());
    }
    if let Some(v) = payload.get("conference_profile").and_then(|v| v.as_str()) {
        am.conference_profile = Set(v.to_owned());
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_conf(&r))).into_response(),
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
    let am: conferences::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
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
