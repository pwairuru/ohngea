//! IVR menus router — mirrors `app/ivr_menus/router.py`.
//!
//! Order: models → schemas → handlers → xml.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_add") {
        return (s, j).into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        )
            .into_response();
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
        )
            .into_response();
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
        .into_response()
}

async fn list_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = ivr_menus::Entity::find()
        .filter(ivr_menus::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
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
    (
        crate::paged_headers("/api/v1/ivr_menus", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn ivr_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<ivr_menus::Model> {
    ivr_menus::Entity::find_by_id(id.to_owned())
        .filter(ivr_menus::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = ivr_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    };
    let mut am: ivr_menus::ActiveModel = r.into();
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = Set(v.to_owned());
    }
    if payload.get("extension").is_some() {
        am.extension = Set(payload
            .get("extension")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "ivr_menu_uuid": r.ivr_menu_uuid,
                "name": r.name,
                "extension": r.extension,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    for o in ivr_menu_options::Entity::find()
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(id.clone()))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let oam: ivr_menu_options::ActiveModel = o.into();
        let _ = oam.delete(db).await;
    }
    let Some(r) = ivr_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    };
    let am: ivr_menus::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- options (standalone CRUD under menu; mirrors Python nested routers) ----
fn ser_opt(r: &ivr_menu_options::Model) -> serde_json::Value {
    serde_json::json!({
        "option_uuid": r.option_uuid,
        "ivr_menu_uuid": r.ivr_menu_uuid,
        "digits": r.digits,
        "action": r.action,
        "param": r.param,
    })
}

async fn opt_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    menu: &str,
    oid: &str,
) -> Option<ivr_menu_options::Model> {
    ivr_menu_options::Entity::find_by_id(oid.to_owned())
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(menu.to_owned()))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn list_options(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if ivr_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = ivr_menu_options::Entity::find()
        .filter(ivr_menu_options::Column::IvrMenuUuid.eq(id.clone()))
        .filter(ivr_menu_options::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_opt).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/ivr_menus/{id}/options"),
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if ivr_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "IVR menu not found"})),
        )
            .into_response();
    }
    let digits = payload.get("digits").and_then(|v| v.as_str()).unwrap_or("");
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");
    if digits.trim().is_empty() || action.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "digits/action required"})),
        )
            .into_response();
    }
    let oid = crate::routing_db::new_uuid();
    let am = ivr_menu_options::ActiveModel {
        option_uuid: Set(oid.clone()),
        ivr_menu_uuid: Set(id.clone()),
        domain_id: Set(domain),
        digits: Set(digits.to_owned()),
        action: Set(action.to_owned()),
        param: Set(payload
            .get("param")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        option_order: Set(payload
            .get("option_order")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "option_uuid": oid,
                "ivr_menu_uuid": id,
                "digits": digits,
                "action": action,
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_edit") {
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
    let mut am: ivr_menu_options::ActiveModel = r.into();
    if let Some(v) = payload.get("digits").and_then(|v| v.as_str()) {
        am.digits = Set(v.to_owned());
    }
    if let Some(v) = payload.get("action").and_then(|v| v.as_str()) {
        am.action = Set(v.to_owned());
    }
    if payload.get("param").is_some() {
        am.param = Set(payload
            .get("param")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_delete") {
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
    let am: ivr_menu_options::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
async fn get_ivr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "ivr_menu_view") {
        return (s, j).into_response();
    }
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
        )
            .into_response();
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
        .into_response()
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
        .route(
            "/api/v1/ivr_menus/{id}",
            axum::routing::get(get_ivr)
                .patch(patch_ivr)
                .delete(delete_ivr),
        )
        .route(
            "/api/v1/ivr_menus/{id}/options",
            axum::routing::get(list_options).post(create_option),
        )
        .route(
            "/api/v1/ivr_menus/{id}/options/{oid}",
            axum::routing::patch(patch_option).delete(delete_option),
        )
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
