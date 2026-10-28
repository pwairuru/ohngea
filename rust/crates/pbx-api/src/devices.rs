//! Devices router + provision render — mirrors `app/devices/router.py`
//! + `app/devices/provision.py` collapsed-matrix templates.
//!
//! Order: models → schemas → handlers → provision.
//!
//! models: re-exports `pbx_db::entities::{devices, device_lines, domains}`.
//! Tenant-guarded CRUD; SIP line 1 (server_address=user domain, user_id=
//! extension) is created alongside the device so provision render is honest.
//! Public: GET /provision/{mac}.cfg (phones fetch without auth).
//!
//! Templates: yealink + cisco embedded fully (verbatim port of
//! `templates/provision/{yealink,cisco}/base.j2`); all other vendors fall
//! back to a generic stub — full 31-vendor matrix is a later pass.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::{device_lines, devices, domains};

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct DeviceCreate {
    mac: String,
    vendor: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    extension: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

fn valid_mac(m: &str) -> bool {
    let s = m.trim();
    if s.len() != 17 {
        return false;
    }
    let parts: Vec<&str> = s.split([':', '-']).collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn norm_mac(m: &str) -> String {
    m.trim().to_lowercase()
}

// ---- handlers ----
async fn create_device(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<DeviceCreate>,
) -> impl IntoResponse {
    if !valid_mac(&payload.mac) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "mac must match XX:XX:XX:XX:XX:XX"})),
        );
    }
    if payload.vendor.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "vendor required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mac = norm_mac(&payload.mac);
    let vendor = payload.vendor.trim().to_lowercase();
    // MAC unique per tenant → second POST is 409 (test accepts 201/200/409).
    let exists = devices::Entity::find()
        .filter(devices::Column::DomainId.eq(domain))
        .filter(devices::Column::Mac.eq(mac.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": "device exists"})),
        );
    }
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = devices::ActiveModel {
        device_uuid: Set(id),
        mac: Set(mac.clone()),
        vendor: Set(vendor.clone()),
        model: Set(payload.model.clone()),
        label: Set(payload.label.clone()),
        enabled: Set(true),
        domain_id: Set(domain),
        extension_uuid: Set(None),
        device_profile_uuid: Set(None),
        device_address: Set(Some(mac.clone())),
        device_label: Set(payload.label.clone()),
        device_vendor: Set(Some(vendor.clone())),
        device_model: Set(payload.model.clone()),
        device_firmware_version: Set(None),
        device_template: Set(None),
        device_location: Set(None),
        device_serial_number: Set(None),
        device_description: Set(None),
        device_username: Set(None),
        device_password: Set(payload.password.clone()),
        device_provisioned_date: Set(None),
        device_provisioned_ip: Set(None),
        device_provisioned_method: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    // Line 1 carries the SIP identity (server_address = sip domain,
    // user_id = extension) — provision renders from here.
    if payload.extension.is_some() || payload.domain.is_some() {
        let lam = device_lines::ActiveModel {
            device_line_uuid: Set(uuid::Uuid::now_v7()),
            domain_id: Set(domain),
            device_uuid: Set(id),
            line_number: Set("1".to_owned()),
            server_address: Set(payload.domain.clone()),
            server_address_primary: Set(payload.domain.clone()),
            server_address_secondary: Set(None),
            outbound_proxy_primary: Set(None),
            outbound_proxy_secondary: Set(None),
            label: Set(payload.label.clone()),
            display_name: Set(payload.extension.clone()),
            user_id: Set(payload.extension.clone()),
            auth_id: Set(payload.extension.clone()),
            password: Set(payload.password.clone()),
            sip_port: Set(None),
            sip_transport: Set(None),
            register_expires: Set(None),
            shared_line: Set(None),
            enabled: Set(true),
            created_at: Set(now),
        };
        let _ = lam.insert(db).await;
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "device_uuid": id,
            "mac": mac,
            "vendor": vendor,
            "model": payload.model,
            "extension": payload.extension,
            "domain": payload.domain,
            "domain_id": domain,
            "enabled": true,
        })),
    )
}

async fn list_devices(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = devices::Entity::find()
        .filter(devices::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let mut data = Vec::new();
    for r in rows.iter().skip(offset as usize).take(per_page as usize) {
        let line = device_lines::Entity::find()
            .filter(device_lines::Column::DeviceUuid.eq(r.device_uuid))
            .one(db)
            .await
            .unwrap_or(None);
        data.push(serde_json::json!({
            "device_uuid": r.device_uuid,
            "mac": r.mac,
            "vendor": r.vendor,
            "model": r.model,
            "label": r.label,
            "domain_id": r.domain_id,
            "enabled": r.enabled,
            "extension": line.as_ref().and_then(|l| l.user_id.clone()),
            "domain": line.as_ref().and_then(|l| l.server_address.clone()),
        }));
    }
    (
        crate::paged_headers("/api/v1/devices", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn get_device(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = devices::Entity::find_by_id(id)
        .filter(devices::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "device_uuid": r.device_uuid,
                "mac": r.mac,
                "vendor": r.vendor,
                "model": r.model,
                "enabled": r.enabled,
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

// ---- provision ----
// Verbatim logic port of `app/devices/provision.py::render_provision`:
// per-vendor base template, model overrides collapsed, generic fallback.
// Templates embedded (no file I/O): yealink + cisco fully, rest generic.
// Full 31-vendor matrix (templates/provision/*) is a later pass.
fn render_yealink(mac: &str, model: &str, extension: &str, domain: &str, password: &str) -> String {
    let mut out = format!(
        "# Yealink provision — collapsed matrix (model: {model})\n\
         # MAC: {mac}  Model: {model}\n\
         account.1.enable = 1\n\
         account.1.label = {extension}\n\
         account.1.display_name = {extension}\n\
         account.1.sip_server_host = {domain}\n\
         account.1.sip_server_port = 5060\n\
         account.1.user_name = {extension}\n\
         account.1.auth_name = {extension}\n"
    );
    if !password.is_empty() {
        out.push_str(&format!("account.1.password = {password}\n"));
    }
    out.push_str("account.1.sip_transp = 0\n");
    out
}

fn render_cisco(mac: &str, extension: &str, domain: &str) -> String {
    format!(
        "#!version:1.0.0.1\n\
         mac={mac}\n\
         # cisco provision\n\
         # MAC: {mac}\n\
         # Domain: {domain}\n\
         # Extension: {extension}\n\
         <device vendor=\"cisco\" mac=\"{mac}\">\n\
         \x20 <account extension=\"{extension}\" domain=\"{domain}\" />\n\
         </device>\n"
    )
}

fn render_generic(vendor: &str, mac: &str, extension: &str, domain: &str) -> String {
    // Generic fallback (mirrors provision.py fallback render).
    // NOTE: full 31-vendor matrix under templates/provision/* lands in a
    // later pass; add per-vendor render_* fns here as vendors are ported.
    format!(
        "# Provision for {vendor} (generic fallback — vendor template not yet ported)\n\
         mac={mac}\n\
         extension={extension}\n\
         domain={domain}\n"
    )
}

fn render_provision(
    vendor: &str,
    mac: &str,
    model: &str,
    extension: &str,
    domain: &str,
    password: &str,
) -> String {
    match vendor.to_lowercase().as_str() {
        "yealink" => render_yealink(mac, model, extension, domain, password),
        "cisco" => render_cisco(mac, extension, domain),
        _ => render_generic(vendor, mac, extension, domain),
    }
}

async fn get_provision(
    axum::extract::Path(mac_param): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Strip optional .cfg suffix (route also matches bare MAC).
    let want = mac_param
        .strip_suffix(".cfg")
        .unwrap_or(&mac_param)
        .trim()
        .to_lowercase();
    let db = crate::routing_db::conn().await;
    // Public endpoint: search across tenants by MAC (phones carry no JWT).
    let rows = devices::Entity::find().all(db).await.unwrap_or_default();
    let dev = rows.iter().find(|d| d.mac.to_lowercase() == want);
    let Some(dev) = dev else {
        return (
            StatusCode::NOT_FOUND,
            [("content-type", "text/plain")],
            "device not found",
        )
            .into_response();
    };
    let line = device_lines::Entity::find()
        .filter(device_lines::Column::DeviceUuid.eq(dev.device_uuid))
        .one(db)
        .await
        .unwrap_or(None);
    let extension = line
        .as_ref()
        .and_then(|l| l.user_id.clone())
        .or_else(|| line.as_ref().and_then(|l| l.display_name.clone()))
        .unwrap_or_default();
    let sip_domain = line
        .as_ref()
        .and_then(|l| l.server_address.clone())
        .unwrap_or_default();
    // Resolve tenant domain name for context; fall back to stored sip domain.
    let domain_name = domains::Entity::find_by_id(dev.domain_id)
        .one(db)
        .await
        .unwrap_or(None)
        .map(|d| d.domain_name)
        .unwrap_or_default();
    let domain = if sip_domain.is_empty() {
        domain_name
    } else {
        sip_domain
    };
    let password = line
        .as_ref()
        .and_then(|l| l.password.clone())
        .unwrap_or_default();
    let model = dev.model.clone().unwrap_or_default();
    let body = render_provision(
        &dev.vendor,
        &dev.mac,
        &model,
        &extension,
        &domain,
        &password,
    );
    (StatusCode::OK, [("content-type", "text/plain")], body).into_response()
}

pub fn devices_router() -> Router {
    Router::new()
        .route("/api/v1/devices", post(create_device).get(list_devices))
        .route("/api/v1/devices/", post(create_device).get(list_devices))
        .route(
            "/api/v1/devices/{id}",
            axum::routing::get(get_device)
                .patch(patch_device)
                .delete(delete_device),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

async fn patch_device(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = devices::Entity::find_by_id(id)
        .filter(devices::Column::DomainId.eq(domain))
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
    let mut am: devices::ActiveModel = r.into();
    if let Some(mac) = payload.get("mac").and_then(|v| v.as_str()) {
        am.mac = Set(mac.trim().to_lowercase());
    }
    if let Some(vendor) = payload.get("vendor").and_then(|v| v.as_str()) {
        am.vendor = Set(vendor.trim().to_lowercase());
    }
    if let Some(model) = payload.get("model").and_then(|v| v.as_str()) {
        am.model = Set(Some(model.to_owned()));
    }
    if let Some(label) = payload.get("label").and_then(|v| v.as_str()) {
        am.label = Set(Some(label.to_owned()));
        am.device_label = Set(Some(label.to_owned()));
    }
    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(enabled);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "device_uuid": r.device_uuid,
                "mac": r.mac,
                "vendor": r.vendor,
                "model": r.model,
                "enabled": r.enabled,
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

async fn delete_device(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = devices::Entity::find_by_id(id)
        .filter(devices::Column::DomainId.eq(domain))
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
    // cascade: device_lines has no DB-level cascade, remove explicitly.
    let lines = device_lines::Entity::find()
        .filter(device_lines::Column::DeviceUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    for l in lines {
        let lam: device_lines::ActiveModel = l.into();
        let _ = lam.delete(db).await;
    }
    let am: devices::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn provision_router() -> Router {
    // Single param route covers BOTH bare MAC and `.cfg` suffix:
    // axum allows only one param per segment (no "{mac}.cfg" pattern), so
    // get_provision strips an optional ".cfg" suffix in the handler.
    // GET /provision/{mac} + GET /provision/{mac}.cfg (same handler).
    Router::new().route("/provision/{mac}", axum::routing::get(get_provision))
}
