//! Email queue router — mirrors `app/email_queue/router.py` submit/list.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{email_queue}`.
//! Tenant-guarded; submit validates from/to; list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::email_queue;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct EmailSubmit {
    email_from: String,
    email_to: String,
    #[serde(default)]
    email_subject: Option<String>,
    #[serde(default)]
    email_body: Option<String>,
}

// ---- handlers ----
async fn submit_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<EmailSubmit>,
) -> impl IntoResponse {
    if payload.email_from.trim().is_empty() || payload.email_to.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "email_from and email_to required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = email_queue::ActiveModel {
        email_queue_uuid: Set(id),
        domain_id: Set(domain),
        email_from: Set(payload.email_from.trim().to_owned()),
        email_to: Set(payload.email_to.trim().to_owned()),
        email_subject: Set(payload.email_subject.clone()),
        email_body: Set(payload.email_body.clone()),
        email_status: Set("waiting".to_owned()),
        created_at: Set(now),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "email_queue_uuid": id,
            "id": id,
            "domain_id": domain,
            "email_from": payload.email_from,
            "email_to": payload.email_to,
            "email_subject": payload.email_subject,
            "email_status": "waiting",
        })),
    )
}

async fn list_email_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = email_queue::Entity::find()
        .filter(email_queue::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_from": r.email_from,
                "email_to": r.email_to,
                "email_subject": r.email_subject,
                "email_status": r.email_status,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = email_queue::Entity::find_by_id(id)
        .filter(email_queue::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_from": r.email_from,
                "email_to": r.email_to,
                "email_subject": r.email_subject,
                "email_status": r.email_status,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn email_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &uuid::Uuid,
) -> Option<email_queue::Model> {
    email_queue::Entity::find_by_id(*id)
        .filter(email_queue::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = email_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: email_queue::ActiveModel = r.into();
    if let Some(v) = payload.get("email_status").and_then(|v| v.as_str()) {
        am.email_status = Set(v.to_owned());
    }
    if let Some(v) = payload.get("email_subject").and_then(|v| v.as_str()) {
        am.email_subject = Set(Some(v.to_owned()));
    }
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_status": r.email_status,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = email_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: email_queue::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn retry_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = email_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: email_queue::ActiveModel = r.into();
    am.email_status = Set("waiting".to_owned());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "email_queue_uuid": r.email_queue_uuid,
                "email_status": r.email_status,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn email_queue_router() -> Router {
    Router::new()
        .route(
            "/api/v1/email_queue",
            post(submit_email).get(list_email_queue),
        )
        .route(
            "/api/v1/email_queue/",
            post(submit_email).get(list_email_queue),
        )
        .route(
            "/api/v1/email_queue/{id}",
            axum::routing::get(get_email)
                .patch(patch_email)
                .delete(delete_email),
        )
        .route("/api/v1/email_queue/{id}/retry", post(retry_email))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
