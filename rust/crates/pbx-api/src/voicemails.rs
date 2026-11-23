//! Voicemails router — mirrors `app/voicemails/router.py` + greetings/messages/options.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::Deserialize;

// ---- models ----
use pbx_db::entities::{voicemail_greetings, voicemail_messages, voicemail_options, voicemails};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct VoicemailCreate {
    voicemail_id: String,
    voicemail_password: String,
    #[serde(default)] description: Option<String>,
    #[serde(default)] enabled: Option<bool>,
    #[serde(default)] voicemail_tutorial: Option<bool>,
    #[serde(default)] greeting_id: Option<String>,
    #[serde(default, alias = "alternate_greet_id")] voicemail_alternate_greet_id: Option<String>,
    #[serde(default)] voicemail_recording_instructions: Option<bool>,
    #[serde(default)] voicemail_recording_options: Option<String>,
    #[serde(default)] voicemail_mail_to: Option<String>,
    #[serde(default)] voicemail_sms_to: Option<String>,
    #[serde(default)] voicemail_transcription_enabled: Option<bool>,
    #[serde(default)] voicemail_attach_file: Option<bool>,
    #[serde(default)] voicemail_file: Option<String>,
    #[serde(default)] voicemail_local_after_email: Option<bool>,
    #[serde(default)] voicemail_local_after_forward: Option<bool>,
    #[serde(default)] voicemail_transcription_prompt_enabled: Option<bool>,
    #[serde(default)] voicemail_transcription_prompt: Option<String>,
    #[serde(default)] alternate_voicemail_destination: Option<String>,
    #[serde(default)] alternate_voicemail_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GreetingCreate {
    greeting_name: String,
    #[serde(default)]
    greeting_text: Option<String>,
    #[serde(default)]
    greeting_id: Option<String>,
    #[serde(default)]
    greeting_voice: Option<String>,
    #[serde(default)]
    greeting_message: Option<String>,
    #[serde(default)]
    greeting_filename: Option<String>,
    #[serde(default)]
    greeting_description: Option<String>,
    #[serde(default)]
    greeting_base64: Option<String>,
}

// ---- handlers ----
fn valid_mailbox(s: &str) -> bool {
    let t = s.trim();
    (2..=7).contains(&t.len()) && t.chars().all(|c| c.is_ascii_digit())
}

fn ser_vm(r: &voicemails::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "voicemail_uuid": r.voicemail_uuid,
        "voicemail_id": r.voicemail_id,
        "enabled": r.enabled,
        "description": r.description,
        "voicemail_tutorial": r.voicemail_tutorial,
        "greeting_id": r.greeting_id,
        "voicemail_alternate_greet_id": r.voicemail_alternate_greet_id,
        "voicemail_recording_instructions": r.voicemail_recording_instructions,
        "voicemail_recording_options": r.voicemail_recording_options,
        "voicemail_mail_to": r.voicemail_mail_to,
        "voicemail_sms_to": r.voicemail_sms_to,
        "voicemail_transcription_enabled": r.voicemail_transcription_enabled,
        "voicemail_transcription_prompt_enabled": r.voicemail_transcription_prompt_enabled,
        "voicemail_transcription_prompt": r.voicemail_transcription_prompt,
        "voicemail_attach_file": r.voicemail_attach_file,
        "voicemail_file": r.voicemail_file,
        "voicemail_local_after_email": r.voicemail_local_after_email,
        "voicemail_local_after_forward": r.voicemail_local_after_forward,
        "alternate_voicemail_destination": r.alternate_voicemail_destination,
        "alternate_voicemail_enabled": r.alternate_voicemail_enabled,
    });
    crate::alias::emit_aliases(&mut v);
    v
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
        enabled: Set(payload.enabled.unwrap_or(true)),
        description: Set(payload.description.clone()),
        voicemail_tutorial: Set(payload.voicemail_tutorial),
        greeting_id: Set(payload.greeting_id.clone()),
        voicemail_alternate_greet_id: Set(payload.voicemail_alternate_greet_id.clone()),
        voicemail_recording_instructions: Set(payload.voicemail_recording_instructions),
        voicemail_recording_options: Set(payload.voicemail_recording_options.clone()),
        voicemail_mail_to: Set(payload.voicemail_mail_to.clone()),
        voicemail_sms_to: Set(payload.voicemail_sms_to.clone()),
        voicemail_transcription_enabled: Set(payload.voicemail_transcription_enabled),
        voicemail_transcription_prompt_enabled: Set(payload.voicemail_transcription_prompt_enabled),
        voicemail_transcription_prompt: Set(payload.voicemail_transcription_prompt.clone()),
        voicemail_attach_file: Set(payload.voicemail_attach_file),
        voicemail_file: Set(payload.voicemail_file.clone()),
        voicemail_local_after_email: Set(payload.voicemail_local_after_email),
        voicemail_local_after_forward: Set(payload.voicemail_local_after_forward),
        alternate_voicemail_destination: Set(payload.alternate_voicemail_destination.clone()),
        alternate_voicemail_enabled: Set(payload.alternate_voicemail_enabled),
        voicemail_name_base64: Set(None),
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
    match vm_or_404(db, domain, &id).await {
        Some(r) => {
            // Directory joins voicemails (Task 3) — invalidate so the new
            // mailbox shows up in served directory XML.
            crate::fs_xml::invalidate_cache();
            (StatusCode::CREATED, Json(ser_vm(&r))).into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response(),
    }
}

async fn list_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "voicemail_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/voicemails", page, per_page, 0),
                Json(crate::paged_body(Vec::<serde_json::Value>::new(), 0, page, per_page)),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let mut sel = voicemails::Entity::find();
    if !show_all {
        sel = sel.filter(voicemails::Column::DomainId.eq(domain));
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_vm).collect();
    (
        crate::paged_headers("/api/v1/voicemails", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn health_vm() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "voicemails"})).into_response()
}

async fn get_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_view") {
        return (s, j).into_response();
    }
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
    for key in ["voicemail_mail_to", "voicemail_sms_to", "voicemail_file", "voicemail_transcription_prompt", "greeting_id", "voicemail_alternate_greet_id", "alternate_voicemail_destination", "voicemail_recording_options", "description"] {
        if key == "description" {
            continue;
        }
        if let Some(v) = payload.get(key).and_then(|v| v.as_str()) {
            match key {
                "voicemail_mail_to" => am.voicemail_mail_to = Set(Some(v.to_owned())),
                "voicemail_sms_to" => am.voicemail_sms_to = Set(Some(v.to_owned())),
                "voicemail_file" => am.voicemail_file = Set(Some(v.to_owned())),
                "voicemail_transcription_prompt" => am.voicemail_transcription_prompt = Set(Some(v.to_owned())),
                "greeting_id" => am.greeting_id = Set(Some(v.to_owned())),
                "voicemail_alternate_greet_id" => am.voicemail_alternate_greet_id = Set(Some(v.to_owned())),
                "voicemail_recording_options" => am.voicemail_recording_options = Set(Some(v.to_owned())),
                _ => am.alternate_voicemail_destination = Set(Some(v.to_owned())),
            }
        }
    }
    for (key, slot) in [
        ("voicemail_tutorial", &mut am.voicemail_tutorial),
        ("voicemail_recording_instructions", &mut am.voicemail_recording_instructions),
        ("voicemail_transcription_enabled", &mut am.voicemail_transcription_enabled),
        ("voicemail_transcription_prompt_enabled", &mut am.voicemail_transcription_prompt_enabled),
        ("voicemail_attach_file", &mut am.voicemail_attach_file),
        ("voicemail_local_after_email", &mut am.voicemail_local_after_email),
        ("voicemail_local_after_forward", &mut am.voicemail_local_after_forward),
        ("alternate_voicemail_enabled", &mut am.alternate_voicemail_enabled),
    ] {
        if let Some(v) = payload.get(key).and_then(|v| v.as_bool()) {
            *slot = Set(Some(v));
        }
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            crate::fs_xml::invalidate_cache();
            (StatusCode::OK, Json(ser_vm(&r))).into_response()
        }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
        Ok(_) => {
            crate::fs_xml::invalidate_cache();
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- messages ----
fn ser_msg(r: &voicemail_messages::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "voicemail_message_uuid": r.voicemail_message_uuid,
        "voicemail_uuid": r.voicemail_uuid,
        "file_name": r.file_name,
        "message_length": r.message_length,
        "created_epoch": r.created_epoch,
        "read_epoch": r.read_epoch,
        "caller_id_name": r.caller_id_name,
        "caller_id_number": r.caller_id_number,
        "message_status": r.message_status,
        "message_priority": r.message_priority,
        "message_transcription": r.message_transcription,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn list_messages(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_view") {
        return (s, j).into_response();
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
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = voicemail_messages::Entity::find()
        .filter(voicemail_messages::Column::VoicemailUuid.eq(id.clone()))
        .filter(voicemail_messages::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_msg).collect();
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
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
        created_epoch: Set(payload.get("created_epoch").and_then(|v| v.as_i64())),
        read_epoch: Set(payload.get("read_epoch").and_then(|v| v.as_i64())),
        caller_id_name: Set(payload.get("caller_id_name").and_then(|v| v.as_str()).map(str::to_owned)),
        caller_id_number: Set(payload.get("caller_id_number").and_then(|v| v.as_str()).map(str::to_owned)),
        message_status: Set(payload.get("message_status").and_then(|v| v.as_str()).map(str::to_owned)),
        message_priority: Set(payload.get("message_priority").and_then(|v| v.as_str()).map(str::to_owned)),
        message_intro_base64: Set(None),
        message_base64: Set(payload.get("message_base64").and_then(|v| v.as_str()).map(str::to_owned)),
        message_transcription: Set(payload.get("message_transcription").and_then(|v| v.as_str()).map(str::to_owned)),
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_view") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_view") {
        return (s, j).into_response();
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
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = voicemail_options::Entity::find()
        .filter(voicemail_options::Column::VoicemailUuid.eq(id.clone()))
        .filter(voicemail_options::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_opt).collect();
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
        greeting_id: Set(None),
        greeting_voice: Set(payload.greeting_voice.clone()),
        greeting_message: Set(payload.greeting_message.clone()),
        greeting_filename: Set(payload.greeting_filename.clone()),
        greeting_description: Set(payload.greeting_description.clone()),
        greeting_base64: Set(payload.greeting_base64.clone()),
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
    let mut v = serde_json::json!({
        "voicemail_greeting_uuid": r.voicemail_greeting_uuid,
        "voicemail_uuid": r.voicemail_uuid,
        "greeting_name": r.greeting_name,
        "greeting_text": r.greeting_text,
        "greeting_id": r.greeting_id,
        "greeting_voice": r.greeting_voice,
        "greeting_message": r.greeting_message,
        "greeting_filename": r.greeting_filename,
        "greeting_description": r.greeting_description,
        "greeting_base64": r.greeting_base64,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn list_greetings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = voicemail_greetings::Entity::find()
        .filter(voicemail_greetings::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_greeting).collect();
    (
        crate::paged_headers("/api/v1/voicemail_greetings", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_greeting_standalone(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
        greeting_id: Set(payload.get("greeting_id").and_then(|v| v.as_str()).map(str::to_owned)),
        greeting_voice: Set(payload.get("greeting_voice").and_then(|v| v.as_str()).map(str::to_owned)),
        greeting_message: Set(payload.get("greeting_message").and_then(|v| v.as_str()).map(str::to_owned)),
        greeting_filename: Set(payload.get("greeting_filename").and_then(|v| v.as_str()).map(str::to_owned)),
        greeting_description: Set(payload.get("greeting_description").and_then(|v| v.as_str()).map(str::to_owned)),
        greeting_base64: Set(payload.get("greeting_base64").and_then(|v| v.as_str()).map(str::to_owned)),
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_view") {
        return (s, j).into_response();
    }
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
    for key in ["greeting_id", "greeting_voice", "greeting_message", "greeting_filename", "greeting_description", "greeting_base64"] {
        if let Some(v) = payload.get(key).and_then(|v| v.as_str()) {
            match key {
                "greeting_id" => am.greeting_id = Set(Some(v.to_owned())),
                "greeting_voice" => am.greeting_voice = Set(Some(v.to_owned())),
                "greeting_message" => am.greeting_message = Set(Some(v.to_owned())),
                "greeting_filename" => am.greeting_filename = Set(Some(v.to_owned())),
                "greeting_description" => am.greeting_description = Set(Some(v.to_owned())),
                _ => am.greeting_base64 = Set(Some(v.to_owned())),
            }
        }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "voicemail_add") {
        return (s, j).into_response();
    }
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
