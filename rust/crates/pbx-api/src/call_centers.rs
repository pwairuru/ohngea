//! Call centers router — mirrors `app/call_centers/router.py`.
//!
//! Order: models → schemas → handlers → esl.
//!
//! models: `call_centers` (queues) + `call_center_agents` + `call_center_tiers`.
//! Queue join goes through the mock ESL transport (same channel store as calls).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use pbx_fs::esl::EsClient;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{call_center_agents, call_centers};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct QueueCreate {
    queue_name: String,
    #[serde(default)]
    queue_extension: Option<String>,
    #[serde(default = "default_strategy")]
    queue_strategy: String,
}

fn default_strategy() -> String {
    "longest-idle-agent".to_owned()
}

#[derive(Debug, Deserialize)]
struct AgentCreate {
    queue_id: String,
    agent_name: String,
    agent_contact: String,
}

#[derive(Debug, Deserialize)]
struct JoinReq {
    agent_name: String,
}

#[derive(Debug, Serialize)]
struct QueueOut {
    call_center_queue_uuid: String,
    queue_name: String,
    queue_extension: Option<String>,
}

// ---- handlers ----
async fn create_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<QueueCreate>,
) -> impl IntoResponse {
    if payload.queue_name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "queue_name required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = call_centers::ActiveModel {
        call_center_queue_uuid: Set(id.clone()),
        domain_id: Set(domain),
        queue_name: Set(payload.queue_name.trim().to_owned()),
        queue_extension: Set(payload.queue_extension.clone()),
        queue_strategy: Set(payload.queue_strategy.clone()),
        queue_enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(QueueOut {
            call_center_queue_uuid: id,
            queue_name: payload.queue_name,
            queue_extension: payload.queue_extension,
        })),
    )
}

async fn create_agent(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<AgentCreate>,
) -> impl IntoResponse {
    if payload.agent_name.trim().is_empty() || payload.agent_contact.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "agent_name/agent_contact required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    // queue must exist in this tenant
    let q = call_centers::Entity::find_by_id(payload.queue_id.clone())
        .filter(call_centers::Column::DomainId.eq(domain.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if q.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "queue not found"})),
        );
    }
    let id = crate::routing_db::new_uuid();
    let am = call_center_agents::ActiveModel {
        call_center_agent_uuid: Set(id.clone()),
        domain_id: Set(domain),
        queue_id: Set(payload.queue_id),
        agent_name: Set(payload.agent_name.clone()),
        agent_contact: Set(payload.agent_contact.clone()),
        agent_status: Set("available".to_owned()),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "call_center_agent_uuid": id,
            "agent_name": payload.agent_name,
            "agent_contact": payload.agent_contact,
        })),
    )
}

async fn list_queues(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = call_centers::Entity::find()
        .filter(call_centers::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "call_center_queue_uuid": r.call_center_queue_uuid,
                "queue_name": r.queue_name,
                "queue_extension": r.queue_extension,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn join_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(qid): axum::extract::Path<String>,
    Json(payload): Json<JoinReq>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let q = call_centers::Entity::find_by_id(qid.clone())
        .filter(call_centers::Column::DomainId.eq(domain.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if q.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "queue not found"})),
        );
    }
    let agent = call_center_agents::Entity::find()
        .filter(call_center_agents::Column::QueueId.eq(qid.clone()))
        .filter(call_center_agents::Column::DomainId.eq(domain))
        .filter(call_center_agents::Column::AgentName.eq(payload.agent_name.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if agent.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "agent not found"})),
        );
    }
    // ---- esl ---- join via mocked transport (mirrors callcenter_config agent add).
    let _reply = pbx_fs::esl::MockEsClient.api(&format!(
        "callcenter_config agent add {qid} {}",
        payload.agent_name
    ));
    let _chan = pbx_fs::esl::mock_originate(&payload.agent_name, &qid);
    (
        StatusCode::OK,
        Json(
            serde_json::json!({"status": "joined", "queue_id": qid, "agent_name": payload.agent_name}),
        ),
    )
}

pub fn call_centers_router() -> Router {
    Router::new()
        .route("/api/v1/call_centers", post(create_queue).get(list_queues))
        .route("/api/v1/call_centers/", post(create_queue).get(list_queues))
        .route("/api/v1/call_centers/agents", post(create_agent))
        .route("/api/v1/call_centers/agents/", post(create_agent))
        .route("/api/v1/call_centers/{id}/join", post(join_queue))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
