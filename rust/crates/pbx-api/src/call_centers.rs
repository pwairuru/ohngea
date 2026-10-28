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
use pbx_db::entities::{call_center_agents, call_center_tiers, call_centers};

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
    #[serde(default)]
    queue_id: Option<String>,
    agent_name: String,
    agent_contact: String,
}

#[derive(Debug, Deserialize)]
struct TierCreate {
    agent_name: String,
    #[serde(default = "default_tier_level")]
    tier_level: i32,
    #[serde(default = "default_tier_position")]
    tier_position: i32,
}

fn default_tier_level() -> i32 {
    1
}
fn default_tier_position() -> i32 {
    1
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
fn ser_queue(r: &call_centers::Model) -> serde_json::Value {
    serde_json::json!({
        "call_center_queue_uuid": r.call_center_queue_uuid,
        "queue_name": r.queue_name,
        "queue_extension": r.queue_extension,
        "queue_strategy": r.queue_strategy,
    })
}

fn ser_agent(r: &call_center_agents::Model) -> serde_json::Value {
    serde_json::json!({
        "call_center_agent_uuid": r.call_center_agent_uuid,
        "queue_id": r.queue_id,
        "agent_name": r.agent_name,
        "agent_contact": r.agent_contact,
        "agent_status": r.agent_status,
    })
}

fn ser_tier(r: &call_center_tiers::Model) -> serde_json::Value {
    serde_json::json!({
        "call_center_tier_uuid": r.call_center_tier_uuid,
        "queue_id": r.queue_id,
        "agent_name": r.agent_name,
        "tier_level": r.tier_level,
        "tier_position": r.tier_position,
    })
}

async fn queue_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<call_centers::Model> {
    call_centers::Entity::find_by_id(id.to_owned())
        .filter(call_centers::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn create_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<QueueCreate>,
) -> impl IntoResponse {
    if payload.queue_name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "queue_name required"})),
        )
            .into_response();
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
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(QueueOut {
            call_center_queue_uuid: id,
            queue_name: payload.queue_name,
            queue_extension: payload.queue_extension,
        })),
    )
        .into_response()
}

async fn list_queues(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = call_centers::Entity::find()
        .filter(call_centers::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_queue)
        .collect();
    (
        crate::paged_headers("/api/v1/call_centers", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn get_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match queue_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_queue(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call center not found"})),
        )
            .into_response(),
    }
}

async fn patch_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call center not found"})),
        )
            .into_response();
    };
    let mut am: call_centers::ActiveModel = r.into();
    if let Some(v) = payload.get("queue_name").and_then(|v| v.as_str()) {
        am.queue_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("queue_extension").and_then(|v| v.as_str()) {
        am.queue_extension = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("queue_strategy").and_then(|v| v.as_str()) {
        am.queue_strategy = Set(v.to_owned());
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_queue(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    for a in call_center_agents::Entity::find()
        .filter(call_center_agents::Column::QueueId.eq(id.clone()))
        .filter(call_center_agents::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let aam: call_center_agents::ActiveModel = a.into();
        let _ = aam.delete(db).await;
    }
    for t in call_center_tiers::Entity::find()
        .filter(call_center_tiers::Column::QueueId.eq(id.clone()))
        .filter(call_center_tiers::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let tam: call_center_tiers::ActiveModel = t.into();
        let _ = tam.delete(db).await;
    }
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call center not found"})),
        )
            .into_response();
    };
    let am: call_centers::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- agents (nested under queue; legacy top-level POST kept) ----
async fn insert_agent(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    queue_id: &str,
    payload: &AgentCreate,
) -> Result<call_center_agents::Model, String> {
    if payload.agent_name.trim().is_empty() || payload.agent_contact.trim().is_empty() {
        return Err("agent_name/agent_contact required".to_owned());
    }
    if queue_or_404(db, domain, queue_id).await.is_none() {
        return Err("NOT_FOUND".to_owned());
    }
    let id = crate::routing_db::new_uuid();
    let am = call_center_agents::ActiveModel {
        call_center_agent_uuid: Set(id),
        domain_id: Set(domain),
        queue_id: Set(queue_id.to_owned()),
        agent_name: Set(payload.agent_name.clone()),
        agent_contact: Set(payload.agent_contact.clone()),
        agent_status: Set("available".to_owned()),
    };
    am.insert(db)
        .await
        .map_err(|e| format!("insert failed: {e}"))
}

async fn create_agent_legacy(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<AgentCreate>,
) -> impl IntoResponse {
    let Some(qid) = payload.queue_id.clone() else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "queue_id required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match insert_agent(db, domain, &qid, &payload).await {
        Ok(r) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "call_center_agent_uuid": r.call_center_agent_uuid,
                "agent_name": r.agent_name,
                "agent_contact": r.agent_contact,
            })),
        )
            .into_response(),
        Err(e) if e == "NOT_FOUND" => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "queue not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": e})),
        )
            .into_response(),
    }
}

async fn list_agents(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(qid): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if queue_or_404(db, domain, &qid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call center not found"})),
        )
            .into_response();
    }
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let rows = call_center_agents::Entity::find()
        .filter(call_center_agents::Column::QueueId.eq(qid.clone()))
        .filter(call_center_agents::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_agent)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/call_centers/{qid}/agents"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_agent_nested(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(qid): axum::extract::Path<String>,
    Json(payload): Json<AgentCreate>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match insert_agent(db, domain, &qid, &payload).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_agent(&r))).into_response(),
        Err(e) if e == "NOT_FOUND" => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call center not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": e})),
        )
            .into_response(),
    }
}

async fn agent_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    qid: &str,
    aid: &str,
) -> Option<call_center_agents::Model> {
    call_center_agents::Entity::find_by_id(aid.to_owned())
        .filter(call_center_agents::Column::QueueId.eq(qid.to_owned()))
        .filter(call_center_agents::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_agent(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((qid, aid)): axum::extract::Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = agent_or_404(db, domain, &qid, &aid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Agent not found"})),
        )
            .into_response();
    };
    let mut am: call_center_agents::ActiveModel = r.into();
    if let Some(v) = payload.get("agent_contact").and_then(|v| v.as_str()) {
        am.agent_contact = Set(v.to_owned());
    }
    if let Some(v) = payload.get("agent_status").and_then(|v| v.as_str()) {
        am.agent_status = Set(v.to_owned());
    }
    if let Some(v) = payload.get("agent_name").and_then(|v| v.as_str()) {
        am.agent_name = Set(v.to_owned());
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_agent(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_agent(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((qid, aid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = agent_or_404(db, domain, &qid, &aid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Agent not found"})),
        )
            .into_response();
    };
    let am: call_center_agents::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- tiers ----
async fn list_tiers(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(qid): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if queue_or_404(db, domain, &qid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call center not found"})),
        )
            .into_response();
    }
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let rows = call_center_tiers::Entity::find()
        .filter(call_center_tiers::Column::QueueId.eq(qid.clone()))
        .filter(call_center_tiers::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_tier)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/call_centers/{qid}/tiers"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_tier(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(qid): axum::extract::Path<String>,
    Json(payload): Json<TierCreate>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if queue_or_404(db, domain, &qid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call center not found"})),
        )
            .into_response();
    }
    if payload.agent_name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "agent_name required"})),
        )
            .into_response();
    }
    let id = crate::routing_db::new_uuid();
    let am = call_center_tiers::ActiveModel {
        call_center_tier_uuid: Set(id.clone()),
        domain_id: Set(domain),
        queue_id: Set(qid.clone()),
        agent_name: Set(payload.agent_name.clone()),
        tier_level: Set(payload.tier_level),
        tier_position: Set(payload.tier_position),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "call_center_tier_uuid": id,
                "queue_id": qid,
                "agent_name": payload.agent_name,
                "tier_level": payload.tier_level,
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

async fn tier_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    qid: &str,
    tid: &str,
) -> Option<call_center_tiers::Model> {
    call_center_tiers::Entity::find_by_id(tid.to_owned())
        .filter(call_center_tiers::Column::QueueId.eq(qid.to_owned()))
        .filter(call_center_tiers::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_tier(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((qid, tid)): axum::extract::Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = tier_or_404(db, domain, &qid, &tid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Tier not found"})),
        )
            .into_response();
    };
    let mut am: call_center_tiers::ActiveModel = r.into();
    if let Some(v) = payload.get("tier_level").and_then(|v| v.as_i64()) {
        am.tier_level = Set(v as i32);
    }
    if let Some(v) = payload.get("tier_position").and_then(|v| v.as_i64()) {
        am.tier_position = Set(v as i32);
    }
    if let Some(v) = payload.get("agent_name").and_then(|v| v.as_str()) {
        am.agent_name = Set(v.to_owned());
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_tier(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_tier(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((qid, tid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = tier_or_404(db, domain, &qid, &tid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Tier not found"})),
        )
            .into_response();
    };
    let am: call_center_tiers::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn join_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(qid): axum::extract::Path<String>,
    Json(payload): Json<JoinReq>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if queue_or_404(db, domain, &qid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "queue not found"})),
        )
            .into_response();
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
        )
            .into_response();
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
        .into_response()
}

pub fn call_centers_router() -> Router {
    Router::new()
        .route("/api/v1/call_centers", post(create_queue).get(list_queues))
        .route("/api/v1/call_centers/", post(create_queue).get(list_queues))
        .route(
            "/api/v1/call_centers/{id}",
            axum::routing::get(get_queue)
                .patch(patch_queue)
                .delete(delete_queue),
        )
        .route("/api/v1/call_centers/agents", post(create_agent_legacy))
        .route("/api/v1/call_centers/agents/", post(create_agent_legacy))
        .route(
            "/api/v1/call_centers/{id}/agents",
            axum::routing::get(list_agents).post(create_agent_nested),
        )
        .route(
            "/api/v1/call_centers/{id}/agents/{aid}",
            axum::routing::patch(patch_agent).delete(delete_agent),
        )
        .route(
            "/api/v1/call_centers/{id}/tiers",
            axum::routing::get(list_tiers).post(create_tier),
        )
        .route(
            "/api/v1/call_centers/{id}/tiers/{tid}",
            axum::routing::patch(patch_tier).delete(delete_tier),
        )
        .route("/api/v1/call_centers/{id}/join", post(join_queue))
        // Singular alias: plugin manifest route is `/call_center`
        // (ground truth `plugins/call_center/manifest.toml`); core wins
        // per `app/main.py` skip list, so the alias reuses core handlers.
        .route("/api/v1/call_center", post(create_queue).get(list_queues))
        .route("/api/v1/call_center/", post(create_queue).get(list_queues))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
