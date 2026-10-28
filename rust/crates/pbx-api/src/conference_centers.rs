//! Conference centers router — mirrors `app/conference_centers/router.py`.
//!
//! Order: models → schemas → handlers.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::{
    conference_centers, conference_rooms, conference_session_details, conference_sessions,
};

// ---- handlers ----
fn ser_center(r: &conference_centers::Model) -> serde_json::Value {
    serde_json::json!({
        "conference_center_uuid": r.conference_center_uuid,
        "conference_center_name": r.conference_center_name,
        "conference_center_extension": r.conference_center_extension,
        "conference_center_description": r.conference_center_description,
        "conference_center_enabled": r.conference_center_enabled,
    })
}

fn ser_room(r: &conference_rooms::Model) -> serde_json::Value {
    serde_json::json!({
        "conference_room_uuid": r.conference_room_uuid,
        "conference_center_uuid": r.conference_center_uuid,
        "conference_room_name": r.conference_room_name,
        "max_members": r.max_members,
        "enabled": r.enabled,
    })
}

fn ser_session(r: &conference_sessions::Model) -> serde_json::Value {
    serde_json::json!({
        "conference_session_uuid": r.conference_session_uuid,
        "conference_center_uuid": r.conference_center_uuid,
        "conference_room_uuid": r.conference_room_uuid,
        "profile": r.profile,
    })
}

fn ser_detail(r: &conference_session_details::Model) -> serde_json::Value {
    serde_json::json!({
        "conference_session_detail_uuid": r.conference_session_detail_uuid,
        "conference_session_uuid": r.conference_session_uuid,
        "username": r.username,
    })
}

async fn center_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<conference_centers::Model> {
    conference_centers::Entity::find_by_id(id.to_owned())
        .filter(conference_centers::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn create_center(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Some(name) = payload
        .get("conference_center_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "conference_center_name required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = conference_centers::ActiveModel {
        conference_center_uuid: Set(id.clone()),
        domain_id: Set(domain),
        conference_center_name: Set(name.to_owned()),
        conference_center_extension: Set(payload
            .get("conference_center_extension")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        conference_center_description: Set(payload
            .get("conference_center_description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        conference_center_enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "conference_center_uuid": id,
                "conference_center_name": name,
                "conference_center_extension": payload.get("conference_center_extension"),
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

async fn list_centers(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = conference_centers::Entity::find()
        .filter(conference_centers::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_center)
        .collect();
    (
        crate::paged_headers("/api/v1/conference_centers", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn get_center(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match center_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_center(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference center not found"})),
        )
            .into_response(),
    }
}

async fn patch_center(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = center_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference center not found"})),
        )
            .into_response();
    };
    let mut am: conference_centers::ActiveModel = r.into();
    if let Some(v) = payload
        .get("conference_center_name")
        .and_then(|v| v.as_str())
    {
        am.conference_center_name = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("conference_center_extension")
        .and_then(|v| v.as_str())
    {
        am.conference_center_extension = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload
        .get("conference_center_description")
        .and_then(|v| v.as_str())
    {
        am.conference_center_description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload
        .get("conference_center_enabled")
        .and_then(|v| v.as_bool())
    {
        am.conference_center_enabled = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_center(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_center(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    for room in conference_rooms::Entity::find()
        .filter(conference_rooms::Column::ConferenceCenterUuid.eq(id.clone()))
        .filter(conference_rooms::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let ram: conference_rooms::ActiveModel = room.into();
        let _ = ram.delete(db).await;
    }
    for sess in conference_sessions::Entity::find()
        .filter(conference_sessions::Column::ConferenceCenterUuid.eq(id.clone()))
        .filter(conference_sessions::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let sid = sess.conference_session_uuid.clone();
        for d in conference_session_details::Entity::find()
            .filter(conference_session_details::Column::ConferenceSessionUuid.eq(sid))
            .filter(conference_session_details::Column::DomainId.eq(domain))
            .all(db)
            .await
            .unwrap_or_default()
        {
            let dam: conference_session_details::ActiveModel = d.into();
            let _ = dam.delete(db).await;
        }
        let sam: conference_sessions::ActiveModel = sess.into();
        let _ = sam.delete(db).await;
    }
    let Some(r) = center_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference center not found"})),
        )
            .into_response();
    };
    let am: conference_centers::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- rooms ----
async fn room_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    center: &str,
    room: &str,
) -> Option<conference_rooms::Model> {
    conference_rooms::Entity::find_by_id(room.to_owned())
        .filter(conference_rooms::Column::ConferenceCenterUuid.eq(center.to_owned()))
        .filter(conference_rooms::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn list_rooms(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(cid): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if center_or_404(db, domain, &cid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference center not found"})),
        )
            .into_response();
    }
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let rows = conference_rooms::Entity::find()
        .filter(conference_rooms::Column::ConferenceCenterUuid.eq(cid.clone()))
        .filter(conference_rooms::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_room)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/conference_centers/{cid}/rooms"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_room(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(cid): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let name = payload
        .get("conference_room_name")
        .or_else(|| payload.get("room_name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "conference_room_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if center_or_404(db, domain, &cid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference center not found"})),
        )
            .into_response();
    }
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = conference_rooms::ActiveModel {
        conference_room_uuid: Set(id.clone()),
        domain_id: Set(domain),
        conference_center_uuid: Set(cid.clone()),
        conference_room_name: Set(name.to_owned()),
        max_members: Set(payload
            .get("max_members")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        enabled: Set(payload
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "conference_room_uuid": id,
                "conference_center_uuid": cid,
                "conference_room_name": name,
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

async fn get_room(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((cid, rid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match room_or_404(db, domain, &cid, &rid).await {
        Some(r) => (StatusCode::OK, Json(ser_room(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference room not found"})),
        )
            .into_response(),
    }
}

async fn patch_room(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((cid, rid)): axum::extract::Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = room_or_404(db, domain, &cid, &rid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference room not found"})),
        )
            .into_response();
    };
    let mut am: conference_rooms::ActiveModel = r.into();
    if let Some(v) = payload
        .get("conference_room_name")
        .or_else(|| payload.get("room_name"))
        .and_then(|v| v.as_str())
    {
        am.conference_room_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("max_members").and_then(|v| v.as_i64()) {
        am.max_members = Set(Some(v as i32));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_room(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_room(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((cid, rid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = room_or_404(db, domain, &cid, &rid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference room not found"})),
        )
            .into_response();
    };
    let am: conference_rooms::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- sessions ----
async fn session_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    cid: &str,
    sid: &str,
) -> Option<conference_sessions::Model> {
    conference_sessions::Entity::find_by_id(sid.to_owned())
        .filter(conference_sessions::Column::ConferenceCenterUuid.eq(cid.to_owned()))
        .filter(conference_sessions::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn list_sessions(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(cid): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if center_or_404(db, domain, &cid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference center not found"})),
        )
            .into_response();
    }
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let rows = conference_sessions::Entity::find()
        .filter(conference_sessions::Column::ConferenceCenterUuid.eq(cid.clone()))
        .filter(conference_sessions::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_session)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/conference_centers/{cid}/sessions"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_session(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(cid): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if center_or_404(db, domain, &cid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference center not found"})),
        )
            .into_response();
    }
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = conference_sessions::ActiveModel {
        conference_session_uuid: Set(id.clone()),
        domain_id: Set(domain),
        conference_center_uuid: Set(Some(cid.clone())),
        conference_room_uuid: Set(payload
            .get("conference_room_uuid")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        profile: Set(payload
            .get("profile")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "conference_session_uuid": id,
                "conference_center_uuid": cid,
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

async fn get_session(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((cid, sid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match session_or_404(db, domain, &cid, &sid).await {
        Some(r) => (StatusCode::OK, Json(ser_session(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference session not found"})),
        )
            .into_response(),
    }
}

async fn delete_session(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((cid, sid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    for d in conference_session_details::Entity::find()
        .filter(conference_session_details::Column::ConferenceSessionUuid.eq(sid.clone()))
        .filter(conference_session_details::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let dam: conference_session_details::ActiveModel = d.into();
        let _ = dam.delete(db).await;
    }
    let Some(r) = session_or_404(db, domain, &cid, &sid).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference session not found"})),
        )
            .into_response();
    };
    let am: conference_sessions::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_session_details(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((cid, sid)): axum::extract::Path<(String, String)>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if session_or_404(db, domain, &cid, &sid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference session not found"})),
        )
            .into_response();
    }
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let rows = conference_session_details::Entity::find()
        .filter(conference_session_details::Column::ConferenceSessionUuid.eq(sid.clone()))
        .filter(conference_session_details::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
        .map(ser_detail)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/conference_centers/{cid}/sessions/{sid}/details"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

pub fn conference_centers_router() -> Router {
    Router::new()
        .route(
            "/api/v1/conference_centers",
            post(create_center).get(list_centers),
        )
        .route(
            "/api/v1/conference_centers/",
            post(create_center).get(list_centers),
        )
        .route(
            "/api/v1/conference_centers/{id}",
            axum::routing::get(get_center)
                .patch(patch_center)
                .delete(delete_center),
        )
        .route(
            "/api/v1/conference_centers/{cid}/rooms",
            axum::routing::get(list_rooms).post(create_room),
        )
        .route(
            "/api/v1/conference_centers/{cid}/rooms/{rid}",
            axum::routing::get(get_room)
                .patch(patch_room)
                .delete(delete_room),
        )
        .route(
            "/api/v1/conference_centers/{cid}/sessions",
            axum::routing::get(list_sessions).post(create_session),
        )
        .route(
            "/api/v1/conference_centers/{cid}/sessions/{sid}",
            axum::routing::get(get_session).delete(delete_session),
        )
        .route(
            "/api/v1/conference_centers/{cid}/sessions/{sid}/details",
            axum::routing::get(list_session_details),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
