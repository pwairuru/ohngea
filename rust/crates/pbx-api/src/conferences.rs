//! Conferences router — mirrors `app/conferences/router.py`.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::conferences;

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

async fn create_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<ConferenceCreate>,
) -> impl IntoResponse {
    if payload.conference_name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "conference_name required"})),
        );
    }
    if !valid_extension(&payload.conference_extension) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "conference_extension must be 2-7 digits"})),
        );
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
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(ConferenceOut {
            conference_uuid: id,
            conference_name: payload.conference_name,
            conference_extension: payload.conference_extension,
        })),
    )
}

async fn list_conf(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = conferences::Entity::find()
        .filter(conferences::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
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

// ---- xml ----
// Conference dialplan fragment (bridge into the room).
pub fn build_conference_dialplan_xml(name: &str, extension: &str) -> String {
    format!(
        "<extension name=\"{name}\"><condition field=\"destination_number\" expression=\"^{extension}$\"><action application=\"conference\" data=\"{name}@default\"/></condition></extension>"
    )
}

pub fn conferences_router() -> Router {
    Router::new()
        .route("/api/v1/conferences", post(create_conf).get(list_conf))
        .route("/api/v1/conferences/", post(create_conf).get(list_conf))
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
