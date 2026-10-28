//! IVR menus router — mirrors `app/ivr_menus/router.py`.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{ivr_menu_options, ivr_menus};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct OptionIn {
    digits: String,
    action: String,
    #[serde(default)]
    param: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IvrCreate {
    name: String,
    #[serde(default)]
    extension: Option<String>,
    #[serde(default)]
    options: Vec<OptionIn>,
}

#[derive(Debug, Serialize)]
struct OptionOut {
    digits: String,
    action: String,
    param: Option<String>,
}

#[derive(Debug, Serialize)]
struct IvrOut {
    ivr_menu_uuid: String,
    name: String,
    options: Vec<OptionOut>,
}

// ---- handlers ----
async fn create_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<IvrCreate>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = ivr_menus::ActiveModel {
        ivr_menu_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set(payload.name.trim().to_owned()),
        extension: Set(payload.extension),
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
    let mut out_opts = Vec::new();
    for (idx, o) in payload.options.iter().enumerate() {
        let oam = ivr_menu_options::ActiveModel {
            option_uuid: Set(crate::routing_db::new_uuid()),
            ivr_menu_uuid: Set(id.clone()),
            domain_id: Set(domain),
            digits: Set(o.digits.clone()),
            action: Set(o.action.clone()),
            param: Set(o.param.clone()),
            option_order: Set(idx as i32),
        };
        let _ = oam.insert(db).await;
        out_opts.push(OptionOut {
            digits: o.digits.clone(),
            action: o.action.clone(),
            param: o.param.clone(),
        });
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(IvrOut {
            ivr_menu_uuid: id,
            name: payload.name,
            options: out_opts,
        })),
    )
}

async fn list_ivr(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = ivr_menus::Entity::find()
        .filter(ivr_menus::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "ivr_menu_uuid": r.ivr_menu_uuid,
                "name": r.name,
                "extension": r.extension,
                "enabled": r.enabled,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = ivr_menus::Entity::find_by_id(id.clone())
        .filter(ivr_menus::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        );
    };
    let opts = ivr_menu_options::Entity::find()
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(id))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let options: Vec<OptionOut> = opts
        .into_iter()
        .map(|o| OptionOut {
            digits: o.digits,
            action: o.action,
            param: o.param,
        })
        .collect();
    (
        StatusCode::OK,
        Json(serde_json::json!(IvrOut {
            ivr_menu_uuid: r.ivr_menu_uuid,
            name: r.name,
            options,
        })),
    )
}

// ---- xml ----
// IVR dialplan fragment (menu entries → transfer actions).
pub fn build_ivr_dialplan_xml(name: &str, options: &[(String, String, Option<String>)]) -> String {
    let entries = options
        .iter()
        .map(|(digits, action, param)| {
            format!(
                "<entry digits=\"{digits}\" action=\"{action}\" param=\"{}\"/>",
                param.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<menu name=\"{name}\">{entries}</menu>")
}

pub fn ivr_menus_router() -> Router {
    Router::new()
        .route("/api/v1/ivr_menus", post(create_ivr).get(list_ivr))
        .route("/api/v1/ivr_menus/", post(create_ivr).get(list_ivr))
        .route("/api/v1/ivr_menus/{id}", axum::routing::get(get_ivr))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

#[cfg(test)]
mod tests {
    use super::build_ivr_dialplan_xml;

    #[test]
    fn ivr_xml_carries_menu_entries() {
        let xml = build_ivr_dialplan_xml(
            "main",
            &[(
                "1".to_owned(),
                "transfer".to_owned(),
                Some("1001".to_owned()),
            )],
        );
        assert!(xml.contains("main") && xml.contains("digits=\"1\"") && xml.contains("1001"));
    }
}
