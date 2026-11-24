//! Conference centers router — mirrors `app/conference_centers/router.py`.
//!
//! Order: models → schemas → handlers.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set,
};

// ---- models ----
use pbx_db::entities::{
    conference_centers, conference_rooms, conference_session_details, conference_sessions,
};

// ---- handlers ----
fn ser_center(r: &conference_centers::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "conference_center_uuid": r.conference_center_uuid,
        "conference_center_name": r.conference_center_name,
        "conference_center_extension": r.conference_center_extension,
        "conference_center_description": r.conference_center_description,
        "conference_center_enabled": r.conference_center_enabled,
        "dialplan_uuid": r.dialplan_uuid,
        "conference_center_pin_length": r.conference_center_pin_length,
        "conference_center_greeting": r.conference_center_greeting,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

fn ser_room(r: &conference_rooms::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "conference_room_uuid": r.conference_room_uuid,
        "conference_center_uuid": r.conference_center_uuid,
        "conference_room_name": r.conference_room_name,
        "max_members": r.max_members,
        "enabled": r.enabled,
        "profile": r.profile,
        "moderator_pin": r.moderator_pin,
        "participant_pin": r.participant_pin,
        "start_datetime": r.start_datetime,
        "stop_datetime": r.stop_datetime,
        "wait_mod": r.wait_mod,
        "moderator_endconf": r.moderator_endconf,
        "announce_name": r.announce_name,
        "announce_count": r.announce_count,
        "announce_recording": r.announce_recording,
        "sounds": r.sounds,
        "mute": r.mute,
        "email_address": r.email_address,
        "account_code": r.account_code,
        "description": r.description,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

fn ser_session(r: &conference_sessions::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "conference_session_uuid": r.conference_session_uuid,
        "conference_center_uuid": r.conference_center_uuid,
        "conference_room_uuid": r.conference_room_uuid,
        "profile": r.profile,
        "meeting_uuid": r.meeting_uuid,
        "recording": r.recording,
        "start_epoch": r.start_epoch,
        "end_epoch": r.end_epoch,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

fn ser_detail(r: &conference_session_details::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "conference_session_detail_uuid": r.conference_session_detail_uuid,
        "conference_session_uuid": r.conference_session_uuid,
        "username": r.username,
        "meeting_uuid": r.meeting_uuid,
        "caller_id_name": r.caller_id_name,
        "caller_id_number": r.caller_id_number,
        "moderator": r.moderator,
        "network_addr": r.network_addr,
        "start_epoch": r.start_epoch,
        "end_epoch": r.end_epoch,
    });
    crate::alias::emit_aliases(&mut v);
    v
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_center_add") {
        return (s, j).into_response();
    }
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
        dialplan_uuid: Set(payload
            .get("dialplan_uuid")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        conference_center_pin_length: Set(payload
            .get("conference_center_pin_length")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        conference_center_greeting: Set(payload
            .get("conference_center_greeting")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_center_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "conference_center_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/conference_centers", page, per_page, 0),
                Json(crate::paged_body(
                    Vec::<serde_json::Value>::new(),
                    0,
                    page,
                    per_page,
                )),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let mut sel = conference_centers::Entity::find();
    if !show_all {
        sel = sel.filter(conference_centers::Column::DomainId.eq(domain));
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_center).collect();
    (
        crate::paged_headers("/api/v1/conference_centers", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_center(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_center_view") {
        return (s, j).into_response();
    }
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_center_edit") {
        return (s, j).into_response();
    }
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
    if payload.get("dialplan_uuid").is_some() {
        am.dialplan_uuid = Set(payload
            .get("dialplan_uuid")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("conference_center_pin_length").is_some() {
        am.conference_center_pin_length = Set(payload
            .get("conference_center_pin_length")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32));
    }
    if payload.get("conference_center_greeting").is_some() {
        am.conference_center_greeting = Set(payload
            .get("conference_center_greeting")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_center_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    // Set deletes: no per-row fetch-then-delete round-trips. Only session
    // ids (tiny strings) are read; details/rooms/sessions go in 3 statements.
    let sess_ids: Vec<String> = conference_sessions::Entity::find()
        .filter(conference_sessions::Column::ConferenceCenterUuid.eq(id.clone()))
        .filter(conference_sessions::Column::DomainId.eq(domain))
        .select_only()
        .column(conference_sessions::Column::ConferenceSessionUuid)
        .into_tuple::<String>()
        .all(db)
        .await
        .unwrap_or_default();
    if !sess_ids.is_empty() {
        let _ = conference_session_details::Entity::delete_many()
            .filter(conference_session_details::Column::ConferenceSessionUuid.is_in(sess_ids))
            .filter(conference_session_details::Column::DomainId.eq(domain))
            .exec(db)
            .await;
    }
    let _ = conference_sessions::Entity::delete_many()
        .filter(conference_sessions::Column::ConferenceCenterUuid.eq(id.clone()))
        .filter(conference_sessions::Column::DomainId.eq(domain))
        .exec(db)
        .await;
    let _ = conference_rooms::Entity::delete_many()
        .filter(conference_rooms::Column::ConferenceCenterUuid.eq(id.clone()))
        .filter(conference_rooms::Column::DomainId.eq(domain))
        .exec(db)
        .await;
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_room_view") {
        return (s, j).into_response();
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
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = conference_rooms::Entity::find()
        .filter(conference_rooms::Column::ConferenceCenterUuid.eq(cid.clone()))
        .filter(conference_rooms::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_room).collect();
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_room_add") {
        return (s, j).into_response();
    }
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
        profile: Set(payload
            .get("profile")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        moderator_pin: Set(payload
            .get("moderator_pin")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        participant_pin: Set(payload
            .get("participant_pin")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        start_datetime: Set(payload
            .get("start_datetime")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())),
        stop_datetime: Set(payload
            .get("stop_datetime")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())),
        wait_mod: Set(payload.get("wait_mod").and_then(|v| v.as_bool())),
        moderator_endconf: Set(payload.get("moderator_endconf").and_then(|v| v.as_bool())),
        announce_name: Set(payload.get("announce_name").and_then(|v| v.as_bool())),
        announce_count: Set(payload.get("announce_count").and_then(|v| v.as_bool())),
        announce_recording: Set(payload.get("announce_recording").and_then(|v| v.as_bool())),
        sounds: Set(payload
            .get("sounds")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        mute: Set(payload.get("mute").and_then(|v| v.as_bool())),
        created: Set(payload
            .get("created")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_by: Set(payload
            .get("created_by")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        email_address: Set(payload
            .get("email_address")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        account_code: Set(payload
            .get("account_code")
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
        Ok(_) => match room_or_404(db, domain, &cid, &id).await {
            Some(r) => (StatusCode::CREATED, Json(ser_room(&r))).into_response(),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": "insert failed"})),
            )
                .into_response(),
        },
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_room_view") {
        return (s, j).into_response();
    }
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_room_edit") {
        return (s, j).into_response();
    }
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
    macro_rules! rstr {
        ($f:ident) => {
            if payload.get(stringify!($f)).is_some() {
                am.$f = Set(payload
                    .get(stringify!($f))
                    .and_then(|v| v.as_str())
                    .map(str::to_owned));
            }
        };
    }
    macro_rules! rbool {
        ($f:ident) => {
            if payload.get(stringify!($f)).is_some() {
                am.$f = Set(payload.get(stringify!($f)).and_then(|v| v.as_bool()));
            }
        };
    }
    rstr!(profile);
    rstr!(moderator_pin);
    rstr!(participant_pin);
    rbool!(wait_mod);
    rbool!(moderator_endconf);
    rbool!(announce_name);
    rbool!(announce_count);
    rbool!(announce_recording);
    rstr!(sounds);
    rbool!(mute);
    rstr!(email_address);
    rstr!(account_code);
    rstr!(description);
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_room_delete") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_session_view") {
        return (s, j).into_response();
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
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = conference_sessions::Entity::find()
        .filter(conference_sessions::Column::ConferenceCenterUuid.eq(cid.clone()))
        .filter(conference_sessions::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_session).collect();
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_session_add") {
        return (s, j).into_response();
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
        meeting_uuid: Set(None),
        recording: Set(None),
        start_epoch: Set(None),
        end_epoch: Set(None),
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_session_view") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_session_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let _ = conference_session_details::Entity::delete_many()
        .filter(conference_session_details::Column::ConferenceSessionUuid.eq(sid.clone()))
        .filter(conference_session_details::Column::DomainId.eq(domain))
        .exec(db)
        .await;
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "conference_session_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if session_or_404(db, domain, &cid, &sid).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Conference session not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = conference_session_details::Entity::find()
        .filter(conference_session_details::Column::ConferenceSessionUuid.eq(sid.clone()))
        .filter(conference_session_details::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_detail).collect();
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
