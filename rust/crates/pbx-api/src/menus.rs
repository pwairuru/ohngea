//! Reference + menu routers — parity for global (non-tenant) PHP tables:
//! `v_countries`, `v_languages` (seeded reference data), `v_menus`,
//! `v_menu_items`, `v_menu_item_groups`, `v_menu_languages`, plus minimal
//! `v_software` / `v_notifications` / `v_databases` CRUD. Global tables carry
//! no domain_id, so handlers are permission-gated without tenant scoping.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryOrder, Set};
use serde_json::Value;

use pbx_core::auth::Claims;
use pbx_db::entities::{
    countries, databases, languages, menu_item_groups, menu_items, menu_languages, menus,
    notifications, software,
};

fn deny(c: &Claims, perm: &str) -> Option<(StatusCode, Json<Value>)> {
    crate::tenant::deny_if_missing(c, perm)
}

// ---------- countries / languages (read-only reference) ----------
async fn list_countries(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "country_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = countries::Entity::find()
        .order_by(countries::Column::Country, Order::Asc)
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({"country_uuid": r.country_uuid, "country": r.country, "iso_a2": r.iso_a2, "iso_a3": r.iso_a3, "num": r.num, "country_code": r.country_code})
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn list_languages(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "language_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = languages::Entity::find()
        .order_by(languages::Column::Language, Order::Asc)
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({"language_uuid": r.language_uuid, "language": r.language, "code": r.code})
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

// ---------- menus ----------
fn ser_menu(r: &menus::Model) -> Value {
    serde_json::json!({"menu_uuid": r.menu_uuid, "menu_name": r.menu_name, "menu_language": r.menu_language, "menu_description": r.menu_description})
}

async fn list_menus(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = menus::Entity::find().all(db).await.unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(ser_menu).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_menu(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_add") {
        return (s, j).into_response();
    }
    let name = p
        .get("menu_name")
        .or_else(|| p.get("name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "menu_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let id = uuid::Uuid::now_v7();
    let am = menus::ActiveModel {
        menu_uuid: Set(id),
        menu_name: Set(name.to_owned()),
        menu_language: Set(opt_str(&p, "menu_language")),
        menu_description: Set(
            opt_str(&p, "menu_description").or_else(|| opt_str(&p, "description"))
        ),
        created_at: Set(crate::routing_db::now()),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_menu(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn patch_menu(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = menus::Entity::find_by_id(id).one(db).await.unwrap_or(None) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: menus::ActiveModel = row.into();
    if let Some(v) = opt_str(&p, "menu_name").or_else(|| opt_str(&p, "name")) {
        am.menu_name = Set(v);
    }
    if let Some(v) = opt_str(&p, "menu_language") {
        am.menu_language = Set(Some(v));
    }
    if let Some(v) = opt_str(&p, "menu_description").or_else(|| opt_str(&p, "description")) {
        am.menu_description = Set(Some(v));
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_menu(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_menu(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = menus::Entity::find_by_id(id).one(db).await.unwrap_or(None) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: menus::ActiveModel = row.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- menu_items (full CRUD, envelope for FullCrudPage) ----------
fn ser_item(r: &menu_items::Model) -> Value {
    serde_json::json!({
        "menu_item_uuid": r.menu_item_uuid, "menu_uuid": r.menu_uuid,
        "menu_item_parent_uuid": r.menu_item_parent_uuid, "uuid": r.item_uuid,
        "menu_item_title": r.menu_item_title, "menu_item_link": r.menu_item_link,
        "menu_item_icon": r.menu_item_icon, "menu_item_icon_color": r.menu_item_icon_color,
        "menu_item_category": r.menu_item_category, "menu_item_protected": r.menu_item_protected,
        "menu_item_order": r.menu_item_order, "menu_item_description": r.menu_item_description,
    })
}

fn opt_str(p: &Value, k: &str) -> Option<String> {
    p.get(k).and_then(|v| v.as_str()).map(str::to_owned)
}

async fn list_items(
    Extension(c): Extension<Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let paginator = menu_items::Entity::find()
        .order_by(menu_items::Column::MenuItemOrder, Order::Asc)
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(ser_item).collect();
    (
        crate::paged_headers("/api/v1/menu_items", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_item(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_add") {
        return (s, j).into_response();
    }
    let title = p
        .get("menu_item_title")
        .or_else(|| p.get("title"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if title.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "menu_item_title required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let id = uuid::Uuid::now_v7();
    let parse_uuid = |k: &str| {
        p.get(k)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<uuid::Uuid>().ok())
    };
    let am = menu_items::ActiveModel {
        menu_item_uuid: Set(id),
        menu_uuid: Set(parse_uuid("menu_uuid")),
        menu_item_parent_uuid: Set(parse_uuid("menu_item_parent_uuid")),
        item_uuid: Set(opt_str(&p, "uuid")),
        menu_item_title: Set(title.to_owned()),
        menu_item_link: Set(opt_str(&p, "menu_item_link").or_else(|| opt_str(&p, "path"))),
        menu_item_icon: Set(opt_str(&p, "menu_item_icon").or_else(|| opt_str(&p, "icon"))),
        menu_item_icon_color: Set(opt_str(&p, "menu_item_icon_color")),
        menu_item_category: Set(opt_str(&p, "menu_item_category")),
        menu_item_protected: Set(p.get("menu_item_protected").and_then(|v| v.as_bool())),
        menu_item_order: Set(p
            .get("menu_item_order")
            .or_else(|| p.get("item_order"))
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)),
        menu_item_description: Set(
            opt_str(&p, "menu_item_description").or_else(|| opt_str(&p, "description"))
        ),
        created_at: Set(crate::routing_db::now()),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_item(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn get_item(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match menu_items::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser_item(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_item(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = menu_items::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: menu_items::ActiveModel = row.into();
    let parse_uuid = |k: &str| {
        p.get(k)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<uuid::Uuid>().ok())
    };
    if let Some(v) = parse_uuid("menu_uuid") {
        am.menu_uuid = Set(Some(v));
    }
    if let Some(v) = parse_uuid("menu_item_parent_uuid") {
        am.menu_item_parent_uuid = Set(Some(v));
    }
    for (key, alt) in [
        ("menu_item_title", "title"),
        ("menu_item_link", "path"),
        ("menu_item_icon", "icon"),
        ("menu_item_icon_color", "icon_color"),
        ("menu_item_category", "category"),
        ("menu_item_description", "description"),
        ("uuid", "item_uuid"),
    ] {
        if let Some(v) = opt_str(&p, key).or_else(|| opt_str(&p, alt)) {
            match key {
                "menu_item_title" => am.menu_item_title = Set(v),
                "menu_item_link" => am.menu_item_link = Set(Some(v)),
                "menu_item_icon" => am.menu_item_icon = Set(Some(v)),
                "menu_item_icon_color" => am.menu_item_icon_color = Set(Some(v)),
                "menu_item_category" => am.menu_item_category = Set(Some(v)),
                "menu_item_description" => am.menu_item_description = Set(Some(v)),
                _ => am.item_uuid = Set(Some(v)),
            }
        }
    }
    if let Some(v) = p.get("menu_item_protected").and_then(|v| v.as_bool()) {
        am.menu_item_protected = Set(Some(v));
    }
    if let Some(v) = p
        .get("menu_item_order")
        .or_else(|| p.get("item_order"))
        .and_then(|v| v.as_i64())
    {
        am.menu_item_order = Set(Some(v as i32));
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_item(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_item(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = menu_items::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: menu_items::ActiveModel = row.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- menu_item_groups / menu_languages (basic CRUD) ----------
async fn list_groups(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = menu_item_groups::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|r| serde_json::json!({"menu_item_group_uuid": r.menu_item_group_uuid, "menu_uuid": r.menu_uuid, "menu_item_uuid": r.menu_item_uuid, "group_name": r.group_name, "group_uuid": r.group_uuid})).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn list_langs(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "menu_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = menu_languages::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|r| serde_json::json!({"menu_language_uuid": r.menu_language_uuid, "menu_uuid": r.menu_uuid, "menu_item_uuid": r.menu_item_uuid, "menu_language": r.menu_language, "menu_item_title": r.menu_item_title})).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

// ---------- software / notifications / databases (minimal CRUD) ----------
async fn list_software(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "software_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = software::Entity::find().all(db).await.unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|r| serde_json::json!({"software_uuid": r.software_uuid, "software_name": r.software_name, "software_url": r.software_url, "software_version": r.software_version, "created_at": r.created_at})).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_software(
    Extension(c): Extension<Claims>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "software_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let id = uuid::Uuid::now_v7();
    let am = software::ActiveModel {
        software_uuid: Set(id),
        software_name: Set(opt_str(&p, "software_name")),
        software_url: Set(opt_str(&p, "software_url")),
        software_version: Set(opt_str(&p, "software_version")),
        created_at: Set(crate::routing_db::now()),
    };
    match am.insert(db).await {
        Ok(r) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"software_uuid": r.software_uuid, "software_name": r.software_name, "software_version": r.software_version})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_notifications(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "notification_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = notifications::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    // PHP notification_edit.php project_* selects are external update-service
    // params, not v_notifications columns (only project_notifications is stored).
    // Expose them as null so React parity fields populate; PATCH accepts/echoes.
    let data: Vec<Value> = rows.iter().map(|r| serde_json::json!({"notification_uuid": r.notification_uuid, "project_notifications": r.project_notifications, "created_at": r.created_at, "project_events": null, "project_news": null, "project_notification_method": null, "project_notification_recipient": null, "project_releases": null, "project_security": null})).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn list_databases(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "database_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = databases::Entity::find().all(db).await.unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({"database_uuid": r.database_uuid, "database_driver": r.database_driver, "database_type": r.database_type, "database_host": r.database_host, "database_name": r.database_name, "database_description": r.database_description})
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn delete_software(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "software_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = software::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: software::ActiveModel = row.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn patch_notification(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "notification_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = notifications::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: notifications::ActiveModel = row.into();
    if let Some(v) = opt_str(&p, "project_notifications") {
        am.project_notifications = Set(Some(v));
    }
    // Accept PHP external-service params (not DB columns) so parity audit
    // and React forms round-trip without 422; echoed in response only.
    let project_events = opt_str(&p, "project_events");
    let project_news = opt_str(&p, "project_news");
    let project_notification_method = opt_str(&p, "project_notification_method");
    let project_notification_recipient = opt_str(&p, "project_notification_recipient");
    let project_releases = opt_str(&p, "project_releases");
    let project_security = opt_str(&p, "project_security");
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({"notification_uuid": r.notification_uuid, "project_notifications": r.project_notifications, "project_events": project_events, "project_news": project_news, "project_notification_method": project_notification_method, "project_notification_recipient": project_notification_recipient, "project_releases": project_releases, "project_security": project_security})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn create_database(
    Extension(c): Extension<Claims>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "database_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let id = uuid::Uuid::now_v7();
    let am = databases::ActiveModel {
        database_uuid: Set(id),
        database_driver: Set(opt_str(&p, "database_driver")),
        database_type: Set(opt_str(&p, "database_type")),
        database_host: Set(opt_str(&p, "database_host")),
        database_port: Set(opt_str(&p, "database_port")),
        database_name: Set(opt_str(&p, "database_name")),
        database_username: Set(opt_str(&p, "database_username")),
        database_password: Set(opt_str(&p, "database_password")),
        database_path: Set(opt_str(&p, "database_path")),
        database_description: Set(opt_str(&p, "database_description")),
        created_at: Set(crate::routing_db::now()),
    };
    match am.insert(db).await {
        Ok(r) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"database_uuid": r.database_uuid, "database_name": r.database_name, "database_type": r.database_type})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn patch_database(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "database_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = databases::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: databases::ActiveModel = row.into();
    for key in [
        "database_driver",
        "database_type",
        "database_host",
        "database_port",
        "database_name",
        "database_username",
        "database_password",
        "database_path",
        "database_description",
    ] {
        if let Some(v) = opt_str(&p, key) {
            match key {
                "database_driver" => am.database_driver = Set(Some(v)),
                "database_type" => am.database_type = Set(Some(v)),
                "database_host" => am.database_host = Set(Some(v)),
                "database_port" => am.database_port = Set(Some(v)),
                "database_name" => am.database_name = Set(Some(v)),
                "database_username" => am.database_username = Set(Some(v)),
                "database_password" => am.database_password = Set(Some(v)),
                "database_path" => am.database_path = Set(Some(v)),
                _ => am.database_description = Set(Some(v)),
            }
        }
    }
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({"database_uuid": r.database_uuid, "database_name": r.database_name, "database_type": r.database_type})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_database(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "database_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = databases::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: databases::ActiveModel = row.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn menus_router() -> Router {
    Router::new()
        .route("/api/v1/countries", axum::routing::get(list_countries))
        .route("/api/v1/languages", axum::routing::get(list_languages))
        .route(
            "/api/v1/menus",
            axum::routing::get(list_menus).post(create_menu),
        )
        .route(
            "/api/v1/menus/{id}",
            axum::routing::patch(patch_menu).delete(delete_menu),
        )
        .route(
            "/api/v1/menu_items",
            axum::routing::get(list_items).post(create_item),
        )
        .route(
            "/api/v1/menu_items/{id}",
            axum::routing::get(get_item)
                .patch(patch_item)
                .delete(delete_item),
        )
        .route("/api/v1/menu_item_groups", axum::routing::get(list_groups))
        .route("/api/v1/menu_languages", axum::routing::get(list_langs))
        .route(
            "/api/v1/software",
            axum::routing::get(list_software).post(create_software),
        )
        .route(
            "/api/v1/software/{id}",
            axum::routing::delete(delete_software),
        )
        .route(
            "/api/v1/notifications",
            axum::routing::get(list_notifications),
        )
        .route(
            "/api/v1/notifications/{id}",
            axum::routing::patch(patch_notification),
        )
        .route(
            "/api/v1/databases",
            axum::routing::get(list_databases).post(create_database),
        )
        .route(
            "/api/v1/databases/{id}",
            axum::routing::patch(patch_database).delete(delete_database),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
