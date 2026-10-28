//! Fax queue router — mirrors `app/fax/router.py` send/queue + `app/fax_queue`.
//!
//! Order: models → schemas → handlers → xml (no FS XML for fax; S3/file stub note).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::fax_queue;

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct FaxQueueSubmit {
    fax_extension: String,
    destination: String,
}

#[derive(Debug, Serialize)]
struct FaxQueueOut {
    fax_queue_uuid: String,
    fax_extension: String,
    destination: String,
    status: String,
}

// ---- handlers ----
async fn submit_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<FaxQueueSubmit>,
) -> impl IntoResponse {
    if payload.fax_extension.trim().is_empty() || payload.destination.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "fax_extension/destination required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let am = fax_queue::ActiveModel {
        fax_queue_uuid: Set(id.clone()),
        domain_id: Set(domain),
        fax_extension: Set(payload.fax_extension.trim().to_owned()),
        destination: Set(payload.destination.trim().to_owned()),
        status: Set("waiting".to_owned()),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(FaxQueueOut {
            fax_queue_uuid: id,
            fax_extension: payload.fax_extension,
            destination: payload.destination,
            status: "waiting".to_owned(),
        })),
    )
}

async fn list_queue(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = fax_queue::Entity::find()
        .filter(fax_queue::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "fax_queue_uuid": r.fax_queue_uuid,
                "fax_extension": r.fax_extension,
                "destination": r.destination,
                "status": r.status,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

pub fn fax_router() -> Router {
    Router::new()
        .route("/api/v1/fax/queue", post(submit_queue).get(list_queue))
        .route("/api/v1/fax/queue/", post(submit_queue).get(list_queue))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
