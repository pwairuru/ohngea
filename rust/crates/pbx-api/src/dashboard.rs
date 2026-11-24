//! Dashboards + widgets router — mirrors `core/dashboard` PHP CRUD.
//! (`v_dashboards`, `v_dashboard_widgets`).
//!
//! Order: models → schemas → handlers.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde_json::Value;

use pbx_core::auth::Claims;
use pbx_db::entities::{dashboard_widget_groups, dashboard_widgets, dashboards};

fn deny(c: &Claims, perm: &str) -> Option<(StatusCode, Json<Value>)> {
    crate::tenant::deny_if_missing(c, perm)
}

fn s(p: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = p.get(*k) {
            if let Some(s) = v.as_str() {
                let t = s.trim();
                if !t.is_empty() {
                    return Some(t.to_owned());
                }
            }
        }
    }
    None
}

fn b(p: &Value, keys: &[&str], dflt: bool) -> bool {
    for k in keys {
        if let Some(v) = p.get(*k) {
            if let Some(b) = v.as_bool() {
                return b;
            }
        }
    }
    dflt
}

fn i(p: &Value, keys: &[&str]) -> Option<i32> {
    for k in keys {
        if let Some(v) = p.get(*k) {
            if let Some(n) = v.as_i64() {
                return Some(n as i32);
            }
        }
    }
    None
}

// ---------- dashboards ----------
fn ser_dashboard(r: &dashboards::Model) -> Value {
    serde_json::json!({
        "dashboard_uuid": r.dashboard_uuid,
        "dashboard_name": r.dashboard_name,
        "dashboard_enabled": r.dashboard_enabled,
        "dashboard_description": r.dashboard_description,
    })
}

async fn create_dashboard(
    Extension(c): Extension<Claims>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "dashboard_add") {
        return (s, j).into_response();
    }
    let name = s(&p, &["dashboard_name", "name"]).unwrap_or_default();
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "dashboard_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &c).await;
    let now = crate::routing_db::now();
    let id = uuid::Uuid::now_v7();
    let am = dashboards::ActiveModel {
        dashboard_uuid: Set(id),
        domain_id: Set(Some(domain)),
        dashboard_name: Set(name),
        dashboard_enabled: Set(b(&p, &["dashboard_enabled", "enabled"], true)),
        dashboard_description: Set(s(&p, &["dashboard_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_dashboard(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_dashboards(
    Extension(c): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "dashboard_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&c, show_q.show.as_deref(), "dashboard_all");
    let txn = match crate::routing_db::tenant_txn(db, &c).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::tenant_domain(db, &c).await;
    let mut sel = dashboards::Entity::find();
    if !show_all {
        // PHP `dashboard_domain` parity: holders also see global (NULL)
        // rows; everyone else sees only their own tenant's rows.
        if crate::tenant::sees_domain_nulls(&c, "dashboard_domain") {
            sel = sel.filter(
                sea_orm::Condition::any()
                    .add(dashboards::Column::DomainId.eq(Some(domain)))
                    .add(dashboards::Column::DomainId.is_null()),
            );
        } else {
            sel = sel.filter(dashboards::Column::DomainId.eq(Some(domain)));
        }
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(ser_dashboard).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

// ---------- widgets ----------
fn ser_widget(r: &dashboard_widgets::Model) -> Value {
    serde_json::json!({
        "dashboard_widget_uuid": r.dashboard_widget_uuid,
        "dashboard_uuid": r.dashboard_uuid,
        "widget_name": r.widget_name,
        "widget_path": r.widget_path,
        "widget_icon": r.widget_icon,
        "widget_icon_color": r.widget_icon_color,
        "widget_url": r.widget_url,
        "widget_target": r.widget_target,
        "widget_width": r.widget_width,
        "widget_height": r.widget_height,
        "widget_content": r.widget_content,
        "widget_content_text_align": r.widget_content_text_align,
        "widget_content_details": r.widget_content_details,
        "widget_chart_type": r.widget_chart_type,
        "widget_label_enabled": r.widget_label_enabled,
        "widget_label_text_color": r.widget_label_text_color,
        "widget_label_text_color_hover": r.widget_label_text_color_hover,
        "widget_label_background_color": r.widget_label_background_color,
        "widget_label_background_color_hover": r.widget_label_background_color_hover,
        "widget_number_text_color": r.widget_number_text_color,
        "widget_number_text_color_hover": r.widget_number_text_color_hover,
        "widget_number_background_color": r.widget_number_background_color,
        "widget_background_gradient_style": r.widget_background_gradient_style,
        "widget_background_gradient_angle": r.widget_background_gradient_angle,
        "widget_column_span": r.widget_column_span,
        "widget_row_span": r.widget_row_span,
        "widget_details_state": r.widget_details_state,
        "widget_order": r.widget_order,
        "widget_enabled": r.widget_enabled,
        "widget_description": r.widget_description,
    })
}

async fn create_widget(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "dashboard_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &c).await;
    // parent must exist (tenant-scoped when domain set)
    let parent = dashboards::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None);
    if parent.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "dashboard not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let wid = uuid::Uuid::now_v7();
    let am = dashboard_widgets::ActiveModel {
        dashboard_widget_uuid: Set(wid),
        domain_id: Set(Some(domain)),
        dashboard_uuid: Set(Some(id)),
        widget_name: Set(s(&p, &["widget_name", "name"])),
        widget_path: Set(s(&p, &["widget_path", "path"])),
        widget_icon: Set(s(&p, &["widget_icon", "icon"])),
        widget_icon_color: Set(s(&p, &["widget_icon_color"])),
        widget_url: Set(s(&p, &["widget_url", "url"])),
        widget_target: Set(s(&p, &["widget_target"])),
        widget_width: Set(s(&p, &["widget_width"])),
        widget_height: Set(s(&p, &["widget_height"])),
        widget_content: Set(s(&p, &["widget_content"])),
        widget_content_text_align: Set(s(&p, &["widget_content_text_align"])),
        widget_content_details: Set(s(&p, &["widget_content_details"])),
        widget_chart_type: Set(s(&p, &["widget_chart_type"])),
        widget_label_enabled: Set(b(&p, &["widget_label_enabled"], false)),
        widget_label_text_color: Set(s(&p, &["widget_label_text_color"])),
        widget_label_text_color_hover: Set(s(&p, &["widget_label_text_color_hover"])),
        widget_label_background_color: Set(s(&p, &["widget_label_background_color"])),
        widget_label_background_color_hover: Set(s(&p, &["widget_label_background_color_hover"])),
        widget_number_text_color: Set(s(&p, &["widget_number_text_color"])),
        widget_number_text_color_hover: Set(s(&p, &["widget_number_text_color_hover"])),
        widget_number_background_color: Set(s(&p, &["widget_number_background_color"])),
        widget_background_gradient_style: Set(s(&p, &["widget_background_gradient_style"])),
        widget_background_gradient_angle: Set(s(&p, &["widget_background_gradient_angle"])),
        widget_column_span: Set(i(&p, &["widget_column_span"])),
        widget_row_span: Set(i(&p, &["widget_row_span"])),
        widget_details_state: Set(s(&p, &["widget_details_state"])),
        widget_order: Set(i(&p, &["widget_order"])),
        widget_enabled: Set(b(&p, &["widget_enabled", "enabled"], true)),
        widget_description: Set(s(&p, &["widget_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_widget(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_widgets(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "dashboard_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = dashboard_widgets::Entity::find()
        .filter(dashboard_widgets::Column::DashboardUuid.eq(Some(id)))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(ser_widget).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

fn ser_widget_group(r: &pbx_db::entities::dashboard_widget_groups::Model) -> Value {
    serde_json::json!({
        "dashboard_widget_group_uuid": r.dashboard_widget_group_uuid,
        "dashboard_uuid": r.dashboard_uuid,
        "dashboard_widget_uuid": r.dashboard_widget_uuid,
        "group_uuid": r.group_uuid,
    })
}

/// Parent widget visible to the caller? Tenant-owned always; global
/// (NULL-domain) widgets need `dashboard_domain` (dashboard list parity).
async fn widget_visible(
    db: &sea_orm::DatabaseConnection,
    c: &Claims,
    wid: &uuid::Uuid,
    did: &uuid::Uuid,
) -> bool {
    use sea_orm::EntityTrait;
    let domain = crate::routing_db::tenant_domain(db, c).await;
    match pbx_db::entities::dashboard_widgets::Entity::find_by_id(*wid)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(w)
            if w.dashboard_uuid == Some(*did)
                && (w.domain_id == Some(domain)
                    || (w.domain_id.is_none()
                        && crate::tenant::sees_domain_nulls(c, "dashboard_domain"))) =>
        {
            true
        }
        _ => false,
    }
}

async fn list_widget_groups(
    Extension(c): Extension<Claims>,
    axum::extract::Path((id, wid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "dashboard_widget_group_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    if !widget_visible(db, &c, &wid, &id).await {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let rows = pbx_db::entities::dashboard_widget_groups::Entity::find()
        .filter(
            pbx_db::entities::dashboard_widget_groups::Column::DashboardWidgetUuid.eq(Some(wid)),
        )
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(ser_widget_group).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn assign_widget_group(
    Extension(c): Extension<Claims>,
    axum::extract::Path((id, wid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "dashboard_widget_group_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    if !widget_visible(db, &c, &wid, &id).await {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let Some(g) = payload
        .get("group_uuid")
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<uuid::Uuid>().ok())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "group_uuid must be a UUID"})),
        )
            .into_response();
    };
    let am = pbx_db::entities::dashboard_widget_groups::ActiveModel {
        dashboard_widget_group_uuid: Set(uuid::Uuid::now_v7()),
        dashboard_uuid: Set(Some(id)),
        dashboard_widget_uuid: Set(Some(wid)),
        group_uuid: Set(Some(g)),
        created_at: Set(crate::routing_db::now()),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_widget_group(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn unassign_widget_group(
    Extension(c): Extension<Claims>,
    axum::extract::Path((id, wid, gid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "dashboard_widget_group_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    if !widget_visible(db, &c, &wid, &id).await {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let Some(row) = pbx_db::entities::dashboard_widget_groups::Entity::find_by_id(gid)
        .one(db)
        .await
        .unwrap_or(None)
        .filter(|r| r.dashboard_widget_uuid == Some(wid))
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: pbx_db::entities::dashboard_widget_groups::ActiveModel = row.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn dashboard_router() -> Router {
    Router::new()
        .route(
            "/api/v1/dashboards",
            axum::routing::post(create_dashboard).get(list_dashboards),
        )
        .route(
            "/api/v1/dashboards/{id}/widgets",
            axum::routing::post(create_widget).get(list_widgets),
        )
        .route(
            "/api/v1/dashboards/{id}/widgets/{wid}/groups",
            axum::routing::post(assign_widget_group).get(list_widget_groups),
        )
        .route(
            "/api/v1/dashboards/{id}/widgets/{wid}/groups/{gid}",
            axum::routing::delete(unassign_widget_group),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
