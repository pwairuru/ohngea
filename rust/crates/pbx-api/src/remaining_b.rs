//! Spillover replacement (part B): per-table CRUD for feature_codes,
//! music_on_hold, phrases (+ details), pin_numbers, number_translations
//! (+ details) and streams.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::Value;

// ---- models ----
type Claims = pbx_core::auth::Claims;
use crate::remaining_a::{b, envelope, i, s, tenant_domain, unprocessable};
use pbx_db::entities::{
    feature_codes, music_on_hold, number_translation_details, number_translations, phrase_details,
    phrases, pin_numbers, streams,
};

// ---- handlers: feature_codes ----
fn ser_feature_code(r: &feature_codes::Model) -> Value {
    serde_json::json!({
        "feature_code_uuid": r.feature_code_uuid,
        "domain_id": r.domain_id.to_string(),
        "feature_code_name": r.feature_code_name,
        "name": r.feature_code_name,
        "feature_code_value": r.feature_code_value,
        "value": r.feature_code_value,
        "feature_code_enabled": r.feature_code_enabled,
        "enabled": r.feature_code_enabled,
        "feature_code_description": r.feature_code_description,
    })
}

async fn create_feature_code(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "feature_code_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["feature_code_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = feature_codes::ActiveModel {
        feature_code_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        feature_code_name: Set(name),
        feature_code_value: Set(s(&payload, &["feature_code_value", "value"]).unwrap_or_default()),
        feature_code_enabled: Set(b(&payload, &["feature_code_enabled", "enabled"], true)),
        feature_code_description: Set(s(&payload, &["feature_code_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_feature_code(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_feature_codes(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "feature_code_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = feature_codes::Entity::find()
        .filter(feature_codes::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_feature_code).collect()).into_response()
}

async fn get_feature_code(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "feature_code_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match feature_codes::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            (StatusCode::OK, Json(ser_feature_code(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_feature_code(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "feature_code_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = feature_codes::Entity::find_by_id(id)
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: feature_codes::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["feature_code_name", "name"]) {
        am.feature_code_name = Set(v);
    }
    if let Some(v) = s(&payload, &["feature_code_value", "value"]) {
        am.feature_code_value = Set(v);
    }
    if payload
        .get("feature_code_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.feature_code_enabled = Set(b(&payload, &["feature_code_enabled", "enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_feature_code(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_feature_code(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "feature_code_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match feature_codes::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: feature_codes::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn feature_codes_router() -> Router {
    Router::new()
        .route(
            "/api/v1/feature_codes",
            axum::routing::post(create_feature_code).get(list_feature_codes),
        )
        .route(
            "/api/v1/feature_codes/{id}",
            axum::routing::get(get_feature_code)
                .patch(patch_feature_code)
                .delete(delete_feature_code),
        )
}

// ---- handlers: music_on_hold ----
fn ser_moh(r: &music_on_hold::Model) -> Value {
    serde_json::json!({
        "music_on_hold_uuid": r.music_on_hold_uuid,
        "domain_id": r.domain_id.to_string(),
        "music_name": r.music_name,
        "name": r.music_name,
        "music_path": r.music_path,
        "path": r.music_path,
        "music_rate": r.music_rate,
        "enabled": r.enabled,
        "description": r.description,
    })
}

async fn create_moh(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "music_on_hold_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["music_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = music_on_hold::ActiveModel {
        music_on_hold_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        music_name: Set(name),
        music_path: Set(s(&payload, &["music_path", "path"]).unwrap_or_default()),
        music_rate: Set(Some(i(&payload, &["music_rate"], 8000))),
        music_shuffle: Set(s(&payload, &["music_shuffle"])),
        music_channels: Set(Some(i(&payload, &["music_channels"], 1))),
        music_interval: Set(Some(i(&payload, &["music_interval"], 20))),
        enabled: Set(b(&payload, &["enabled"], true)),
        description: Set(s(&payload, &["description"])),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_moh(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_moh(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "music_on_hold_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = music_on_hold::Entity::find()
        .filter(music_on_hold::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_moh).collect()).into_response()
}

async fn get_moh(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "music_on_hold_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match music_on_hold::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_moh(&r))).into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_moh(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "music_on_hold_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = music_on_hold::Entity::find_by_id(id)
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: music_on_hold::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["music_name", "name"]) {
        am.music_name = Set(v);
    }
    if let Some(v) = s(&payload, &["music_path", "path"]) {
        am.music_path = Set(v);
    }
    if payload.get("enabled").is_some() {
        am.enabled = Set(b(&payload, &["enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_moh(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_moh(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "music_on_hold_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match music_on_hold::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: music_on_hold::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn moh_router() -> Router {
    Router::new()
        .route(
            "/api/v1/music_on_hold",
            axum::routing::post(create_moh).get(list_moh),
        )
        .route(
            "/api/v1/music_on_hold/{id}",
            axum::routing::get(get_moh)
                .patch(patch_moh)
                .delete(delete_moh),
        )
}

// ---- handlers: phrases (+ details child) ----
fn ser_phrase(r: &phrases::Model) -> Value {
    serde_json::json!({
        "phrase_uuid": r.phrase_uuid,
        "domain_id": r.domain_id.to_string(),
        "phrase_name": r.phrase_name,
        "name": r.phrase_name,
        "phrase_language": r.phrase_language,
        "phrase_enabled": r.phrase_enabled,
        "enabled": r.phrase_enabled,
        "phrase_description": r.phrase_description,
    })
}

fn ser_phrase_detail(d: &phrase_details::Model) -> Value {
    serde_json::json!({
        "phrase_detail_uuid": d.phrase_detail_uuid,
        "phrase_uuid": d.phrase_uuid,
        "phrase_detail_function": d.phrase_detail_function,
        "phrase_detail_data": d.phrase_detail_data,
        "phrase_detail_order": d.phrase_detail_order,
    })
}

async fn create_phrase(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "phrase_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["phrase_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = phrases::ActiveModel {
        phrase_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        phrase_name: Set(name),
        phrase_language: Set(s(&payload, &["phrase_language"])),
        phrase_enabled: Set(b(&payload, &["phrase_enabled", "enabled"], true)),
        phrase_description: Set(s(&payload, &["phrase_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_phrase(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_phrases(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "phrase_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = phrases::Entity::find()
        .filter(phrases::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_phrase).collect()).into_response()
}

async fn scoped_phrase(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<phrases::Model> {
    match phrases::Entity::find_by_id(id.to_owned())
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => Some(r),
        _ => None,
    }
}

async fn get_phrase(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "phrase_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_phrase(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_phrase(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_phrase(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "phrase_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = scoped_phrase(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: phrases::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["phrase_name", "name"]) {
        am.phrase_name = Set(v);
    }
    if payload
        .get("phrase_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.phrase_enabled = Set(b(&payload, &["phrase_enabled", "enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_phrase(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_phrase(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "phrase_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_phrase(db, domain, &id).await {
        Some(r) => {
            let am: phrases::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn create_phrase_detail(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "phrase_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_phrase(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let am = phrase_details::ActiveModel {
        phrase_detail_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        phrase_uuid: Set(id.clone()),
        phrase_detail_function: Set(s(&payload, &["phrase_detail_function", "function"])),
        phrase_detail_data: Set(s(&payload, &["phrase_detail_data", "data"])),
        phrase_detail_order: Set(i(&payload, &["phrase_detail_order", "order"], 0)),
        created_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(d) => (StatusCode::CREATED, Json(ser_phrase_detail(&d))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_phrase_details(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "phrase_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_phrase(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let rows = phrase_details::Entity::find()
        .filter(phrase_details::Column::PhraseUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_phrase_detail).collect()).into_response()
}

fn phrases_router() -> Router {
    Router::new()
        .route(
            "/api/v1/phrases",
            axum::routing::post(create_phrase).get(list_phrases),
        )
        .route(
            "/api/v1/phrases/{id}",
            axum::routing::get(get_phrase)
                .patch(patch_phrase)
                .delete(delete_phrase),
        )
        .route(
            "/api/v1/phrases/{id}/details",
            axum::routing::post(create_phrase_detail).get(list_phrase_details),
        )
}

// ---- handlers: pin_numbers ----
fn ser_pin(r: &pin_numbers::Model) -> Value {
    serde_json::json!({
        "pin_number_uuid": r.pin_number_uuid,
        "domain_id": r.domain_id.to_string(),
        "pin_number": r.pin_number,
        "number": r.pin_number,
        "accountcode": r.accountcode,
        "enabled": r.enabled,
        "description": r.description,
    })
}

async fn create_pin(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "pin_number_add") {
        return (s, j).into_response();
    }
    let Some(number) = s(&payload, &["pin_number", "number"]) else {
        return unprocessable("number required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = pin_numbers::ActiveModel {
        pin_number_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        pin_number: Set(number),
        accountcode: Set(s(&payload, &["accountcode"])),
        enabled: Set(b(&payload, &["enabled"], true)),
        description: Set(s(&payload, &["description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_pin(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_pins(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "pin_number_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = pin_numbers::Entity::find()
        .filter(pin_numbers::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_pin).collect()).into_response()
}

async fn get_pin(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "pin_number_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match pin_numbers::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_pin(&r))).into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_pin(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "pin_number_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = pin_numbers::Entity::find_by_id(id)
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: pin_numbers::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["pin_number", "number"]) {
        am.pin_number = Set(v);
    }
    if payload.get("enabled").is_some() {
        am.enabled = Set(b(&payload, &["enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_pin(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_pin(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "pin_number_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match pin_numbers::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: pin_numbers::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn pins_router() -> Router {
    Router::new()
        .route(
            "/api/v1/pin_numbers",
            axum::routing::post(create_pin).get(list_pins),
        )
        .route(
            "/api/v1/pin_numbers/{id}",
            axum::routing::get(get_pin)
                .patch(patch_pin)
                .delete(delete_pin),
        )
}

// ---- handlers: number_translations (+ details child) ----
fn ser_nt(r: &number_translations::Model) -> Value {
    serde_json::json!({
        "number_translation_uuid": r.number_translation_uuid,
        "domain_id": r.domain_id.to_string(),
        "number_translation_name": r.number_translation_name,
        "name": r.number_translation_name,
        "number_translation_regex": r.number_translation_regex,
        "number_translation_replace": r.number_translation_replace,
        "number_translation_enabled": r.number_translation_enabled,
        "enabled": r.number_translation_enabled,
        "number_translation_description": r.number_translation_description,
    })
}

fn ser_nt_detail(d: &number_translation_details::Model) -> Value {
    serde_json::json!({
        "number_translation_detail_uuid": d.number_translation_detail_uuid,
        "number_translation_uuid": d.number_translation_uuid,
        "number_translation_detail_regex": d.number_translation_detail_regex,
        "number_translation_detail_replace": d.number_translation_detail_replace,
        "number_translation_detail_order": d.number_translation_detail_order,
    })
}

async fn create_nt(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "number_translation_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["number_translation_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = number_translations::ActiveModel {
        number_translation_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        number_translation_name: Set(name),
        number_translation_regex: Set(s(&payload, &["number_translation_regex", "regex"])),
        number_translation_replace: Set(s(&payload, &["number_translation_replace", "replace"])),
        number_translation_enabled: Set(b(
            &payload,
            &["number_translation_enabled", "enabled"],
            true,
        )),
        number_translation_description: Set(s(
            &payload,
            &["number_translation_description", "description"],
        )),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_nt(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_nt(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "number_translation_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = number_translations::Entity::find()
        .filter(number_translations::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_nt).collect()).into_response()
}

async fn scoped_nt(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<number_translations::Model> {
    match number_translations::Entity::find_by_id(id.to_owned())
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => Some(r),
        _ => None,
    }
}

async fn get_nt(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "number_translation_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_nt(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_nt(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_nt(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "number_translation_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = scoped_nt(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: number_translations::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["number_translation_name", "name"]) {
        am.number_translation_name = Set(v);
    }
    if let Some(v) = s(&payload, &["number_translation_regex", "regex"]) {
        am.number_translation_regex = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["number_translation_replace", "replace"]) {
        am.number_translation_replace = Set(Some(v));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_nt(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_nt(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "number_translation_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_nt(db, domain, &id).await {
        Some(r) => {
            let am: number_translations::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn create_nt_detail(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "number_translation_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_nt(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let (Some(regex), Some(replace)) = (
        s(&payload, &["number_translation_detail_regex", "regex"]),
        s(&payload, &["number_translation_detail_replace", "replace"]),
    ) else {
        return unprocessable("regex and replace required").into_response();
    };
    let now = crate::routing_db::now();
    let am = number_translation_details::ActiveModel {
        number_translation_detail_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        number_translation_uuid: Set(id.clone()),
        number_translation_detail_regex: Set(regex),
        number_translation_detail_replace: Set(replace),
        number_translation_detail_order: Set(i(
            &payload,
            &["number_translation_detail_order", "order"],
            0,
        )),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(d) => (StatusCode::CREATED, Json(ser_nt_detail(&d))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_nt_details(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "number_translation_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_nt(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let rows = number_translation_details::Entity::find()
        .filter(number_translation_details::Column::NumberTranslationUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_nt_detail).collect()).into_response()
}

fn nt_router() -> Router {
    Router::new()
        .route(
            "/api/v1/number_translations",
            axum::routing::post(create_nt).get(list_nt),
        )
        .route(
            "/api/v1/number_translations/{id}",
            axum::routing::get(get_nt).patch(patch_nt).delete(delete_nt),
        )
        .route(
            "/api/v1/number_translations/{id}/details",
            axum::routing::post(create_nt_detail).get(list_nt_details),
        )
}

// ---- handlers: streams ----
fn ser_stream(r: &streams::Model) -> Value {
    serde_json::json!({
        "stream_uuid": r.stream_uuid,
        "domain_id": r.domain_id.to_string(),
        "stream_name": r.stream_name,
        "name": r.stream_name,
        "stream_location": r.stream_location,
        "stream_enabled": r.stream_enabled,
        "enabled": r.stream_enabled,
        "stream_description": r.stream_description,
    })
}

async fn create_stream(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "stream_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["stream_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = streams::ActiveModel {
        stream_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        stream_name: Set(name),
        stream_location: Set(s(&payload, &["stream_location", "location"])),
        stream_enabled: Set(b(&payload, &["stream_enabled", "enabled"], true)),
        stream_description: Set(s(&payload, &["stream_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_stream(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_streams(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "stream_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = streams::Entity::find()
        .filter(streams::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_stream).collect()).into_response()
}

async fn get_stream(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "stream_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match streams::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_stream(&r))).into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_stream(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "stream_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = streams::Entity::find_by_id(id)
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: streams::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["stream_name", "name"]) {
        am.stream_name = Set(v);
    }
    if let Some(v) = s(&payload, &["stream_location", "location"]) {
        am.stream_location = Set(Some(v));
    }
    if payload
        .get("stream_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.stream_enabled = Set(b(&payload, &["stream_enabled", "enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_stream(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_stream(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "stream_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match streams::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: streams::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn streams_router() -> Router {
    Router::new()
        .route(
            "/api/v1/streams",
            axum::routing::post(create_stream).get(list_streams),
        )
        .route(
            "/api/v1/streams/{id}",
            axum::routing::get(get_stream)
                .patch(patch_stream)
                .delete(delete_stream),
        )
}

// ---- registry ----
pub fn spillover_b_router() -> Router {
    feature_codes_router()
        .merge(moh_router())
        .merge(phrases_router())
        .merge(pins_router())
        .merge(nt_router())
        .merge(streams_router())
}
