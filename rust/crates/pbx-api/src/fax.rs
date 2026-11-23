//! Fax router — mirrors `app/fax/router.py` (accounts/files/logs/users/send)
//! + `app/fax_queue` (status/retry).
//!
//! Order: models → schemas → handlers → xml (no FS XML for fax; S3/file stub note).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{fax_files, fax_logs, fax_queue, fax_users, faxes};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct FaxQueueSubmit {
    fax_extension: String,
    destination: String,
}

#[derive(Debug, Serialize)]
struct FaxQueueOut {
    fax_queue_uuid: String,
    fax_extension: String,
    destination: String,
    status: String,
}

// ---- handlers ----
/// PHP ground truth (`fax_edit.php`): `empty($fax_extension)` is the only
/// server-side rule — no digit/length regex.
fn valid_ext(s: &str) -> bool {
    !s.trim().is_empty()
}

fn ser_fax(r: &faxes::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "fax_uuid": r.fax_uuid,
        "fax_extension": r.fax_extension,
        "fax_name": r.fax_name,
        "description": r.description,
        "enabled": r.enabled,
        "dialplan_uuid": r.dialplan_uuid,
        "fax_destination_number": r.fax_destination_number,
        "fax_prefix": r.fax_prefix,
        "fax_email_connection_type": r.fax_email_connection_type,
        "fax_email_connection_host": r.fax_email_connection_host,
        "fax_email_connection_port": r.fax_email_connection_port,
        "fax_email_connection_security": r.fax_email_connection_security,
        "fax_email_connection_validate": r.fax_email_connection_validate,
        "fax_email_connection_username": r.fax_email_connection_username,
        "fax_email_connection_password": r.fax_email_connection_password,
        "fax_email_connection_mailbox": r.fax_email_connection_mailbox,
        "fax_email_inbound_subject_tag": r.fax_email_inbound_subject_tag,
        "fax_email_outbound_subject_tag": r.fax_email_outbound_subject_tag,
        "fax_email_outbound_authorized_senders": r.fax_email_outbound_authorized_senders,
        "fax_pin_number": r.fax_pin_number,
        "fax_caller_id_name": r.fax_caller_id_name,
        "fax_caller_id_number": r.fax_caller_id_number,
        "fax_toll_allow": r.fax_toll_allow,
        "fax_file": r.fax_file,
        "fax_user_list": r.fax_user_list,
        "fax_forward_number": r.fax_forward_number,
        "fax_send_channels": r.fax_send_channels,
        "accountcode": r.accountcode,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

fn ser_queue(r: &fax_queue::Model) -> serde_json::Value {
    let mut v = serde_json::json!({
        "fax_queue_uuid": r.fax_queue_uuid,
        "fax_uuid": r.fax_uuid,
        "fax_extension": r.fax_extension,
        "destination": r.destination,
        "status": r.status,
        "origination_uuid": r.origination_uuid,
        "fax_log_uuid": r.fax_log_uuid,
        "fax_date": r.fax_date,
        "hostname": r.hostname,
        "fax_caller_id_name": r.fax_caller_id_name,
        "fax_caller_id_number": r.fax_caller_id_number,
        "fax_recipient": r.fax_recipient,
        "fax_number": r.fax_number,
        "fax_prefix": r.fax_prefix,
        "fax_email_address": r.fax_email_address,
        "fax_file": r.fax_file,
        "fax_retry_date": r.fax_retry_date,
        "fax_notify_sent": r.fax_notify_sent,
        "fax_notify_date": r.fax_notify_date,
        "fax_retry_count": r.fax_retry_count,
        "fax_accountcode": r.fax_accountcode,
        "fax_command": r.fax_command,
        "fax_response": r.fax_response,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn fax_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<faxes::Model> {
    // PHP parity (fax_edit.php): detail loads are tenant-scoped; global
    // rows surface in lists only.
    faxes::Entity::find_by_id(id.to_owned())
        .filter(faxes::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn health_fax() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "fax"}))
}

async fn create_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_add") {
        return (s, j).into_response();
    }
    let ext = payload
        .get("fax_extension")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    let name = payload
        .get("fax_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if !valid_ext(ext) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "fax_extension required"})),
        )
            .into_response();
    }
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "fax_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = faxes::ActiveModel {
        fax_uuid: Set(id.clone()),
        domain_id: Set(Some(domain)),
        fax_extension: Set(ext.to_owned()),
        fax_name: Set(name.to_owned()),
        description: Set(payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        enabled: Set(true),
        dialplan_uuid: Set(payload
            .get("dialplan_uuid")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_destination_number: Set(payload
            .get("fax_destination_number")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_prefix: Set(payload
            .get("fax_prefix")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_connection_type: Set(payload
            .get("fax_email_connection_type")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_connection_host: Set(payload
            .get("fax_email_connection_host")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_connection_port: Set(payload
            .get("fax_email_connection_port")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        fax_email_connection_security: Set(payload
            .get("fax_email_connection_security")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_connection_validate: Set(payload
            .get("fax_email_connection_validate")
            .and_then(|v| v.as_bool())),
        fax_email_connection_username: Set(payload
            .get("fax_email_connection_username")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_connection_password: Set(payload
            .get("fax_email_connection_password")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_connection_mailbox: Set(payload
            .get("fax_email_connection_mailbox")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_inbound_subject_tag: Set(payload
            .get("fax_email_inbound_subject_tag")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_outbound_subject_tag: Set(payload
            .get("fax_email_outbound_subject_tag")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_email_outbound_authorized_senders: Set(payload
            .get("fax_email_outbound_authorized_senders")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_pin_number: Set(payload
            .get("fax_pin_number")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_caller_id_name: Set(payload
            .get("fax_caller_id_name")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_caller_id_number: Set(payload
            .get("fax_caller_id_number")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_toll_allow: Set(payload
            .get("fax_toll_allow")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_file: Set(payload
            .get("fax_file")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_user_list: Set(payload
            .get("fax_user_list")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_forward_number: Set(payload
            .get("fax_forward_number")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_send_channels: Set(payload
            .get("fax_send_channels")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        accountcode: Set(payload
            .get("accountcode")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "fax_uuid": id,
                "fax_extension": ext,
                "fax_name": name,
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

async fn list_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "fax_extension_view_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/fax", page, per_page, 0),
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
    let mut sel = faxes::Entity::find();
    if !show_all {
        // PHP three-tier visibility (app/fax/fax.php): domain perm sees
        // own domain + global rows; plain fax_view sees only assigned rows.
        if crate::tenant::sees_domain_nulls(&claims, "fax_extension_view_domain")
            || crate::tenant::has_permission(&claims, "fax_extension_view_all")
        {
            sel = sel.filter(
                Condition::any()
                    .add(faxes::Column::DomainId.eq(domain))
                    .add(faxes::Column::DomainId.is_null()),
            );
        } else {
            let mine: Vec<String> = fax_users::Entity::find()
                .filter(fax_users::Column::UserUuid.eq(claims.sub.clone()))
                .all(&txn)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|r| r.fax_uuid)
                .collect();
            sel = sel
                .filter(faxes::Column::DomainId.eq(domain))
                .filter(faxes::Column::FaxUuid.is_in(mine));
        }
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_fax).collect();
    (
        crate::paged_headers("/api/v1/fax", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match fax_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_fax(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response(),
    }
}

async fn patch_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = fax_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    };
    let mut am: faxes::ActiveModel = r.into();
    if let Some(v) = payload.get("fax_name").and_then(|v| v.as_str()) {
        am.fax_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    for key in [
        "fax_extension",
        "dialplan_uuid",
        "fax_destination_number",
        "fax_prefix",
        "fax_email_connection_type",
        "fax_email_connection_host",
        "fax_email_connection_security",
        "fax_email_connection_username",
        "fax_email_connection_password",
        "fax_email_connection_mailbox",
        "fax_email_inbound_subject_tag",
        "fax_email_outbound_subject_tag",
        "fax_email_outbound_authorized_senders",
        "fax_pin_number",
        "fax_caller_id_name",
        "fax_caller_id_number",
        "fax_toll_allow",
        "fax_file",
        "fax_user_list",
        "fax_forward_number",
        "accountcode",
    ] {
        if let Some(v) = payload.get(key).and_then(|v| v.as_str()) {
            match key {
                "fax_extension" => am.fax_extension = Set(v.to_owned()),
                "dialplan_uuid" => am.dialplan_uuid = Set(Some(v.to_owned())),
                "fax_destination_number" => am.fax_destination_number = Set(Some(v.to_owned())),
                "fax_prefix" => am.fax_prefix = Set(Some(v.to_owned())),
                "fax_email_connection_type" => {
                    am.fax_email_connection_type = Set(Some(v.to_owned()))
                }
                "fax_email_connection_host" => {
                    am.fax_email_connection_host = Set(Some(v.to_owned()))
                }
                "fax_email_connection_security" => {
                    am.fax_email_connection_security = Set(Some(v.to_owned()))
                }
                "fax_email_connection_username" => {
                    am.fax_email_connection_username = Set(Some(v.to_owned()))
                }
                "fax_email_connection_password" => {
                    am.fax_email_connection_password = Set(Some(v.to_owned()))
                }
                "fax_email_connection_mailbox" => {
                    am.fax_email_connection_mailbox = Set(Some(v.to_owned()))
                }
                "fax_email_inbound_subject_tag" => {
                    am.fax_email_inbound_subject_tag = Set(Some(v.to_owned()))
                }
                "fax_email_outbound_subject_tag" => {
                    am.fax_email_outbound_subject_tag = Set(Some(v.to_owned()))
                }
                "fax_email_outbound_authorized_senders" => {
                    am.fax_email_outbound_authorized_senders = Set(Some(v.to_owned()))
                }
                "fax_pin_number" => am.fax_pin_number = Set(Some(v.to_owned())),
                "fax_caller_id_name" => am.fax_caller_id_name = Set(Some(v.to_owned())),
                "fax_caller_id_number" => am.fax_caller_id_number = Set(Some(v.to_owned())),
                "fax_toll_allow" => am.fax_toll_allow = Set(Some(v.to_owned())),
                "fax_file" => am.fax_file = Set(Some(v.to_owned())),
                "fax_user_list" => am.fax_user_list = Set(Some(v.to_owned())),
                "fax_forward_number" => am.fax_forward_number = Set(Some(v.to_owned())),
                _ => am.accountcode = Set(Some(v.to_owned())),
            }
        }
    }
    if let Some(v) = payload
        .get("fax_email_connection_port")
        .and_then(|v| v.as_i64())
    {
        am.fax_email_connection_port = Set(Some(v as i32));
    }
    if let Some(v) = payload
        .get("fax_email_connection_validate")
        .and_then(|v| v.as_bool())
    {
        am.fax_email_connection_validate = Set(Some(v));
    }
    if let Some(v) = payload.get("fax_send_channels").and_then(|v| v.as_i64()) {
        am.fax_send_channels = Set(Some(v as i32));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_fax(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    // Set deletes: 3 statements, no per-row fetch-then-delete round-trips.
    let _ = fax_files::Entity::delete_many()
        .filter(fax_files::Column::FaxUuid.eq(id.clone()))
        .filter(fax_files::Column::DomainId.eq(domain))
        .exec(db)
        .await;
    let _ = fax_logs::Entity::delete_many()
        .filter(fax_logs::Column::FaxUuid.eq(id.clone()))
        .filter(fax_logs::Column::DomainId.eq(domain))
        .exec(db)
        .await;
    let _ = fax_users::Entity::delete_many()
        .filter(fax_users::Column::FaxUuid.eq(id.clone()))
        .filter(fax_users::Column::DomainId.eq(domain))
        .exec(db)
        .await;
    let Some(r) = fax_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    };
    let am: faxes::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_files(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = fax_files::Entity::find()
        .filter(fax_files::Column::FaxUuid.eq(id.clone()))
        .filter(fax_files::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let mut v = serde_json::json!({
                "fax_file_uuid": r.fax_file_uuid,
                "fax_uuid": r.fax_uuid,
                "fax_file_path": r.fax_file_path,
                "fax_mode": r.fax_mode,
                "fax_recipient": r.fax_recipient,
                "fax_date": r.fax_date,
            });
            crate::alias::emit_aliases(&mut v);
            v
        })
        .collect();
    (
        crate::paged_headers(&format!("/api/v1/fax/{id}/files"), page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn upload_file(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let fid = crate::routing_db::new_uuid();
    let am = fax_files::ActiveModel {
        fax_file_uuid: Set(fid.clone()),
        domain_id: Set(domain),
        fax_uuid: Set(id.clone()),
        fax_mode: Set(payload
            .get("fax_mode")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_recipient: Set(payload
            .get("fax_recipient")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_destination: Set(payload
            .get("fax_destination")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_file_type: Set(payload
            .get("fax_file_type")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_file_path: Set(payload
            .get("fax_file_path")
            .or_else(|| payload.get("file_name"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        fax_caller_id_name: Set(None),
        fax_caller_id_number: Set(None),
        fax_date: Set(None),
        fax_epoch: Set(None),
        fax_base64: Set(payload
            .get("fax_base64")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        read_date: Set(None),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"fax_file_uuid": fid, "fax_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_logs(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = fax_logs::Entity::find()
        .filter(fax_logs::Column::FaxUuid.eq(id.clone()))
        .filter(fax_logs::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let mut v = serde_json::json!({
                "fax_log_uuid": r.fax_log_uuid,
                "fax_uuid": r.fax_uuid,
                "fax_success": r.fax_success,
                "fax_result_code": r.fax_result_code,
                "fax_result_text": r.fax_result_text,
                "fax_file": r.fax_file,
                "fax_duration": r.fax_duration,
            });
            crate::alias::emit_aliases(&mut v);
            v
        })
        .collect();
    (
        crate::paged_headers(&format!("/api/v1/fax/{id}/logs"), page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn list_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = fax_users::Entity::find()
        .filter(fax_users::Column::FaxUuid.eq(id.clone()))
        .filter(fax_users::Column::DomainId.eq(domain))
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
                "fax_user_uuid": r.fax_user_uuid,
                "fax_uuid": r.fax_uuid,
                "user_uuid": r.user_uuid,
            })
        })
        .collect();
    (
        crate::paged_headers(&format!("/api/v1/fax/{id}/users"), page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn send_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(f) = fax_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    };
    let dest = payload
        .get("destination")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if dest.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "destination required"})),
        )
            .into_response();
    }
    // Enqueue a fax_queue job (mirrors Python send → queue insert).
    let qid = crate::routing_db::new_uuid();
    let am = fax_queue::ActiveModel {
        fax_queue_uuid: Set(qid.clone()),
        domain_id: Set(domain),
        fax_uuid: Set(Some(id.clone())),
        fax_extension: Set(f.fax_extension),
        destination: Set(dest.to_owned()),
        status: Set("waiting".to_owned()),
        origination_uuid: Set(None),
        fax_log_uuid: Set(None),
        fax_date: Set(None),
        hostname: Set(None),
        fax_caller_id_name: Set(None),
        fax_caller_id_number: Set(None),
        fax_recipient: Set(None),
        fax_number: Set(Some(dest.to_owned())),
        fax_prefix: Set(None),
        fax_email_address: Set(None),
        fax_file: Set(None),
        fax_retry_date: Set(None),
        fax_notify_sent: Set(None),
        fax_notify_date: Set(None),
        fax_retry_count: Set(Some(0)),
        fax_accountcode: Set(None),
        fax_command: Set(None),
        fax_response: Set(None),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "fax_queue_uuid": qid,
            "fax_uuid": id,
            "destination": dest,
            "status": "waiting",
        })),
    )
        .into_response()
}

// ---- fax_queue (legacy submit path + full CRUD) ----
async fn submit_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_queue_add") {
        return (s, j).into_response();
    }
    let ext = payload
        .get("fax_extension")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let dest = payload
        .get("destination")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if ext.is_empty() || dest.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "fax_extension/destination required"})),
        )
            .into_response();
    }
    insert_queue_job(&claims, &payload).await
}

async fn insert_queue_job(
    claims: &pbx_core::auth::Claims,
    payload: &serde_json::Value,
) -> axum::response::Response {
    let ext = payload
        .get("fax_extension")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_owned();
    let dest = payload
        .get("destination")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_owned();
    let str_opt = |k: &str| {
        payload
            .get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.to_owned())
    };
    let id = crate::routing_db::new_uuid();
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(claims);
    let am = fax_queue::ActiveModel {
        fax_queue_uuid: Set(id.clone()),
        domain_id: Set(domain),
        fax_uuid: Set(str_opt("fax_uuid")),
        fax_extension: Set(ext.clone()),
        destination: Set(dest.clone()),
        status: Set(str_opt("status").unwrap_or_else(|| "waiting".to_owned())),
        origination_uuid: Set(str_opt("origination_uuid")),
        fax_log_uuid: Set(str_opt("fax_log_uuid")),
        fax_date: Set(str_opt("fax_date")),
        hostname: Set(str_opt("hostname")),
        fax_caller_id_name: Set(str_opt("fax_caller_id_name")),
        fax_caller_id_number: Set(str_opt("fax_caller_id_number")),
        fax_recipient: Set(str_opt("fax_recipient")),
        fax_number: Set(Some(str_opt("fax_number").unwrap_or_else(|| dest.clone()))),
        fax_prefix: Set(str_opt("fax_prefix")),
        fax_email_address: Set(str_opt("fax_email_address")),
        fax_file: Set(str_opt("fax_file")),
        fax_retry_date: Set(str_opt("fax_retry_date")),
        fax_notify_sent: Set(payload.get("fax_notify_sent").and_then(|v| v.as_bool())),
        fax_notify_date: Set(str_opt("fax_notify_date")),
        fax_retry_count: Set(payload
            .get("fax_retry_count")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32)
            .or(Some(0))),
        fax_accountcode: Set(str_opt("fax_accountcode")),
        fax_command: Set(str_opt("fax_command")),
        fax_response: Set(str_opt("fax_response")),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(FaxQueueOut {
            fax_queue_uuid: id,
            fax_extension: ext.to_owned(),
            destination: dest.to_owned(),
            status: "waiting".to_owned(),
        })),
    )
        .into_response()
}

async fn list_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_queue_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "fax_queue_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/fax/queue", page, per_page, 0),
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
    let mut sel = fax_queue::Entity::find();
    if !show_all {
        sel = sel.filter(fax_queue::Column::DomainId.eq(domain));
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_queue).collect();
    (
        crate::paged_headers("/api/v1/fax_queue", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn queue_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<fax_queue::Model> {
    fax_queue::Entity::find_by_id(id.to_owned())
        .filter(fax_queue::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn get_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_queue_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match queue_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_queue(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response(),
    }
}

async fn patch_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_queue_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response();
    };
    let mut am: fax_queue::ActiveModel = r.into();
    if let Some(v) = payload.get("status").and_then(|v| v.as_str()) {
        am.status = Set(v.to_owned());
    }
    if let Some(v) = payload.get("destination").and_then(|v| v.as_str()) {
        am.destination = Set(v.to_owned());
    }
    for key in [
        "fax_uuid",
        "fax_extension",
        "origination_uuid",
        "fax_log_uuid",
        "fax_date",
        "hostname",
        "fax_caller_id_name",
        "fax_caller_id_number",
        "fax_recipient",
        "fax_number",
        "fax_prefix",
        "fax_email_address",
        "fax_file",
        "fax_retry_date",
        "fax_notify_date",
        "fax_accountcode",
        "fax_command",
        "fax_response",
    ] {
        if let Some(v) = payload.get(key) {
            if v.is_null() {
                match key {
                    "fax_uuid" => am.fax_uuid = Set(None),
                    "origination_uuid" => am.origination_uuid = Set(None),
                    "fax_log_uuid" => am.fax_log_uuid = Set(None),
                    "fax_date" => am.fax_date = Set(None),
                    "hostname" => am.hostname = Set(None),
                    "fax_caller_id_name" => am.fax_caller_id_name = Set(None),
                    "fax_caller_id_number" => am.fax_caller_id_number = Set(None),
                    "fax_recipient" => am.fax_recipient = Set(None),
                    "fax_number" => am.fax_number = Set(None),
                    "fax_prefix" => am.fax_prefix = Set(None),
                    "fax_email_address" => am.fax_email_address = Set(None),
                    "fax_file" => am.fax_file = Set(None),
                    "fax_retry_date" => am.fax_retry_date = Set(None),
                    "fax_notify_date" => am.fax_notify_date = Set(None),
                    "fax_accountcode" => am.fax_accountcode = Set(None),
                    "fax_command" => am.fax_command = Set(None),
                    _ => am.fax_response = Set(None),
                }
            } else if let Some(s) = v.as_str() {
                let owned = Some(s.to_owned());
                match key {
                    "fax_uuid" => am.fax_uuid = Set(owned),
                    "fax_extension" => am.fax_extension = Set(s.to_owned()),
                    "origination_uuid" => am.origination_uuid = Set(owned),
                    "fax_log_uuid" => am.fax_log_uuid = Set(owned),
                    "fax_date" => am.fax_date = Set(owned),
                    "hostname" => am.hostname = Set(owned),
                    "fax_caller_id_name" => am.fax_caller_id_name = Set(owned),
                    "fax_caller_id_number" => am.fax_caller_id_number = Set(owned),
                    "fax_recipient" => am.fax_recipient = Set(owned),
                    "fax_number" => am.fax_number = Set(owned),
                    "fax_prefix" => am.fax_prefix = Set(owned),
                    "fax_email_address" => am.fax_email_address = Set(owned),
                    "fax_file" => am.fax_file = Set(owned),
                    "fax_retry_date" => am.fax_retry_date = Set(owned),
                    "fax_notify_date" => am.fax_notify_date = Set(owned),
                    "fax_accountcode" => am.fax_accountcode = Set(owned),
                    "fax_command" => am.fax_command = Set(owned),
                    _ => am.fax_response = Set(owned),
                }
            }
        }
    }
    if let Some(v) = payload.get("fax_notify_sent").and_then(|v| v.as_bool()) {
        am.fax_notify_sent = Set(Some(v));
    }
    if let Some(v) = payload.get("fax_retry_count").and_then(|v| v.as_i64()) {
        am.fax_retry_count = Set(Some(v as i32));
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_queue(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_queue_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response();
    };
    let am: fax_queue::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn status_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match queue_or_404(db, domain, &id).await {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "fax_queue_uuid": r.fax_queue_uuid,
                "status": r.status,
                "fax_extension": r.fax_extension,
                "destination": r.destination,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response(),
    }
}

async fn retry_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fax_queue_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response();
    };
    let mut am: fax_queue::ActiveModel = r.into();
    am.status = Set("waiting".to_owned());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "fax_queue_uuid": r.fax_queue_uuid,
                "status": r.status,
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

pub fn fax_router() -> Router {
    Router::new()
        .route("/api/v1/fax/health", axum::routing::get(health_fax))
        .route("/api/v1/fax", post(create_fax).get(list_fax))
        .route("/api/v1/fax/", post(create_fax).get(list_fax))
        .route(
            "/api/v1/fax/{id}",
            axum::routing::get(get_fax)
                .patch(patch_fax)
                .delete(delete_fax),
        )
        .route(
            "/api/v1/fax/{id}/files",
            axum::routing::get(list_files).post(upload_file),
        )
        .route("/api/v1/fax/{id}/logs", axum::routing::get(list_logs))
        .route("/api/v1/fax/{id}/users", axum::routing::get(list_users))
        .route("/api/v1/fax/{id}/send", post(send_fax))
        .route("/api/v1/fax/queue", post(submit_queue).get(list_queue))
        .route("/api/v1/fax/queue/", post(submit_queue).get(list_queue))
        .route("/api/v1/fax_queue", post(submit_queue).get(list_queue))
        .route("/api/v1/fax_queue/", post(submit_queue).get(list_queue))
        .route(
            "/api/v1/fax/queue/{id}",
            axum::routing::get(get_queue)
                .patch(patch_queue)
                .delete(delete_queue),
        )
        .route(
            "/api/v1/fax_queue/{id}",
            axum::routing::get(get_queue)
                .patch(patch_queue)
                .delete(delete_queue),
        )
        .route(
            "/api/v1/fax_queue/{id}/status",
            axum::routing::get(status_queue),
        )
        .route("/api/v1/fax_queue/{id}/retry", post(retry_queue))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
