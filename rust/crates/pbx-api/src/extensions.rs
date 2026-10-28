//! Extensions router — mirrors `app/extensions/router.py`.
//!
//! SeaORM/postgres backed (`pbx_db::entities::extensions`) with per-request
//! `set_tenant` RLS scoping. Legacy `domain` string from the contract harness
//! is carried in `dial_string` (compat shim); prod scoping uses `domain_id`
//! from verified JWT claims.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extension {
    pub extension: String,
    pub password: String,
    pub domain: String,
}

#[derive(Debug, Deserialize)]
struct ExtensionCreate {
    extension: String,
    password: Option<String>,
    domain: Option<String>,
}

fn validate(ext: &str) -> Result<(), String> {
    pbx_db::extensions::validate_extension(ext)
}

async fn create_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<ExtensionCreate>,
) -> impl IntoResponse {
    let ext = payload.extension.trim().to_owned();
    if let Err(msg) = validate(&ext) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": msg})),
        );
    }
    let domain_str = payload.domain.unwrap_or_else(|| "example.com".to_owned());
    let password = pbx_db::extensions::ensure_password(payload.password.as_deref());
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
    let user_context = pbx_db::extensions::default_user_context(None, &domain_str);
    let am = pbx_db::entities::extensions::ActiveModel {
        extension_uuid: Set(uuid::Uuid::now_v7()),
        domain_id: Set(domain_id),
        extension: Set(ext.clone()),
        number_alias: Set(None),
        password: Set(Some(password.clone())),
        accountcode: Set(None),
        effective_caller_id_name: Set(ext.clone()),
        effective_caller_id_number: Set(None),
        outbound_caller_id_name: Set(None),
        outbound_caller_id_number: Set(None),
        emergency_caller_id_name: Set(None),
        emergency_caller_id_number: Set(None),
        directory_first_name: Set(None),
        directory_last_name: Set(None),
        directory_visible: Set(false),
        directory_exten_visible: Set(false),
        max_registrations: Set(None),
        limit_max: Set(None),
        limit_destination: Set(None),
        missed_call_app: Set(None),
        missed_call_data: Set(None),
        user_context: Set(Some(user_context)),
        toll_allow: Set(None),
        call_timeout: Set(None),
        call_group: Set(None),
        call_screen_enabled: Set(false),
        user_record: Set(None),
        hold_music: Set(None),
        auth_acl: Set(None),
        cidr: Set(None),
        sip_force_contact: Set(None),
        sip_force_expires: Set(None),
        nibble_account: Set(None),
        mwi_account: Set(None),
        sip_bypass_media: Set(None),
        absolute_codec_string: Set(None),
        force_ping: Set(false),
        dial_string: Set(Some(domain_str.clone())),
        do_not_disturb: Set(false),
        forward_all_destination: Set(None),
        forward_all_enabled: Set(false),
        forward_busy_destination: Set(None),
        forward_busy_enabled: Set(false),
        forward_no_answer_destination: Set(None),
        forward_no_answer_enabled: Set(false),
        forward_user_not_registered_destination: Set(None),
        forward_user_not_registered_enabled: Set(false),
        follow_me_uuid: Set(None),
        follow_me_enabled: Set(false),
        extension_language: Set(None),
        extension_dialect: Set(None),
        extension_voice: Set(None),
        extension_type: Set(None),
        enabled: Set(true),
        description: Set(None),
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
    let rec = Extension {
        extension: ext,
        password,
        domain: domain_str,
    };
    (StatusCode::CREATED, Json(serde_json::json!(rec)))
}

async fn list_extensions(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/extensions", page, per_page, 0),
                Json(
                    serde_json::json!({"data": [], "total": 0, "page": page, "per_page": per_page}),
                ),
            );
        }
    };
    let domain_id = crate::routing_db::domain(&claims);
    let rows = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
        .all(&txn)
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let total = rows.len() as u64;
    let items: Vec<Extension> = rows
        .into_iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(|r| Extension {
            extension: r.extension,
            password: r.password.unwrap_or_default(),
            domain: r.dial_string.unwrap_or_else(|| claims.domain_id.clone()),
        })
        .collect();
    (
        crate::paged_headers("/api/v1/extensions", page, per_page, total),
        Json(crate::paged_body(items, total, page, per_page)),
    )
}

/// Lookup used by the FS XML directory handler (SeaORM backed).
/// Exact domain match first, then IP-fallback: any domain for user.
pub async fn find_extension(user: &str, domain: &str) -> Option<Extension> {
    let db = crate::routing_db::conn().await;
    let rows = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::Extension.eq(user))
        .all(db)
        .await
        .unwrap_or_default();
    if rows.is_empty() {
        return None;
    }
    let pick = rows
        .iter()
        .find(|r| r.dial_string.as_deref() == Some(domain))
        .or(rows.first())?;
    Some(Extension {
        extension: pick.extension.clone(),
        password: pick.password.clone().unwrap_or_default(),
        domain: pick
            .dial_string
            .clone()
            .unwrap_or_else(|| domain.to_owned()),
    })
}

pub fn extensions_router() -> Router {
    Router::new()
        .route(
            "/api/v1/extensions",
            post(create_extension).get(list_extensions),
        )
        // Trailing-slash alias (Python router accepts both).
        .route(
            "/api/v1/extensions/",
            post(create_extension).get(list_extensions),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
