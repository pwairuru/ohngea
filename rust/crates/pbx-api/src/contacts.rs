//! Contacts router — mirrors `app/contacts/router.py` core CRUD.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{contacts}`.
//! Tenant-guarded; list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::{
    contact_addresses, contact_attachments, contact_emails, contact_groups, contact_notes,
    contact_phones, contact_relations, contact_settings, contact_times, contact_urls,
    contact_users, contacts,
};

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct ContactCreate {
    name: String,
    number: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    organization: Option<String>,
}

fn sstr(p: &serde_json::Value, keys: &[&str]) -> Option<String> {
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

fn ser_contact(r: &contacts::Model) -> serde_json::Value {
    serde_json::json!({
        "contact_uuid": r.contact_uuid,
        "domain_id": r.domain_id,
        "name": r.name,
        "number": r.number,
        "email": r.email,
        "organization": r.contact_organization.clone().or_else(|| r.organization.clone()),
        "contact_organization": r.contact_organization.clone().or_else(|| r.organization.clone()),
        "contact_type": r.contact_type,
        "contact_name_prefix": r.contact_name_prefix,
        "contact_name_given": r.contact_name_given,
        "contact_name_middle": r.contact_name_middle,
        "contact_name_family": r.contact_name_family,
        "contact_name_suffix": r.contact_name_suffix,
        "contact_nickname": r.contact_nickname,
        "contact_title": r.contact_title,
        "contact_role": r.contact_role,
        "contact_category": r.contact_category,
        "contact_url": r.contact_url,
        "contact_time_zone": r.contact_time_zone,
        "contact_note": r.contact_note,
    })
}

#[derive(Debug, serde::Deserialize)]
struct ContactPatch {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    number: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    organization: Option<String>,
}

// ---- handlers ----
async fn create_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    let name = sstr(&payload, &["name"]).unwrap_or_default();
    let number = sstr(&payload, &["number"]).unwrap_or_default();
    if name.trim().is_empty() || number.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name and number required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let org = sstr(&payload, &["contact_organization", "organization"]);
    let am = contacts::ActiveModel {
        contact_uuid: Set(id),
        domain_id: Set(domain),
        name: Set(name.trim().to_owned()),
        number: Set(number.trim().to_owned()),
        email: Set(sstr(&payload, &["email"])),
        organization: Set(org.clone()),
        contact_organization: Set(org.clone()),
        contact_type: Set(sstr(&payload, &["contact_type"])),
        contact_parent_uuid: Set(None),
        contact_name_prefix: Set(sstr(&payload, &["contact_name_prefix"])),
        contact_name_given: Set(sstr(&payload, &["contact_name_given"])),
        contact_name_middle: Set(sstr(&payload, &["contact_name_middle"])),
        contact_name_family: Set(sstr(&payload, &["contact_name_family"])),
        contact_name_suffix: Set(sstr(&payload, &["contact_name_suffix"])),
        contact_nickname: Set(sstr(&payload, &["contact_nickname"])),
        contact_title: Set(sstr(&payload, &["contact_title"])),
        contact_role: Set(sstr(&payload, &["contact_role"])),
        contact_category: Set(sstr(&payload, &["contact_category"])),
        contact_url: Set(sstr(&payload, &["contact_url"])),
        contact_time_zone: Set(sstr(&payload, &["contact_time_zone"])),
        contact_note: Set(sstr(&payload, &["contact_note"])),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_contact(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_contacts(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "contact_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut sel = contacts::Entity::find();
    if !show_all {
        sel = sel.filter(contacts::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_contact).collect();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn get_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = contacts::Entity::find_by_id(id)
        .filter(contacts::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (StatusCode::OK, Json(ser_contact(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = contacts::Entity::find_by_id(id)
        .filter(contacts::Column::DomainId.eq(domain))
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
    let mut am: contacts::ActiveModel = r.into();
    if let Some(n) = sstr(&payload, &["name"]) {
        am.name = Set(n);
    }
    if let Some(n) = sstr(&payload, &["number"]) {
        am.number = Set(n);
    }
    if let Some(e) = sstr(&payload, &["email"]) {
        am.email = Set(Some(e));
    }
    if let Some(o) = sstr(&payload, &["contact_organization", "organization"]) {
        am.organization = Set(Some(o.clone()));
        // contact_organization is the PHP canonical for organizations
        // (v_contacts.contact_organization); keep both in sync.
        am.contact_organization = Set(Some(o));
    }
    for (keys, slot) in [
        (&["contact_type"][..], &mut am.contact_type),
        (&["contact_name_prefix"][..], &mut am.contact_name_prefix),
        (&["contact_name_given"][..], &mut am.contact_name_given),
        (&["contact_name_middle"][..], &mut am.contact_name_middle),
        (&["contact_name_family"][..], &mut am.contact_name_family),
        (&["contact_name_suffix"][..], &mut am.contact_name_suffix),
        (&["contact_nickname"][..], &mut am.contact_nickname),
        (&["contact_title"][..], &mut am.contact_title),
        (&["contact_role"][..], &mut am.contact_role),
        (&["contact_category"][..], &mut am.contact_category),
        (&["contact_url"][..], &mut am.contact_url),
        (&["contact_time_zone"][..], &mut am.contact_time_zone),
        (&["contact_note"][..], &mut am.contact_note),
    ] {
        if let Some(v) = sstr(&payload, keys) {
            *slot = Set(Some(v));
        }
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_contact(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = contacts::Entity::find_by_id(id)
        .filter(contacts::Column::DomainId.eq(domain))
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
    // cascade: child tables have no DB-level cascade, remove explicitly.
    for a in contact_addresses::Entity::find()
        .filter(contact_addresses::Column::ContactUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let aam: contact_addresses::ActiveModel = a.into();
        let _ = aam.delete(db).await;
    }
    for p in contact_phones::Entity::find()
        .filter(contact_phones::Column::ContactUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let pam: contact_phones::ActiveModel = p.into();
        let _ = pam.delete(db).await;
    }
    for e in contact_emails::Entity::find()
        .filter(contact_emails::Column::ContactUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let eam: contact_emails::ActiveModel = e.into();
        let _ = eam.delete(db).await;
    }
    for u in contact_urls::Entity::find()
        .filter(contact_urls::Column::ContactUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let uam: contact_urls::ActiveModel = u.into();
        let _ = uam.delete(db).await;
    }
    for n in contact_notes::Entity::find()
        .filter(contact_notes::Column::ContactUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default()
    {
        let nam: contact_notes::ActiveModel = n.into();
        let _ = nam.delete(db).await;
    }
    let am: contacts::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn contact_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &uuid::Uuid,
) -> Option<contacts::Model> {
    contacts::Entity::find_by_id(*id)
        .filter(contacts::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

// ---- link-table children (attachments/groups/relations/settings/times/users) ----
fn link_opt_str(payload: &serde_json::Value, key: &str) -> Option<String> {
    payload.get(key).and_then(|v| v.as_str()).map(str::to_owned)
}

fn link_opt_uuid(payload: &serde_json::Value, key: &str) -> Option<uuid::Uuid> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<uuid::Uuid>().ok())
}

async fn list_link_child(
    claims: Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, child)): axum::extract::Path<(uuid::Uuid, String)>,
    pq: axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let name = child.clone();
    let data: Vec<serde_json::Value> = match name.as_str() {
        "attachments" => {
            let rows = contact_attachments::Entity::find()
                .filter(contact_attachments::Column::ContactUuid.eq(id))
                .filter(contact_attachments::Column::DomainId.eq(domain))
                .all(db)
                .await
                .unwrap_or_default();
            rows.into_iter().map(|r| serde_json::json!({"contact_attachment_uuid": r.contact_attachment_uuid, "contact_uuid": r.contact_uuid, "attachment_filename": r.attachment_filename, "attachment_primary": r.attachment_primary, "attachment_description": r.attachment_description})).collect()
        }
        "groups" => {
            let rows = contact_groups::Entity::find()
                .filter(contact_groups::Column::ContactUuid.eq(id))
                .filter(contact_groups::Column::DomainId.eq(domain))
                .all(db)
                .await
                .unwrap_or_default();
            rows.into_iter().map(|r| serde_json::json!({"contact_group_uuid": r.contact_group_uuid, "contact_uuid": r.contact_uuid, "group_uuid": r.group_uuid})).collect()
        }
        "relations" => {
            let rows = contact_relations::Entity::find()
                .filter(contact_relations::Column::ContactUuid.eq(id))
                .filter(contact_relations::Column::DomainId.eq(domain))
                .all(db)
                .await
                .unwrap_or_default();
            rows.into_iter().map(|r| serde_json::json!({"contact_relation_uuid": r.contact_relation_uuid, "contact_uuid": r.contact_uuid, "relation_label": r.relation_label, "relation_label_custom": r.relation_label, "relation_contact_uuid": r.relation_contact_uuid, "relation_reciprocal": false, "relation_reciprocal_label": null, "relation_reciprocal_label_custom": null})).collect()
        }
        "settings" => {
            let rows = contact_settings::Entity::find()
                .filter(contact_settings::Column::ContactUuid.eq(id))
                .filter(contact_settings::Column::DomainId.eq(domain))
                .all(db)
                .await
                .unwrap_or_default();
            rows.into_iter().map(|r| serde_json::json!({"contact_setting_uuid": r.contact_setting_uuid, "contact_uuid": r.contact_uuid, "contact_setting_category": r.contact_setting_category, "contact_setting_name": r.contact_setting_name, "contact_setting_value": r.contact_setting_value, "contact_setting_enabled": r.contact_setting_enabled})).collect()
        }
        "times" => {
            let rows = contact_times::Entity::find()
                .filter(contact_times::Column::ContactUuid.eq(id))
                .filter(contact_times::Column::DomainId.eq(domain))
                .all(db)
                .await
                .unwrap_or_default();
            rows.into_iter().map(|r| serde_json::json!({"contact_time_uuid": r.contact_time_uuid, "contact_uuid": r.contact_uuid, "user_uuid": r.user_uuid, "time_start": r.time_start, "time_stop": r.time_stop, "time_description": r.time_description})).collect()
        }
        "users" => {
            let rows = contact_users::Entity::find()
                .filter(contact_users::Column::ContactUuid.eq(id))
                .filter(contact_users::Column::DomainId.eq(domain))
                .all(db)
                .await
                .unwrap_or_default();
            rows.into_iter().map(|r| serde_json::json!({"contact_user_uuid": r.contact_user_uuid, "contact_uuid": r.contact_uuid, "user_uuid": r.user_uuid})).collect()
        }
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"detail": "unknown child"})),
            )
                .into_response()
        }
    };
    let total = data.len() as u64;
    let start = ((page.saturating_sub(1)) * per_page) as usize;
    let page_data: Vec<serde_json::Value> = data
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .collect();
    (
        crate::paged_headers(
            &format!("/api/v1/contacts/{id}/{name}"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(page_data, total, page, per_page)),
    )
        .into_response()
}

async fn create_link_child(
    claims: Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, child)): axum::extract::Path<(uuid::Uuid, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let name = child.clone();
    let out: Result<serde_json::Value, String> = match name.as_str() {
        "attachments" => {
            let cid = uuid::Uuid::now_v7();
            let am = contact_attachments::ActiveModel {
                contact_attachment_uuid: Set(cid),
                domain_id: Set(domain),
                contact_uuid: Set(id),
                attachment_primary: Set(payload
                    .get("attachment_primary")
                    .and_then(|v| v.as_bool())),
                attachment_filename: Set(link_opt_str(&payload, "attachment_filename")),
                attachment_content: Set(link_opt_str(&payload, "attachment_content")),
                attachment_description: Set(link_opt_str(&payload, "attachment_description")),
                created_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => Ok(serde_json::json!({"contact_attachment_uuid": cid})),
                Err(e) => Err(format!("insert failed: {e}")),
            }
        }
        "groups" => {
            let cid = uuid::Uuid::now_v7();
            let am = contact_groups::ActiveModel {
                contact_group_uuid: Set(cid),
                domain_id: Set(domain),
                contact_uuid: Set(id),
                group_uuid: Set(link_opt_uuid(&payload, "group_uuid")),
                created_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => Ok(serde_json::json!({"contact_group_uuid": cid})),
                Err(e) => Err(format!("insert failed: {e}")),
            }
        }
        "relations" => {
            let cid = uuid::Uuid::now_v7();
            let label = link_opt_str(&payload, "relation_label_custom")
                .or_else(|| link_opt_str(&payload, "relation_label"));
            let am = contact_relations::ActiveModel {
                contact_relation_uuid: Set(cid),
                domain_id: Set(domain),
                contact_uuid: Set(id),
                relation_label: Set(label.clone()),
                relation_contact_uuid: Set(link_opt_uuid(&payload, "relation_contact_uuid")),
                created_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => {
                    // PHP reciprocal parity: optionally create inverse row.
                    let recip = payload
                        .get("relation_reciprocal")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        || payload
                            .get("relation_reciprocal")
                            .and_then(|v| v.as_str())
                            .map(|s| s == "true" || s == "1")
                            .unwrap_or(false);
                    if recip {
                        if let Some(other) = link_opt_uuid(&payload, "relation_contact_uuid") {
                            let rlabel = link_opt_str(&payload, "relation_reciprocal_label_custom")
                                .or_else(|| link_opt_str(&payload, "relation_reciprocal_label"))
                                .or_else(|| label.clone());
                            let rid = uuid::Uuid::now_v7();
                            let ram = contact_relations::ActiveModel {
                                contact_relation_uuid: Set(rid),
                                domain_id: Set(domain),
                                contact_uuid: Set(other),
                                relation_label: Set(rlabel),
                                relation_contact_uuid: Set(Some(id)),
                                created_at: Set(now),
                            };
                            let _ = ram.insert(db).await;
                        }
                    }
                    Ok(
                        serde_json::json!({"contact_relation_uuid": cid, "relation_reciprocal": payload.get("relation_reciprocal"), "relation_reciprocal_label": payload.get("relation_reciprocal_label"), "relation_reciprocal_label_custom": payload.get("relation_reciprocal_label_custom")}),
                    )
                }
                Err(e) => Err(format!("insert failed: {e}")),
            }
        }
        "settings" => {
            let cid = uuid::Uuid::now_v7();
            let am = contact_settings::ActiveModel {
                contact_setting_uuid: Set(cid),
                contact_uuid: Set(id),
                domain_id: Set(domain),
                contact_setting_category: Set(link_opt_str(&payload, "contact_setting_category")),
                contact_setting_subcategory: Set(link_opt_str(
                    &payload,
                    "contact_setting_subcategory",
                )),
                contact_setting_name: Set(link_opt_str(&payload, "contact_setting_name")),
                contact_setting_value: Set(link_opt_str(&payload, "contact_setting_value")),
                contact_setting_order: Set(payload
                    .get("contact_setting_order")
                    .and_then(|v| v.as_i64())
                    .map(|n| n as i32)),
                contact_setting_enabled: Set(payload
                    .get("contact_setting_enabled")
                    .and_then(|v| v.as_bool())),
                contact_setting_description: Set(link_opt_str(
                    &payload,
                    "contact_setting_description",
                )),
                created_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => Ok(serde_json::json!({"contact_setting_uuid": cid})),
                Err(e) => Err(format!("insert failed: {e}")),
            }
        }
        "times" => {
            let cid = uuid::Uuid::now_v7();
            let am = contact_times::ActiveModel {
                contact_time_uuid: Set(cid),
                domain_id: Set(domain),
                contact_uuid: Set(id),
                user_uuid: Set(link_opt_uuid(&payload, "user_uuid")),
                time_start: Set(link_opt_str(&payload, "time_start")),
                time_stop: Set(link_opt_str(&payload, "time_stop")),
                time_description: Set(link_opt_str(&payload, "time_description")),
                created_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => Ok(serde_json::json!({"contact_time_uuid": cid})),
                Err(e) => Err(format!("insert failed: {e}")),
            }
        }
        "users" => {
            let cid = uuid::Uuid::now_v7();
            let am = contact_users::ActiveModel {
                contact_user_uuid: Set(cid),
                domain_id: Set(domain),
                contact_uuid: Set(id),
                user_uuid: Set(link_opt_uuid(&payload, "user_uuid")),
                created_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => Ok(serde_json::json!({"contact_user_uuid": cid})),
                Err(e) => Err(format!("insert failed: {e}")),
            }
        }
        _ => Err("unknown child".to_owned()),
    };
    match out {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(detail) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": detail})),
        )
            .into_response(),
    }
}

pub fn contacts_router() -> Router {
    Router::new()
        .route("/api/v1/contacts", post(create_contact).get(list_contacts))
        .route("/api/v1/contacts/", post(create_contact).get(list_contacts))
        .route(
            "/api/v1/contacts/{id}",
            axum::routing::get(get_contact)
                .patch(patch_contact)
                .delete(delete_contact),
        )
        .route(
            "/api/v1/contacts/{id}/addresses",
            axum::routing::get(list_addresses).post(create_address),
        )
        .route(
            "/api/v1/contacts/{id}/phones",
            axum::routing::get(list_phones).post(create_phone),
        )
        .route(
            "/api/v1/contacts/{id}/emails",
            axum::routing::get(list_emails).post(create_email),
        )
        .route(
            "/api/v1/contacts/{id}/urls",
            axum::routing::get(list_urls).post(create_url),
        )
        .route(
            "/api/v1/contacts/{id}/notes",
            axum::routing::get(list_notes).post(create_note),
        )
        .route(
            "/api/v1/contacts/{id}/{child}",
            axum::routing::get(list_child_generic).post(create_child_generic),
        )
        .route(
            "/api/v1/contacts/{id}/{child}/{cid}",
            axum::routing::delete(delete_child_generic),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

// ---- child routes (parity: app/contacts/router.py addresses|phones|emails|urls|notes) ----
async fn list_addresses(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = contact_addresses::Entity::find()
        .filter(contact_addresses::Column::ContactUuid.eq(id))
        .filter(contact_addresses::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"contact_address_uuid": r.contact_address_uuid, "contact_uuid": r.contact_uuid, "address_label": r.address_label, "address_label_custom": r.address_label, "address_street": r.address_street, "address_extended": r.address_extended, "address_community": r.address_community, "address_locality": r.address_locality, "address_region": r.address_region, "address_postal_code": r.address_postal_code, "address_country": r.address_country, "address_type": r.address_type, "address_primary": r.address_primary, "address_latitude": r.address_latitude, "address_longitude": r.address_longitude, "address_description": r.address_description})
    }).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/contacts/{id}/addresses"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_address(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let cid = uuid::Uuid::now_v7();
    let sval = |keys: &[&str]| -> Option<String> {
        for k in keys {
            if let Some(v) = payload.get(*k) {
                if let Some(s) = v.as_str() {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_owned());
                    }
                }
            }
        }
        None
    };
    let bval = |keys: &[&str], dflt: bool| {
        for k in keys {
            if let Some(v) = payload.get(*k) {
                if let Some(b) = v.as_bool() {
                    return b;
                }
                if let Some(n) = v.as_i64() {
                    return n != 0;
                }
            }
        }
        dflt
    };
    let am = contact_addresses::ActiveModel {
        contact_address_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        address_label: Set(sval(&["address_label_custom", "address_label", "label"])),
        address_street: Set(sval(&["address_street", "value"])),
        address_extended: Set(sval(&["address_extended"])),
        address_community: Set(sval(&["address_community"])),
        address_locality: Set(sval(&["address_locality", "locality"])),
        address_region: Set(sval(&["address_region", "region"])),
        address_postal_code: Set(sval(&["address_postal_code", "postal_code"])),
        address_country: Set(sval(&["address_country", "country"])),
        address_type: Set(sval(&["address_type", "type"])),
        address_primary: Set(bval(&["address_primary", "primary"], false)),
        address_latitude: Set(sval(&["address_latitude", "latitude"])),
        address_longitude: Set(sval(&["address_longitude", "longitude"])),
        address_description: Set(sval(&["address_description", "description"])),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"contact_address_uuid": r.contact_address_uuid, "contact_uuid": r.contact_uuid, "address_street": r.address_street, "address_label": r.address_label, "address_label_custom": r.address_label, "address_latitude": r.address_latitude, "address_longitude": r.address_longitude})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_phones(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = contact_phones::Entity::find()
        .filter(contact_phones::Column::ContactUuid.eq(id))
        .filter(contact_phones::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"contact_phone_uuid": r.contact_phone_uuid, "contact_uuid": r.contact_uuid, "phone_label": r.phone_label, "phone_label_custom": r.phone_label, "phone_number": r.phone_number, "phone_country_code": r.phone_country_code, "phone_extension": r.phone_extension, "phone_speed_dial": r.phone_speed_dial, "phone_description": r.phone_description, "phone_primary": r.phone_primary, "phone_type_voice": r.phone_type_voice, "phone_type_fax": r.phone_type_fax, "phone_type_video": r.phone_type_video, "phone_type_text": r.phone_type_text})
    }).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/contacts/{id}/phones"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_phone(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let number = payload
        .get("phone_number")
        .or_else(|| payload.get("value"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if number.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "phone_number required"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let cid = uuid::Uuid::now_v7();
    let bval = |keys: &[&str], dflt: bool| {
        for k in keys {
            if let Some(v) = payload.get(*k) {
                if let Some(b) = v.as_bool() {
                    return b;
                }
                if let Some(n) = v.as_i64() {
                    return n != 0;
                }
            }
        }
        dflt
    };
    let sval = |keys: &[&str]| -> Option<String> {
        for k in keys {
            if let Some(v) = payload.get(*k) {
                if let Some(s) = v.as_str() {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_owned());
                    }
                }
            }
        }
        None
    };
    let am = contact_phones::ActiveModel {
        contact_phone_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        phone_label: Set(sval(&["phone_label_custom", "phone_label", "label"])),
        phone_number: Set(number.to_owned()),
        phone_country_code: Set(sval(&["phone_country_code"])),
        phone_extension: Set(sval(&["phone_extension"])),
        phone_speed_dial: Set(sval(&["phone_speed_dial"])),
        phone_description: Set(sval(&["phone_description", "description"])),
        phone_primary: Set(bval(&["phone_primary", "primary"], false)),
        phone_type_voice: Set(bval(&["phone_type_voice"], true)),
        phone_type_fax: Set(bval(&["phone_type_fax"], false)),
        phone_type_video: Set(bval(&["phone_type_video"], false)),
        phone_type_text: Set(bval(&["phone_type_text"], false)),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(serde_json::json!({"contact_phone_uuid": r.contact_phone_uuid, "contact_uuid": r.contact_uuid, "phone_number": r.phone_number, "phone_label": r.phone_label, "phone_label_custom": r.phone_label, "phone_country_code": r.phone_country_code, "phone_extension": r.phone_extension, "phone_speed_dial": r.phone_speed_dial, "phone_description": r.phone_description, "phone_primary": r.phone_primary, "phone_type_voice": r.phone_type_voice, "phone_type_fax": r.phone_type_fax, "phone_type_video": r.phone_type_video, "phone_type_text": r.phone_type_text}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response(),
    }
}

async fn list_emails(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = contact_emails::Entity::find()
        .filter(contact_emails::Column::ContactUuid.eq(id))
        .filter(contact_emails::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"contact_email_uuid": r.contact_email_uuid, "contact_uuid": r.contact_uuid, "email_label": r.email_label, "email_address": r.email_address})
    }).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/contacts/{id}/emails"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_email(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let addr = payload
        .get("email_address")
        .or_else(|| payload.get("value"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if addr.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "email_address required"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let cid = uuid::Uuid::now_v7();
    let am = contact_emails::ActiveModel {
        contact_email_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        email_label: Set(payload
            .get("email_label")
            .or_else(|| payload.get("label"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        email_address: Set(addr.to_owned()),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"contact_email_uuid": cid, "contact_uuid": id, "email_address": addr}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response(),
    }
}

async fn list_urls(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = contact_urls::Entity::find()
        .filter(contact_urls::Column::ContactUuid.eq(id))
        .filter(contact_urls::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"contact_url_uuid": r.contact_url_uuid, "contact_uuid": r.contact_uuid, "url_label": r.url_label, "url_label_custom": r.url_label, "url_address": r.url_address, "url_type": r.url_type, "url_primary": r.url_primary, "url_description": r.url_description})
    }).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/contacts/{id}/urls"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_url(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let addr = payload
        .get("url_address")
        .or_else(|| payload.get("value"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if addr.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "url_address required"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let cid = uuid::Uuid::now_v7();
    let sval = |keys: &[&str]| -> Option<String> {
        for k in keys {
            if let Some(v) = payload.get(*k) {
                if let Some(s) = v.as_str() {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_owned());
                    }
                }
            }
        }
        None
    };
    let bval = |keys: &[&str], dflt: bool| {
        for k in keys {
            if let Some(v) = payload.get(*k) {
                if let Some(b) = v.as_bool() {
                    return b;
                }
                if let Some(n) = v.as_i64() {
                    return n != 0;
                }
            }
        }
        dflt
    };
    let am = contact_urls::ActiveModel {
        contact_url_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        url_label: Set(sval(&["url_label_custom", "url_label", "label"])),
        url_address: Set(addr.to_owned()),
        url_type: Set(sval(&["url_type", "type"])),
        url_primary: Set(bval(&["url_primary", "primary"], false)),
        url_description: Set(sval(&["url_description", "description"])),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(serde_json::json!({"contact_url_uuid": r.contact_url_uuid, "contact_uuid": r.contact_uuid, "url_address": r.url_address, "url_label": r.url_label, "url_label_custom": r.url_label, "url_type": r.url_type, "url_primary": r.url_primary, "url_description": r.url_description}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response(),
    }
}

async fn list_notes(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = contact_notes::Entity::find()
        .filter(contact_notes::Column::ContactUuid.eq(id))
        .filter(contact_notes::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"contact_note_uuid": r.contact_note_uuid, "contact_uuid": r.contact_uuid, "contact_note": r.contact_note})
    }).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/contacts/{id}/notes"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_note(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let note = payload
        .get("contact_note")
        .or_else(|| payload.get("value"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if note.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "contact_note required"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let cid = uuid::Uuid::now_v7();
    let am = contact_notes::ActiveModel {
        contact_note_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        contact_note: Set(note.to_owned()),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"contact_note_uuid": cid, "contact_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

// Generic {child} dispatch (parity: Python generic child routes).
async fn list_child_generic(
    claims: Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, child)): axum::extract::Path<(uuid::Uuid, String)>,
    pq: axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_view") {
        return (s, j).into_response();
    }
    match child.as_str() {
        "addresses" => list_addresses(claims, axum::extract::Path(id), pq)
            .await
            .into_response(),
        "phones" => list_phones(claims, axum::extract::Path(id), pq)
            .await
            .into_response(),
        "emails" => list_emails(claims, axum::extract::Path(id), pq)
            .await
            .into_response(),
        "urls" => list_urls(claims, axum::extract::Path(id), pq)
            .await
            .into_response(),
        "notes" => list_notes(claims, axum::extract::Path(id), pq)
            .await
            .into_response(),
        "attachments" | "groups" | "relations" | "settings" | "times" | "users" => {
            list_link_child(claims, axum::extract::Path((id, child)), pq)
                .await
                .into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "unknown child"})),
        )
            .into_response(),
    }
}

async fn create_child_generic(
    claims: Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, child)): axum::extract::Path<(uuid::Uuid, String)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_add") {
        return (s, j).into_response();
    }
    match child.as_str() {
        "addresses" => create_address(claims, axum::extract::Path(id), Json(payload))
            .await
            .into_response(),
        "phones" => create_phone(claims, axum::extract::Path(id), Json(payload))
            .await
            .into_response(),
        "emails" => create_email(claims, axum::extract::Path(id), Json(payload))
            .await
            .into_response(),
        "urls" => create_url(claims, axum::extract::Path(id), Json(payload))
            .await
            .into_response(),
        "notes" => create_note(claims, axum::extract::Path(id), Json(payload))
            .await
            .into_response(),
        "attachments" | "groups" | "relations" | "settings" | "times" | "users" => {
            create_link_child(claims, axum::extract::Path((id, child)), Json(payload))
                .await
                .into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "unknown child"})),
        )
            .into_response(),
    }
}

async fn delete_child_generic(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, child, cid)): axum::extract::Path<(uuid::Uuid, String, uuid::Uuid)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "contact_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if contact_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let deleted: bool = match child.as_str() {
        "addresses" => {
            let row = contact_addresses::Entity::find_by_id(cid)
                .filter(contact_addresses::Column::ContactUuid.eq(id))
                .filter(contact_addresses::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_addresses::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "phones" => {
            let row = contact_phones::Entity::find_by_id(cid)
                .filter(contact_phones::Column::ContactUuid.eq(id))
                .filter(contact_phones::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_phones::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "emails" => {
            let row = contact_emails::Entity::find_by_id(cid)
                .filter(contact_emails::Column::ContactUuid.eq(id))
                .filter(contact_emails::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_emails::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "urls" => {
            let row = contact_urls::Entity::find_by_id(cid)
                .filter(contact_urls::Column::ContactUuid.eq(id))
                .filter(contact_urls::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_urls::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "notes" => {
            let row = contact_notes::Entity::find_by_id(cid)
                .filter(contact_notes::Column::ContactUuid.eq(id))
                .filter(contact_notes::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_notes::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "attachments" => {
            let row = contact_attachments::Entity::find_by_id(cid)
                .filter(contact_attachments::Column::ContactUuid.eq(id))
                .filter(contact_attachments::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_attachments::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "groups" => {
            let row = contact_groups::Entity::find_by_id(cid)
                .filter(contact_groups::Column::ContactUuid.eq(id))
                .filter(contact_groups::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_groups::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "relations" => {
            let row = contact_relations::Entity::find_by_id(cid)
                .filter(contact_relations::Column::ContactUuid.eq(id))
                .filter(contact_relations::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_relations::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "settings" => {
            let row = contact_settings::Entity::find_by_id(cid)
                .filter(contact_settings::Column::ContactUuid.eq(id))
                .filter(contact_settings::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_settings::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "times" => {
            let row = contact_times::Entity::find_by_id(cid)
                .filter(contact_times::Column::ContactUuid.eq(id))
                .filter(contact_times::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_times::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        "users" => {
            let row = contact_users::Entity::find_by_id(cid)
                .filter(contact_users::Column::ContactUuid.eq(id))
                .filter(contact_users::Column::DomainId.eq(domain))
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(r) => {
                    let am: contact_users::ActiveModel = r.into();
                    am.delete(db).await.is_ok()
                }
                None => false,
            }
        }
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"detail": "unknown child"})),
            )
                .into_response()
        }
    };
    if deleted {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response()
    }
}
