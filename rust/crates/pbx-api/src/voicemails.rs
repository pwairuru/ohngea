//! Voicemails router — mirrors `app/voicemails/router.py` + greetings/messages/options.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{voicemail_greetings, voicemail_messages, voicemail_options, voicemails};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct VoicemailCreate {
    voicemail_id: String,
    voicemail_password: String,
}

#[derive(Debug, Deserialize)]
struct GreetingCreate {
    greeting_name: String,
    #[serde(default)]
    greeting_text: Option<String>,
}

#[derive(Debug, Serialize)]
struct VoicemailOut {
    voicemail_uuid: String,
    voicemail_id: String,
}

// ---- handlers ----
fn valid_mailbox(s: &str) -> bool {
    let t = s.trim();
    (2..=7).contains(&t.len()) && t.chars().all(|c| c.is_ascii_digit())
}

fn ser_vm(r: &voicemails::Model) -> serde_json::Value {
    serde_json::json!({
        "voicemail_uuid": r.voicemail_uuid,
        "voicemail_id": r.voicemail_id,
        "enabled": r.enabled,
    })
}

async fn vm_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<voicemails::Model> {
    voicemails::Entity::find_by_id(id.to_owned())
        .filter(voicemails::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn create_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<VoicemailCreate>,
) -> impl IntoResponse {
    if !valid_mailbox(&payload.voicemail_id) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "voicemail_id must be 2-7 digits"})),
        )
            .into_response();
    }
    if payload.voicemail_password.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "voicemail_password required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = voicemails::ActiveModel {
        voicemail_uuid: Set(id.clone()),
        domain_id: Set(domain),
        voicemail_id: Set(payload.voicemail_id.trim().to_owned()),
        voicemail_password: Set(payload.voicemail_password),
        enabled: Set(true),
        description: Set(None),
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
        Json(serde_json::json!(VoicemailOut {
            voicemail_uuid: id,
            voicemail_id: payload.voicemail_id,
        })),
    )
        .into_response()
}

async fn list_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = voicemails::Entity::find()
        .filter(voicemails::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_vm)
        .collect();
    (
        crate::paged_headers("/api/v1/voicemails", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn health_vm() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "voicemails"}))
}

async fn get_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match vm_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_vm(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response(),
    }
}

async fn patch_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = vm_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    };
    let mut am: voicemails::ActiveModel = r.into();
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_vm(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    for m in voicemail_messages::Entity::find()
        .filter(voicemail_messages::Column::VoicemailUuid.eq(id.clone()))
        .filter(voicemail_messages::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let mam: voicemail_messages::ActiveModel = m.into();
        let _ = mam.delete(db).await;
    }
    for o in voicemail_options::Entity::find()
        .filter(voicemail_options::Column::VoicemailUuid.eq(id.clone()))
        .filter(voicemail_options::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let oam: voicemail_options::ActiveModel = o.into();
        let _ = oam.delete(db).await;
    }
    for g in voicemail_greetings::Entity::find()
        .filter(voicemail_greetings::Column::VoicemailUuid.eq(id.clone()))
        .filter(voicemail_greetings::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let gam: voicemail_greetings::ActiveModel = g.into();
        let _ = gam.delete(db).await;
    }
    let Some(r) = vm_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    };
    let am: voicemails::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- messages ----
fn ser_msg(r: &voicemail_messages::Model) -> serde_json::Value {
    serde_json::json!({
        "voicemail_message_uuid": r.voicemail_message_uuid,
        "voicemail_uuid": r.voicemail_uuid,
        "file_name": r.file_name,
        "message_length": r.message_length,
    })
}

async fn list_messages(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if vm_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    }
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let rows = voicemail_messages::Entity::find()
        .filter(voicemail_messages::Column::VoicemailUuid.eq(id.clone()))
        .filter(voicemail_messages::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_msg)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/voicemails/{id}/messages"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_message(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if vm_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    }
    let mid = crate::routing_db::new_uuid();
    let am = voicemail_messages::ActiveModel {
        voicemail_message_uuid: Set(mid.clone()),
        domain_id: Set(domain),
        voicemail_uuid: Set(id.clone()),
        file_name: Set(payload
            .get("file_name")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        message_length: Set(payload
            .get("message_length")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "voicemail_message_uuid": mid,
                "voicemail_uuid": id,
                "file_name": payload.get("file_name"),
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

async fn msg_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    vm: &str,
    mid: &str,
) -> Option<voicemail_messages::Model> {
    voicemail_messages::Entity::find_by_id(mid.to_owned())
        .filter(voicemail_messages::Column::VoicemailUuid.eq(vm.to_owned()))
        .filter(voicemail_messages::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_message(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, mid)): axum::extract::Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = msg_or_404(db, domain, &id, &mid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Message not found"})),
        )
            .into_response();
    };
    let mut am: voicemail_messages::ActiveModel = r.into();
    if let Some(v) = payload.get("file_name").and_then(|v| v.as_str()) {
        am.file_name = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("message_length").and_then(|v| v.as_i64()) {
        am.message_length = Set(Some(v as i32));
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_msg(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_message(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, mid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = msg_or_404(db, domain, &id, &mid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Message not found"})),
        )
            .into_response();
    };
    let am: voicemail_messages::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn playback_message(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, mid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = msg_or_404(db, domain, &id, &mid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Message not found"})),
        )
            .into_response();
    };
    let Some(f) = r.file_name else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Message has no audio file"})),
        )
            .into_response();
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "voicemail_message_uuid": r.voicemail_message_uuid,
            "file_name": f,
            "playback_url": format!("/api/v1/voicemails/{id}/messages/{mid}/playback"),
        })),
    )
        .into_response()
}

// ---- options ----
fn ser_opt(r: &voicemail_options::Model) -> serde_json::Value {
    serde_json::json!({
        "voicemail_option_uuid": r.voicemail_option_uuid,
        "voicemail_uuid": r.voicemail_uuid,
        "digits": r.digits,
        "action": r.action,
        "param": r.param,
    })
}

async fn opt_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    vm: &str,
    oid: &str,
) -> Option<voicemail_options::Model> {
    voicemail_options::Entity::find_by_id(oid.to_owned())
        .filter(voicemail_options::Column::VoicemailUuid.eq(vm.to_owned()))
        .filter(voicemail_options::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn list_options(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if vm_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    }
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let rows = voicemail_options::Entity::find()
        .filter(voicemail_options::Column::VoicemailUuid.eq(id.clone()))
        .filter(voicemail_options::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_opt)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/voicemails/{id}/options"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_option(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if vm_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let oid = crate::routing_db::new_uuid();
    let am = voicemail_options::ActiveModel {
        voicemail_option_uuid: Set(oid.clone()),
        domain_id: Set(domain),
        voicemail_uuid: Set(id.clone()),
        digits: Set(payload
            .get("digits")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        action: Set(payload
            .get("action")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        param: Set(payload
            .get("param")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        description: Set(payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "voicemail_option_uuid": oid,
                "voicemail_uuid": id,
                "digits": payload.get("digits"),
                "action": payload.get("action"),
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

async fn patch_option(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, oid)): axum::extract::Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = opt_or_404(db, domain, &id, &oid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Option not found"})),
        )
            .into_response();
    };
    let mut am: voicemail_options::ActiveModel = r.into();
    if let Some(v) = payload.get("digits").and_then(|v| v.as_str()) {
        am.digits = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("action").and_then(|v| v.as_str()) {
        am.action = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("param").and_then(|v| v.as_str()) {
        am.param = Set(Some(v.to_owned()));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_opt(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_option(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, oid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = opt_or_404(db, domain, &id, &oid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Option not found"})),
        )
            .into_response();
    };
    let am: voicemail_options::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- greetings (nested create + standalone CRUD) ----
async fn create_greeting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<GreetingCreate>,
) -> impl IntoResponse {
    if payload.greeting_name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "greeting_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if vm_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    }
    let gid = crate::routing_db::new_uuid();
    let gam = voicemail_greetings::ActiveModel {
        voicemail_greeting_uuid: Set(gid.clone()),
        domain_id: Set(domain),
        voicemail_uuid: Set(id.clone()),
        greeting_name: Set(payload.greeting_name.clone()),
        greeting_text: Set(payload.greeting_text.clone()),
    };
    if gam.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "voicemail_greeting_uuid": gid,
            "voicemail_uuid": id,
            "greeting_name": payload.greeting_name,
        })),
    )
        .into_response()
}

fn ser_greeting(r: &voicemail_greetings::Model) -> serde_json::Value {
    serde_json::json!({
        "voicemail_greeting_uuid": r.voicemail_greeting_uuid,
        "voicemail_uuid": r.voicemail_uuid,
        "greeting_name": r.greeting_name,
        "greeting_text": r.greeting_text,
    })
}

async fn list_greetings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = voicemail_greetings::Entity::find()
        .filter(voicemail_greetings::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_greeting)
        .collect();
    (
        crate::paged_headers("/api/v1/voicemail_greetings", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn create_greeting_standalone(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Some(name) = payload
        .get("greeting_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "greeting_name required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    // Parent link: voicemail_uuid direct, else voicemail_id mailbox lookup.
    // Python stores a free-form mailbox string; Rust enforces the FK link,
    // so an unresolvable mailbox is 404 (mirrors nested-route behavior).
    let mut vm_id = payload
        .get("voicemail_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    if vm_id.is_empty() {
        if let Some(mbox) = payload
            .get("voicemail_id")
            .or_else(|| payload.get("mailbox"))
            .and_then(|v| v.as_str())
        {
            let row = voicemails::Entity::find()
                .filter(voicemails::Column::VoicemailId.eq(mbox.to_owned()))
                .filter(voicemails::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(v) => vm_id = v.voicemail_uuid,
                None => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({"detail": "Voicemail not found"})),
                    )
                        .into_response();
                }
            }
        }
    }
    if !vm_id.is_empty() && vm_or_404(db, domain, &vm_id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Voicemail not found"})),
        )
            .into_response();
    }
    let gid = crate::routing_db::new_uuid();
    let gam = voicemail_greetings::ActiveModel {
        voicemail_greeting_uuid: Set(gid.clone()),
        domain_id: Set(domain),
        voicemail_uuid: Set(vm_id.clone()),
        greeting_name: Set(name.to_owned()),
        greeting_text: Set(payload
            .get("greeting_text")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
    };
    match gam.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "voicemail_greeting_uuid": gid,
                "voicemail_uuid": vm_id,
                "greeting_name": name,
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

async fn greeting_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    gid: &str,
) -> Option<voicemail_greetings::Model> {
    voicemail_greetings::Entity::find_by_id(gid.to_owned())
        .filter(voicemail_greetings::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn get_greeting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match greeting_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_greeting(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Greeting not found"})),
        )
            .into_response(),
    }
}

async fn patch_greeting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = greeting_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Greeting not found"})),
        )
            .into_response();
    };
    let mut am: voicemail_greetings::ActiveModel = r.into();
    if let Some(v) = payload.get("greeting_name").and_then(|v| v.as_str()) {
        am.greeting_name = Set(v.to_owned());
    }
    if payload.get("greeting_text").is_some() {
        am.greeting_text = Set(payload
            .get("greeting_text")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_greeting(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_greeting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = greeting_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Greeting not found"})),
        )
            .into_response();
    };
    let am: voicemail_greetings::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- xml ----
pub fn build_voicemail_directory_xml(mailbox: &str, password: &str, domain: &str) -> String {
    format!(
        "<user id=\"{mailbox}@{domain}\"><params><param name=\"password\" value=\"{password}\"/></params></user>"
    )
}

pub fn voicemails_router() -> Router {
    Router::new()
        .route("/api/v1/voicemails/health", axum::routing::get(health_vm))
        .route("/api/v1/voicemails", post(create_vm).get(list_vm))
        .route("/api/v1/voicemails/", post(create_vm).get(list_vm))
        .route(
            "/api/v1/voicemails/{id}",
            axum::routing::get(get_vm).patch(patch_vm).delete(delete_vm),
        )
        .route(
            "/api/v1/voicemails/{id}/messages",
            axum::routing::get(list_messages).post(create_message),
        )
        .route(
            "/api/v1/voicemails/{id}/messages/{mid}",
            axum::routing::patch(patch_message).delete(delete_message),
        )
        .route(
            "/api/v1/voicemails/{id}/messages/{mid}/playback",
            axum::routing::get(playback_message),
        )
        .route(
            "/api/v1/voicemails/{id}/options",
            axum::routing::get(list_options).post(create_option),
        )
        .route(
            "/api/v1/voicemails/{id}/options/{oid}",
            axum::routing::patch(patch_option).delete(delete_option),
        )
        .route("/api/v1/voicemails/{id}/greetings", post(create_greeting))
        .route(
            "/api/v1/voicemail_greetings",
            post(create_greeting_standalone).get(list_greetings),
        )
        .route(
            "/api/v1/voicemail_greetings/",
            post(create_greeting_standalone).get(list_greetings),
        )
        .route(
            "/api/v1/voicemail_greetings/{id}",
            axum::routing::get(get_greeting)
                .patch(patch_greeting)
                .delete(delete_greeting),
        )
        // Singular alias: plugin manifest route is `/voicemail`
        // (ground truth `plugins/voicemail/manifest.toml`); core wins
        // per `app/main.py` skip list, so the alias reuses core handlers.
        .route("/api/v1/voicemail", post(create_vm).get(list_vm))
        .route("/api/v1/voicemail/", post(create_vm).get(list_vm))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

#[cfg(test)]
mod tests {
    use super::build_voicemail_directory_xml;

    #[test]
    fn voicemail_xml_carries_mailbox_and_domain() {
        let xml = build_voicemail_directory_xml("1001", "1234", "example.com");
        assert!(xml.contains("1001") && xml.contains("example.com") && xml.contains("1234"));
    }
}
