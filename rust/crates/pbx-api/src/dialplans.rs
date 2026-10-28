//! Dialplans router — mirrors `app/dialplans/router.py`.
//!
//! SeaORM backed with per-request tenant txn (`BEGIN` + `SET LOCAL
//! app.domain_uuid`, FORCE RLS parity). Legacy `domain` string is carried
//! in `hostname` (compat shim); prod scoping uses `domain_id`.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialplan {
    pub dialplan_uuid: String,
    pub name: String,
    pub context: String,
    pub domain: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
struct DialplanCreate {
    name: Option<String>,
    context: Option<String>,
    domain: Option<String>,
    enabled: Option<bool>,
}

async fn create_dialplan(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<DialplanCreate>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            );
        }
    };
    let now = crate::routing_db::now();
    let id = uuid::Uuid::now_v7();
    let name = payload.name.unwrap_or_else(|| "local".to_owned());
    let context = payload.context.unwrap_or_else(|| "default".to_owned());
    let domain_str = payload.domain.unwrap_or_else(|| claims.domain_id.clone());
    let enabled = payload.enabled.unwrap_or(true);
    let am = pbx_db::entities::dialplans::ActiveModel {
        dialplan_uuid: Set(id),
        name: Set(name.clone()),
        context: Set(context.clone()),
        enabled: Set(enabled),
        domain_id: Set(domain_id),
        description: Set(None),
        dialplan_continue: Set(false),
        dialplan_order: Set(None),
        hostname: Set(Some(domain_str.clone())),
        app_uuid: Set(None),
        dialplan_number: Set(None),
        dialplan_destination: Set(None),
        dialplan_xml: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(&txn).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    let _ = txn.commit().await;
    let rec = Dialplan {
        dialplan_uuid: id.to_string(),
        name,
        context,
        domain: domain_str,
        enabled,
    };
    (StatusCode::CREATED, Json(serde_json::json!(rec)))
}

async fn list_dialplans(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})),
    };
    let domain_id = crate::routing_db::domain(&claims);
    let rows = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id))
        .all(&txn)
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let items: Vec<Dialplan> = rows
        .into_iter()
        .map(|r| Dialplan {
            dialplan_uuid: r.dialplan_uuid.to_string(),
            name: r.name,
            context: r.context,
            domain: r.hostname.unwrap_or_else(|| claims.domain_id.clone()),
            enabled: r.enabled,
        })
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn get_dialplan(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    let domain_id = crate::routing_db::domain(&claims);
    let Ok(uid) = id.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let row = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DialplanUuid.eq(uid))
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id))
        .one(&txn)
        .await
        .unwrap_or(None);
    let _ = txn.commit().await;
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!(Dialplan {
                dialplan_uuid: r.dialplan_uuid.to_string(),
                name: r.name,
                context: r.context,
                domain: r.hostname.unwrap_or_else(|| claims.domain_id.clone()),
                enabled: r.enabled,
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

pub fn dialplans_router() -> Router {
    Router::new()
        .route(
            "/api/v1/dialplans",
            post(create_dialplan).get(list_dialplans),
        )
        .route("/api/v1/dialplans/{id}", axum::routing::get(get_dialplan))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
