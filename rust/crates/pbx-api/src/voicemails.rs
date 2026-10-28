//! Voicemails router — mirrors `app/voicemails/router.py`.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{voicemail_greetings, voicemails};

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

async fn create_vm(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<VoicemailCreate>,
) -> impl IntoResponse {
    if !valid_mailbox(&payload.voicemail_id) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "voicemail_id must be 2-7 digits"})),
        );
    }
    if payload.voicemail_password.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "voicemail_password required"})),
        );
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
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(VoicemailOut {
            voicemail_uuid: id,
            voicemail_id: payload.voicemail_id,
        })),
    )
}

async fn list_vm(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = voicemails::Entity::find()
        .filter(voicemails::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "voicemail_uuid": r.voicemail_uuid,
                "voicemail_id": r.voicemail_id,
                "enabled": r.enabled,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn create_greeting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<GreetingCreate>,
) -> impl IntoResponse {
    if payload.greeting_name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "greeting_name required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let vm = voicemails::Entity::find_by_id(id.clone())
        .filter(voicemails::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    if vm.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "voicemail not found"})),
        );
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
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "voicemail_greeting_uuid": gid,
            "voicemail_uuid": id,
            "greeting_name": payload.greeting_name,
        })),
    )
}

// ---- xml ----
// Voicemail directory fragment (mailbox auth for FS `directory` section).
pub fn build_voicemail_directory_xml(mailbox: &str, password: &str, domain: &str) -> String {
    format!(
        "<user id=\"{mailbox}@{domain}\"><params><param name=\"password\" value=\"{password}\"/></params></user>"
    )
}

pub fn voicemails_router() -> Router {
    Router::new()
        .route("/api/v1/voicemails", post(create_vm).get(list_vm))
        .route("/api/v1/voicemails/", post(create_vm).get(list_vm))
        .route("/api/v1/voicemails/{id}/greetings", post(create_greeting))
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
