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
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

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
    #[serde(default)]
    device_enabled_date: Option<String>,
    #[serde(default)]
    device_user_uuid: Option<String>,
    #[serde(default)]
    device_uuid_alternate: Option<String>,
    #[serde(default)]
    device_provisioned_agent: Option<String>,
    #[serde(default)]
    device_profile_uuid: Option<String>,
    #[serde(default)]
    device_template: Option<String>,
    #[serde(default)]
    device_location: Option<String>,
    #[serde(default)]
    device_serial_number: Option<String>,
    #[serde(default)]
    device_firmware_version: Option<String>,
    #[serde(default)]
    device_description: Option<String>,
    #[serde(default)]
    device_username: Option<String>,
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_add") {
        return (s, j).into_response();
    }
    if !valid_mac(&payload.mac) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "mac must match XX:XX:XX:XX:XX:XX"})),
        )
            .into_response();
    }
    if payload.vendor.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "vendor required"})),
        )
            .into_response();
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
        )
            .into_response();
    }
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = devices::ActiveModel {
        device_enabled_date: Set(payload.device_enabled_date.as_deref().and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())),
        device_user_uuid: Set(payload.device_user_uuid.as_deref().and_then(|s| s.parse::<uuid::Uuid>().ok())),
        device_uuid_alternate: Set(payload.device_uuid_alternate.clone()),
        device_provisioned_agent: Set(payload.device_provisioned_agent.clone()),
        device_uuid: Set(id),
        mac: Set(mac.clone()),
        vendor: Set(vendor.clone()),
        model: Set(payload.model.clone()),
        label: Set(payload.label.clone()),
        enabled: Set(true),
        domain_id: Set(domain),
        extension_uuid: Set(None),
        device_profile_uuid: Set(payload.device_profile_uuid.as_deref().and_then(|s| s.parse::<uuid::Uuid>().ok())),
        device_address: Set(Some(mac.clone())),
        device_label: Set(payload.label.clone()),
        device_vendor: Set(Some(vendor.clone())),
        device_model: Set(payload.model.clone()),
        device_firmware_version: Set(payload.device_firmware_version.clone()),
        device_template: Set(payload.device_template.clone()),
        device_location: Set(payload.device_location.clone()),
        device_serial_number: Set(payload.device_serial_number.clone()),
        device_description: Set(payload.device_description.clone()),
        device_username: Set(payload.device_username.clone()),
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
        )
            .into_response();
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
            "label": payload.label,
            "extension": payload.extension,
            "domain": payload.domain,
            "domain_id": domain,
            "enabled": true,
            "device_profile_uuid": payload.device_profile_uuid,
            "device_template": payload.device_template,
            "device_location": payload.device_location,
            "device_serial_number": payload.device_serial_number,
            "device_firmware_version": payload.device_firmware_version,
            "device_description": payload.device_description,
            "device_username": payload.device_username,
            "device_password": payload.password,
        })),
    )
        .into_response()
}

async fn list_devices(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "device_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/devices", page, per_page, 0),
                Json(crate::paged_body(Vec::<serde_json::Value>::new(), 0, page, per_page)),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut sel = devices::Entity::find();
    if !show_all {
        sel = sel.filter(devices::Column::DomainId.eq(domain));
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    // One batched child query per page (was: N+1 `device_lines::one()`
    // per device row). First line per device wins, matching old `.one()`.
    let ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.device_uuid).collect();
    let lines = if ids.is_empty() {
        Vec::new()
    } else {
        device_lines::Entity::find()
            .filter(device_lines::Column::DeviceUuid.is_in(ids))
            .all(&txn)
            .await
            .unwrap_or_default()
    };
    let mut first_line: std::collections::HashMap<uuid::Uuid, &device_lines::Model> =
        std::collections::HashMap::new();
    for l in &lines {
        first_line.entry(l.device_uuid).or_insert(l);
    }
    let mut data = Vec::new();
    for r in rows.iter() {
        let line = first_line.get(&r.device_uuid).copied();
        data.push(ser_device(r, line));
    }
    (
        crate::paged_headers("/api/v1/devices", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

fn ser_device(r: &devices::Model, line: Option<&device_lines::Model>) -> serde_json::Value {
    serde_json::json!({
        "device_uuid": r.device_uuid,
        "mac": r.mac,
        "vendor": r.vendor,
        "model": r.model,
        "label": r.label,
        "domain_id": r.domain_id,
        "enabled": r.enabled,
        "extension_uuid": r.extension_uuid,
        "device_profile_uuid": r.device_profile_uuid,
        "device_address": r.device_address,
        "device_label": r.device_label,
        "device_vendor": r.device_vendor,
        "device_model": r.device_model,
        "device_firmware_version": r.device_firmware_version,
        "device_template": r.device_template,
        "device_location": r.device_location,
        "device_serial_number": r.device_serial_number,
        "device_description": r.device_description,
        "device_username": r.device_username,
        "device_password": r.device_password,
        "device_uuid_alternate": r.device_uuid_alternate,
        "device_provisioned_date": r.device_provisioned_date,
        "device_provisioned_ip": r.device_provisioned_ip,
        "device_provisioned_method": r.device_provisioned_method,
        "extension": line.as_ref().and_then(|l| l.user_id.clone()),
        "domain": line.as_ref().and_then(|l| l.server_address.clone()),
    })
}

async fn get_device(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = devices::Entity::find_by_id(id)
        .filter(devices::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => {
            let line = device_lines::Entity::find()
                .filter(device_lines::Column::DeviceUuid.eq(id))
                .one(db)
                .await
                .unwrap_or(None);
            (
                StatusCode::OK,
                Json(ser_device(&r, line.as_ref())),
            )
                .into_response()
        }
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
// Templates embedded via `crate::provision` (minijinja, include_str!,
// single-binary prod, no file I/O): all 31 vendors under
// `templates/provision/*`, verbatim port. Yealink/cisco base keep
// byte-identical legacy render (existing tests); model overrides
// (T46S/T29G/SPA122/...) render via minijinja parity templates.
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
    let v = vendor.to_lowercase();
    let m = model.trim();
    let ml = m.to_lowercase();
    match v.as_str() {
        // Byte-identical legacy base; model overrides via minijinja parity.
        "yealink" => match ml.as_str() {
            "t46s" | "t29g" => crate::provision::render(&v, mac, m, extension, domain, password)
                .unwrap_or_else(|| render_yealink(mac, model, extension, domain, password)),
            _ => render_yealink(mac, model, extension, domain, password),
        },
        "cisco" => match ml.as_str() {
            "spa122" => crate::provision::render(&v, mac, m, extension, domain, password)
                .unwrap_or_else(|| render_cisco(mac, extension, domain)),
            _ => render_cisco(mac, extension, domain),
        },
        _ => crate::provision::render(&v, mac, m, extension, domain, password)
            .unwrap_or_else(|| render_generic(vendor, mac, extension, domain)),
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
    // Filtered query (not full-table scan) to avoid loading all tenants.
    let dev = devices::Entity::find()
        .filter(devices::Column::Mac.eq(want.clone()))
        .one(db)
        .await
        .unwrap_or(None);
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_edit") {
        return (s, j).into_response();
    }
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
        if !valid_mac(mac) {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "mac must match XX:XX:XX:XX:XX:XX"})),
            )
                .into_response();
        }
        am.mac = Set(norm_mac(mac));
    }
    if let Some(vendor) = payload.get("vendor").and_then(|v| v.as_str()) {
        if vendor.trim().is_empty() {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "vendor required"})),
            )
                .into_response();
        }
        am.vendor = Set(vendor.trim().to_lowercase());
    }
    if let Some(model) = payload.get("model").and_then(|v| v.as_str()) {
        am.model = Set(Some(model.to_owned()));
    }
    if let Some(label) = payload.get("label").and_then(|v| v.as_str()) {
        am.label = Set(Some(label.to_owned()));
        am.device_label = Set(Some(label.to_owned()));
    }
    for key in [
        "device_template",
        "device_location",
        "device_serial_number",
        "device_firmware_version",
        "device_description",
        "device_username",
        "device_password",
        "device_model",
        "device_vendor",
        "device_address",
    ] {
        if let Some(v) = payload.get(key) {
            let slot: &mut sea_orm::ActiveValue<Option<String>> = match key {
                "device_template" => &mut am.device_template,
                "device_location" => &mut am.device_location,
                "device_serial_number" => &mut am.device_serial_number,
                "device_firmware_version" => &mut am.device_firmware_version,
                "device_description" => &mut am.device_description,
                "device_username" => &mut am.device_username,
                "device_password" => &mut am.device_password,
                "device_model" => &mut am.device_model,
                "device_vendor" => &mut am.device_vendor,
                _ => &mut am.device_address,
            };
            if v.is_null() {
                *slot = Set(None);
            } else if let Some(s) = v.as_str() {
                *slot = Set(Some(s.to_owned()));
            }
        }
    }
    if let Some(v) = payload.get("device_profile_uuid").and_then(|v| v.as_str()) {
        if let Ok(u) = v.parse::<uuid::Uuid>() {
            am.device_profile_uuid = Set(Some(u));
        }
    }
    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(enabled);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(ser_device(&r, None)),
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_delete") {
        return (s, j).into_response();
    }
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
    // Cascade as set deletes: no per-row fetch-then-delete round-trips.
    let _ = device_lines::Entity::delete_many()
        .filter(device_lines::Column::DeviceUuid.eq(id))
        .exec(db)
        .await;
    let _ = device_keys::Entity::delete_many()
        .filter(device_keys::Column::DeviceUuid.eq(id))
        .exec(db)
        .await;
    let _ = device_settings::Entity::delete_many()
        .filter(device_settings::Column::DeviceUuid.eq(id))
        .exec(db)
        .await;
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

// ---- keys/lines/settings + device_profiles (parity: app/devices/router.py) ----
use pbx_db::entities::{device_keys, device_profiles, device_settings};

async fn device_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &uuid::Uuid,
) -> Option<devices::Model> {
    devices::Entity::find_by_id(*id)
        .filter(devices::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn list_keys(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if device_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = device_keys::Entity::find()
        .filter(device_keys::Column::DeviceUuid.eq(id))
        .filter(device_keys::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"device_key_uuid": r.device_key_uuid, "device_uuid": r.device_uuid, "device_key_value": r.device_key_value, "device_key_label": r.device_key_label})
    }).collect();
    (
        crate::paged_headers(&format!("/api/v1/devices/{id}/keys"), page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_key(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if device_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let kid = uuid::Uuid::now_v7();
    let am = device_keys::ActiveModel {
        device_key_uuid: Set(kid),
        domain_id: Set(domain),
        device_uuid: Set(id),
        device_profile_uuid: Set(None),
        device_key_id: Set(payload
            .get("device_key_id")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        device_key_category: Set(payload
            .get("device_key_category")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        device_key_vendor: Set(None),
        device_key_type: Set(payload
            .get("device_key_type")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        device_key_subtype: Set(None),
        device_key_line: Set(payload
            .get("device_key_line")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        device_key_value: Set(payload
            .get("device_key_value")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        device_key_extension: Set(payload
            .get("device_key_extension")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        device_key_protected: Set(None),
        device_key_label: Set(payload
            .get("device_key_label")
            .or_else(|| payload.get("label"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        device_key_icon: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"device_key_uuid": kid, "device_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn patch_key(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, kid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_keys::Entity::find_by_id(kid)
        .filter(device_keys::Column::DeviceUuid.eq(id))
        .filter(device_keys::Column::DomainId.eq(domain))
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
    let mut am: device_keys::ActiveModel = r.into();
    if let Some(v) = payload.get("device_key_value").and_then(|v| v.as_str()) {
        am.device_key_value = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("device_key_label").and_then(|v| v.as_str()) {
        am.device_key_label = Set(Some(v.to_owned()));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(serde_json::json!({"device_key_uuid": r.device_key_uuid, "device_key_value": r.device_key_value}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("update failed: {e}")}))).into_response(),
    }
}

async fn delete_key(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, kid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_keys::Entity::find_by_id(kid)
        .filter(device_keys::Column::DeviceUuid.eq(id))
        .filter(device_keys::Column::DomainId.eq(domain))
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
    let am: device_keys::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_lines(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if device_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = device_lines::Entity::find()
        .filter(device_lines::Column::DeviceUuid.eq(id))
        .filter(device_lines::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"device_line_uuid": r.device_line_uuid, "device_uuid": r.device_uuid, "line_number": r.line_number, "user_id": r.user_id})
    }).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/devices/{id}/lines"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_line(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if device_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let lid = uuid::Uuid::now_v7();
    let am = device_lines::ActiveModel {
        device_line_uuid: Set(lid),
        domain_id: Set(domain),
        device_uuid: Set(id),
        line_number: Set(payload
            .get("line_number")
            .and_then(|v| v.as_str())
            .unwrap_or("1")
            .to_owned()),
        server_address: Set(payload
            .get("server_address")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        server_address_primary: Set(None),
        server_address_secondary: Set(None),
        outbound_proxy_primary: Set(None),
        outbound_proxy_secondary: Set(None),
        label: Set(payload
            .get("label")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        display_name: Set(payload
            .get("display_name")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        user_id: Set(payload
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        auth_id: Set(payload
            .get("auth_id")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        password: Set(payload
            .get("password")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        sip_port: Set(None),
        sip_transport: Set(None),
        register_expires: Set(None),
        shared_line: Set(None),
        enabled: Set(true),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"device_line_uuid": lid, "device_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn patch_line(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, lid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_lines::Entity::find_by_id(lid)
        .filter(device_lines::Column::DeviceUuid.eq(id))
        .filter(device_lines::Column::DomainId.eq(domain))
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
    let mut am: device_lines::ActiveModel = r.into();
    if let Some(v) = payload.get("user_id").and_then(|v| v.as_str()) {
        am.user_id = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({"device_line_uuid": r.device_line_uuid, "user_id": r.user_id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_line(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, lid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_lines::Entity::find_by_id(lid)
        .filter(device_lines::Column::DeviceUuid.eq(id))
        .filter(device_lines::Column::DomainId.eq(domain))
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
    let am: device_lines::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_settings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if device_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let paginator = device_settings::Entity::find()
        .filter(device_settings::Column::DeviceUuid.eq(id))
        .filter(device_settings::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"device_setting_uuid": r.device_setting_uuid, "device_uuid": r.device_uuid, "device_setting_name": r.device_setting_name, "device_setting_value": r.device_setting_value})
    }).collect();
    (
        crate::paged_headers(
            &format!("/api/v1/devices/{id}/settings"),
            page,
            per_page,
            total,
        ),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if device_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let name = payload
        .get("device_setting_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "device_setting_name required"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let sid = uuid::Uuid::now_v7();
    let am = device_settings::ActiveModel {
        device_setting_uuid: Set(sid),
        domain_id: Set(domain),
        device_uuid: Set(id),
        device_profile_uuid: Set(None),
        device_setting_category: Set(payload
            .get("device_setting_category")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        device_setting_subcategory: Set(None),
        device_setting_name: Set(name.to_owned()),
        device_setting_value: Set(payload
            .get("device_setting_value")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        device_setting_enabled: Set(true),
        device_setting_description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"device_setting_uuid": sid, "device_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn patch_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, sid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_settings::Entity::find_by_id(sid)
        .filter(device_settings::Column::DeviceUuid.eq(id))
        .filter(device_settings::Column::DomainId.eq(domain))
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
    let mut am: device_settings::ActiveModel = r.into();
    if let Some(v) = payload.get("device_setting_value").and_then(|v| v.as_str()) {
        am.device_setting_value = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload
        .get("device_setting_enabled")
        .or_else(|| payload.get("enabled"))
        .and_then(|v| v.as_bool())
    {
        am.device_setting_enabled = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(serde_json::json!({"device_setting_uuid": r.device_setting_uuid, "device_setting_value": r.device_setting_value}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("update failed: {e}")}))).into_response(),
    }
}

async fn delete_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, sid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_settings::Entity::find_by_id(sid)
        .filter(device_settings::Column::DeviceUuid.eq(id))
        .filter(device_settings::Column::DomainId.eq(domain))
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
    let am: device_settings::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- device_profiles CRUD ----
async fn list_profiles(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["device_profile_view", "device_view"]) {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "device_profile_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/device_profiles", page, per_page, 0),
                Json(crate::paged_body(Vec::<serde_json::Value>::new(), 0, page, per_page)),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut sel = device_profiles::Entity::find();
    if !show_all {
        sel = sel.filter(device_profiles::Column::DomainId.eq(domain));
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(|r| {
        serde_json::json!({"device_profile_uuid": r.device_profile_uuid, "device_profile_name": r.device_profile_name, "device_profile_enabled": r.device_profile_enabled})
    }).collect();
    (
        crate::paged_headers("/api/v1/device_profiles", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn create_profile(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_add") {
        return (s, j).into_response();
    }
    let name = payload
        .get("device_profile_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "device_profile_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let pid = uuid::Uuid::now_v7();
    let am = device_profiles::ActiveModel {
        device_profile_uuid: Set(pid),
        domain_id: Set(domain),
        device_profile_name: Set(name.to_owned()),
        device_profile_enabled: Set(true),
        device_profile_description: Set(payload
            .get("device_profile_description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"device_profile_uuid": pid, "device_profile_name": name})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn get_profile(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_profiles::Entity::find_by_id(id)
        .filter(device_profiles::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (StatusCode::OK, Json(serde_json::json!({"device_profile_uuid": r.device_profile_uuid, "device_profile_name": r.device_profile_name}))).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

async fn patch_profile(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_profiles::Entity::find_by_id(id)
        .filter(device_profiles::Column::DomainId.eq(domain))
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
    let mut am: device_profiles::ActiveModel = r.into();
    if let Some(v) = payload.get("device_profile_name").and_then(|v| v.as_str()) {
        am.device_profile_name = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("device_profile_description")
        .and_then(|v| v.as_str())
    {
        am.device_profile_description = Set(Some(v.to_owned()));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(serde_json::json!({"device_profile_uuid": r.device_profile_uuid, "device_profile_name": r.device_profile_name}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("update failed: {e}")}))).into_response(),
    }
}

async fn delete_profile(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "device_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = device_profiles::Entity::find_by_id(id)
        .filter(device_profiles::Column::DomainId.eq(domain))
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
    let am: device_profiles::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn device_sub_router() -> Router {
    Router::new()
        .route(
            "/api/v1/devices/{id}/keys",
            axum::routing::get(list_keys).post(create_key),
        )
        .route(
            "/api/v1/devices/{id}/keys/{kid}",
            axum::routing::patch(patch_key).delete(delete_key),
        )
        .route(
            "/api/v1/devices/{id}/lines",
            axum::routing::get(list_lines).post(create_line),
        )
        .route(
            "/api/v1/devices/{id}/lines/{lid}",
            axum::routing::patch(patch_line).delete(delete_line),
        )
        .route(
            "/api/v1/devices/{id}/settings",
            axum::routing::get(list_settings).post(create_setting),
        )
        .route(
            "/api/v1/devices/{id}/settings/{sid}",
            axum::routing::patch(patch_setting).delete(delete_setting),
        )
        .route(
            "/api/v1/device_profiles",
            post(create_profile).get(list_profiles),
        )
        .route(
            "/api/v1/device_profiles/",
            post(create_profile).get(list_profiles),
        )
        .route(
            "/api/v1/device_profiles/{id}",
            axum::routing::get(get_profile)
                .patch(patch_profile)
                .delete(delete_profile),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
