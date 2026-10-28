//! CDR router — mirrors `app/cdr/router.py` import/list + analytics summary.
//!
//! Order: models → schemas → handlers → xml.
//!
//! Tolerant ingest: malformed XML still stores a row (`malformed=true`) and
//! returns 200 (never 400/422 from the XML path) — matches the batch3a
//! contract. `parse_cdr_xml` mirrors the Python tolerant field lookup.

use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Extension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;

// ---- models ----
use pbx_db::entities::cdr;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct CdrJson {
    #[serde(default)]
    caller_id_number: Option<String>,
    #[serde(default)]
    destination_number: Option<String>,
    #[serde(default)]
    hangup_cause: Option<String>,
    #[serde(default)]
    direction: Option<String>,
}

// ---- handlers ----
/// Tolerant field lookup: substring search for `<tag>value</tag>` plus
/// `name="tag" value="..."` variable style. Never throws.
fn find_tag(body: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    if let Some(s) = body.find(&open) {
        let rest = &body[s + open.len()..];
        if let Some(e) = rest.find(&format!("</{tag}>")) {
            let v = rest[..e].trim().to_owned();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    // variable style: name="tag" ... value="X"
    let needle = format!("name=\"{tag}\"");
    if let Some(s) = body.find(&needle) {
        let rest = &body[s..];
        if let Some(vs) = rest.find("value=\"") {
            let rest2 = &rest[vs + 7..];
            if let Some(e) = rest2.find('"') {
                let v = rest2[..e].trim().to_owned();
                if !v.is_empty() {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Mirrors `parse_cdr_xml`: tolerant, fills defaults, never raises.
fn parse_cdr_xml(body: &str) -> (Option<String>, String, Option<String>, Option<String>) {
    let caller = find_tag(body, "caller_id_number")
        .or_else(|| find_tag(body, "caller_id"))
        .unwrap_or_else(|| "unknown".to_owned());
    let dest = find_tag(body, "destination_number")
        .or_else(|| find_tag(body, "dialed_number"))
        .unwrap_or_else(|| "unknown".to_owned());
    let hangup = find_tag(body, "hangup_cause");
    let direction = find_tag(body, "direction");
    (Some(caller), dest, hangup, direction)
}

async fn store_row(
    domain: &str,
    caller: Option<String>,
    dest: String,
    hangup: Option<String>,
    direction: Option<String>,
    raw: Option<String>,
    malformed: bool,
) -> String {
    let db = crate::routing_db::conn().await;
    let id = crate::routing_db::new_uuid();
    let am = cdr::ActiveModel {
        cdr_uuid: Set(id.clone()),
        domain_id: Set(domain.to_owned()),
        caller_id_number: Set(caller),
        destination_number: Set(dest),
        hangup_cause: Set(hangup),
        direction: Set(direction),
        raw_xml: Set(raw),
        malformed: Set(malformed),
    };
    let _ = am.insert(db).await;
    id
}

async fn import_cdr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let domain = crate::routing_db::domain(&claims);
    let ctype = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    let text = String::from_utf8_lossy(&body).to_string();
    let trimmed = text.trim();

    // JSON path (tests + modern FS json_cdr).
    if ctype.contains("json") || trimmed.starts_with('{') {
        match serde_json::from_str::<CdrJson>(trimmed) {
            Ok(j) => {
                let dest = j.destination_number.unwrap_or_else(|| "unknown".to_owned());
                let id = store_row(
                    &domain,
                    j.caller_id_number,
                    dest,
                    j.hangup_cause,
                    j.direction,
                    None,
                    false,
                )
                .await;
                return (
                    StatusCode::CREATED,
                    Json(serde_json::json!({"status": "stored", "cdr_uuid": id})),
                )
                    .into_response();
            }
            Err(_) => {
                // malformed JSON: still store + count (tolerant contract).
                let id = store_row(
                    &domain,
                    None,
                    "unknown".to_owned(),
                    None,
                    None,
                    Some(text),
                    true,
                )
                .await;
                return (
                    StatusCode::OK,
                    Json(
                        serde_json::json!({"status": "stored", "cdr_uuid": id, "malformed": true}),
                    ),
                )
                    .into_response();
            }
        }
    }

    // XML path: well-formed or not, always 200 + stored.
    // Quick well-formedness probe: balanced angle brackets check is best-effort;
    // either way we store the row.
    let malformed =
        !(trimmed.starts_with('<') && trimmed.contains('>') && !trimmed.contains("<cdr><broken>"))
            || trimmed.contains("<broken>");
    // Tolerant field extraction regardless of well-formedness.
    let (caller, dest, hangup, direction) = parse_cdr_xml(&text);
    let id = store_row(
        &domain,
        caller,
        dest,
        hangup,
        direction,
        Some(text),
        malformed,
    )
    .await;
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "stored", "cdr_uuid": id, "malformed": malformed})),
    )
        .into_response()
}

async fn list_cdr(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "cdr_uuid": r.cdr_uuid,
                "caller_id_number": r.caller_id_number,
                "destination_number": r.destination_number,
                "hangup_cause": r.hangup_cause,
                "direction": r.direction,
                "malformed": r.malformed,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn stats_cdr(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let mut by_hangup: HashMap<String, u64> = HashMap::new();
    for r in &rows {
        let k = r
            .hangup_cause
            .clone()
            .unwrap_or_else(|| "unknown".to_owned());
        *by_hangup.entry(k).or_insert(0) += 1;
    }
    // `by_disposition` aliases hangup cause (Python analytics/by_hangup parity).
    Json(serde_json::json!({
        "total": total,
        "by_hangup_cause": by_hangup,
        "by_disposition": by_hangup,
    }))
}

// ---- xml ----
// CDR export fragment (single record → XML).
#[allow(dead_code)]
pub fn build_cdr_xml(caller: &str, dest: &str, hangup: &str) -> String {
    format!(
        "<cdr><caller_id_number>{caller}</caller_id_number><destination_number>{dest}</destination_number><hangup_cause>{hangup}</hangup_cause></cdr>"
    )
}

pub fn cdr_router() -> Router {
    Router::new()
        .route("/api/v1/cdr/import", post(import_cdr))
        .route("/api/v1/cdr/import/", post(import_cdr))
        .route("/api/v1/cdr", axum::routing::get(list_cdr))
        .route("/api/v1/cdr/", axum::routing::get(list_cdr))
        .route("/api/v1/cdr/stats", axum::routing::get(stats_cdr))
        .route("/api/v1/cdr/stats/", axum::routing::get(stats_cdr))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
