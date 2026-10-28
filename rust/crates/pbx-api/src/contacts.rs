//! Contacts router — mirrors `app/contacts/router.py` core CRUD.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{contacts}`.
//! Tenant-guarded; list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::{
    contact_addresses, contact_emails, contact_notes, contact_phones, contact_urls, contacts,
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
    Json(payload): Json<ContactCreate>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() || payload.number.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name and number required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = contacts::ActiveModel {
        contact_uuid: Set(id),
        domain_id: Set(domain),
        name: Set(payload.name.trim().to_owned()),
        number: Set(payload.number.trim().to_owned()),
        email: Set(payload.email.clone()),
        organization: Set(payload.organization.clone()),
        created_at: Set(now),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "contact_uuid": id,
            "domain_id": domain,
            "name": payload.name,
            "number": payload.number,
            "email": payload.email,
            "organization": payload.organization,
        })),
    )
}

async fn list_contacts(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = contacts::Entity::find()
        .filter(contacts::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "contact_uuid": r.contact_uuid,
                "domain_id": r.domain_id,
                "name": r.name,
                "number": r.number,
                "email": r.email,
                "organization": r.organization,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = contacts::Entity::find_by_id(id)
        .filter(contacts::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "contact_uuid": r.contact_uuid,
                "domain_id": r.domain_id,
                "name": r.name,
                "number": r.number,
                "email": r.email,
                "organization": r.organization,
            })),
        )
            .into_response(),
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
    Json(payload): Json<ContactPatch>,
) -> impl IntoResponse {
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
    if let Some(n) = payload.name {
        am.name = Set(n);
    }
    if let Some(n) = payload.number {
        am.number = Set(n);
    }
    if payload.email.is_some() {
        am.email = Set(payload.email);
    }
    if payload.organization.is_some() {
        am.organization = Set(payload.organization);
    }
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "contact_uuid": r.contact_uuid,
                "name": r.name,
                "number": r.number,
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

async fn delete_contact(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
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
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!(null))).into_response(),
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
    let (per_page, offset) = pq.normalized();
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
    let rows = contact_addresses::Entity::find()
        .filter(contact_addresses::Column::ContactUuid.eq(id))
        .filter(contact_addresses::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows.iter().skip(offset as usize).take(per_page as usize).map(|r| {
        serde_json::json!({"contact_address_uuid": r.contact_address_uuid, "contact_uuid": r.contact_uuid, "address_label": r.address_label, "address_street": r.address_street, "address_locality": r.address_locality})
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
    let am = contact_addresses::ActiveModel {
        contact_address_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        address_label: Set(payload
            .get("address_label")
            .or_else(|| payload.get("label"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        address_street: Set(payload
            .get("address_street")
            .or_else(|| payload.get("value"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        address_locality: Set(payload
            .get("address_locality")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"contact_address_uuid": cid, "contact_uuid": id})),
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
    let (per_page, offset) = pq.normalized();
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
    let rows = contact_phones::Entity::find()
        .filter(contact_phones::Column::ContactUuid.eq(id))
        .filter(contact_phones::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows.iter().skip(offset as usize).take(per_page as usize).map(|r| {
        serde_json::json!({"contact_phone_uuid": r.contact_phone_uuid, "contact_uuid": r.contact_uuid, "phone_label": r.phone_label, "phone_number": r.phone_number})
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
    let am = contact_phones::ActiveModel {
        contact_phone_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        phone_label: Set(payload
            .get("phone_label")
            .or_else(|| payload.get("label"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        phone_number: Set(number.to_owned()),
        phone_primary: Set(false),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"contact_phone_uuid": cid, "contact_uuid": id, "phone_number": number}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response(),
    }
}

async fn list_emails(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
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
    let rows = contact_emails::Entity::find()
        .filter(contact_emails::Column::ContactUuid.eq(id))
        .filter(contact_emails::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows.iter().skip(offset as usize).take(per_page as usize).map(|r| {
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
    let (per_page, offset) = pq.normalized();
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
    let rows = contact_urls::Entity::find()
        .filter(contact_urls::Column::ContactUuid.eq(id))
        .filter(contact_urls::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows.iter().skip(offset as usize).take(per_page as usize).map(|r| {
        serde_json::json!({"contact_url_uuid": r.contact_url_uuid, "contact_uuid": r.contact_uuid, "url_label": r.url_label, "url_address": r.url_address})
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
    let am = contact_urls::ActiveModel {
        contact_url_uuid: Set(cid),
        domain_id: Set(domain),
        contact_uuid: Set(id),
        url_label: Set(payload
            .get("url_label")
            .or_else(|| payload.get("label"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        url_address: Set(addr.to_owned()),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"contact_url_uuid": cid, "contact_uuid": id, "url_address": addr}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response(),
    }
}

async fn list_notes(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
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
    let rows = contact_notes::Entity::find()
        .filter(contact_notes::Column::ContactUuid.eq(id))
        .filter(contact_notes::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows.iter().skip(offset as usize).take(per_page as usize).map(|r| {
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
