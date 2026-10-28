//! Contacts router — mirrors `app/contacts/router.py` core CRUD.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{contacts}`.
//! Tenant-guarded; list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::contacts;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct ContactCreate {
    name: String,
    number: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    organization: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ContactPatch {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    number: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    organization: Option<String>,
}

// ---- handlers ----
async fn create_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<ContactCreate>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() || payload.number.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name and number required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = contacts::ActiveModel {
        contact_uuid: Set(id),
        domain_id: Set(domain),
        name: Set(payload.name.trim().to_owned()),
        number: Set(payload.number.trim().to_owned()),
        email: Set(payload.email.clone()),
        organization: Set(payload.organization.clone()),
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
            "contact_uuid": id,
            "domain_id": domain,
            "name": payload.name,
            "number": payload.number,
            "email": payload.email,
            "organization": payload.organization,
        })),
    )
}

async fn list_contacts(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = contacts::Entity::find()
        .filter(contacts::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "contact_uuid": r.contact_uuid,
                "domain_id": r.domain_id,
                "name": r.name,
                "number": r.number,
                "email": r.email,
                "organization": r.organization,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = contacts::Entity::find_by_id(id)
        .filter(contacts::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "contact_uuid": r.contact_uuid,
                "domain_id": r.domain_id,
                "name": r.name,
                "number": r.number,
                "email": r.email,
                "organization": r.organization,
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

async fn patch_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<ContactPatch>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = contacts::Entity::find_by_id(id)
        .filter(contacts::Column::DomainId.eq(domain))
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
    let mut am: contacts::ActiveModel = r.into();
    if let Some(n) = payload.name {
        am.name = Set(n);
    }
    if let Some(n) = payload.number {
        am.number = Set(n);
    }
    if payload.email.is_some() {
        am.email = Set(payload.email);
    }
    if payload.organization.is_some() {
        am.organization = Set(payload.organization);
    }
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "contact_uuid": r.contact_uuid,
                "name": r.name,
                "number": r.number,
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

async fn delete_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = contacts::Entity::find_by_id(id)
        .filter(contacts::Column::DomainId.eq(domain))
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
    let am: contacts::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!(null))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn contacts_router() -> Router {
    Router::new()
        .route("/api/v1/contacts", post(create_contact).get(list_contacts))
        .route("/api/v1/contacts/", post(create_contact).get(list_contacts))
        .route(
            "/api/v1/contacts/{id}",
            axum::routing::get(get_contact)
                .patch(patch_contact)
                .delete(delete_contact),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
