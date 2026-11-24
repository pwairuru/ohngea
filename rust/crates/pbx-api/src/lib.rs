#![recursion_limit = "256"]
mod admin;
pub mod alias;
mod analytics;
mod auth;
mod backup;
pub mod banner;
mod call_centers;
mod calls;
mod cdr;
mod conference_centers;
mod conference_controls;
mod conferences;
mod contacts;
mod dashboard;
mod devices;
pub(crate) mod dialplans;
mod domains;
mod email_queue;
mod extensions;
mod fax;
mod fs_status;
pub(crate) use fs_status::fs_status_probe;
mod fs_xml;
mod gateways;
mod groups;
mod health;
mod ivr_menus;
mod logs;
mod menus;
mod missing_tables;
mod permissions;
pub mod plugin_sandbox;
pub mod plugins;
pub mod provision;
mod recordings;
mod remaining;
mod remaining_a;
mod remaining_b;
mod remaining_c;
mod ring_groups;
/// Routing-DB helpers (tenant txn/GUCs). Public so integration tests can
/// seed rows directly (e.g. legacy NULL-domain rows) against the same
/// connection the handlers use.
pub mod routing_db;
mod services;
mod settings;
mod sip;
mod system;
pub mod tenant;
mod users;
mod vars_fs;
mod voicemails;

pub use auth::auth_router;
pub use health::health_router;
pub use tenant::{require_fs_or_tenant, require_tenant};
// XML fragment builders (FS `dialplan`/`directory` wiring swap points).
pub use cdr::{build_cdr_csv, build_cdr_xml, extension_summary, CdrExportRow};
pub use conferences::build_conference_dialplan_xml;
pub use ivr_menus::build_ivr_dialplan_xml;
pub use ring_groups::build_ring_group_dialplan_xml;
pub use voicemails::build_voicemail_directory_xml;

/// 16k header tolerance target (Global Constraints: JWT claims, HS256, 16k header tolerance).
/// axum 0.8 `serve` exposes no `max_buf_size` hook (verified in axum 0.8.9
/// `src/serve/mod.rs` — only `with_graceful_shutdown`/`local_addr`), so the
/// limit is enforced at the reverse proxy in production (nginx
/// `large_client_header_buffers`, see rust/docker-compose.rust.yml comment).
/// This constant documents the contract and is asserted by tests.
pub const MAX_HEADER_BYTES: usize = 16384;

/// Accept-compat middleware: accepts `Accept: application/vnd.pbx.v1+json`
/// (and anything else) without 406, adds `Vary: Accept` to every response.
/// 406-rejection is explicitly NOT enabled yet — see Global Constraints.
async fn accept_compat_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        axum::http::header::VARY,
        axum::http::HeaderValue::from_static("Accept"),
    );
    // Global fallback (parity with app/main.py::add_pagination_headers):
    // expose pagination headers whenever X-Total-Count is present.
    if res.headers().contains_key("x-total-count") {
        res.headers_mut().insert(
            "access-control-expose-headers",
            axum::http::HeaderValue::from_static("X-Total-Count, Link"),
        );
    }
    res
}

/// Shared `page`/`per_page` query (parity: `page>=1`, `per_page` 1..=100,
/// defaults match Python routers).
#[derive(Debug, serde::Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
    /// `?show=all` cross-domain scope (PHP `show == 'all'` parity). Read via
    /// [`tenant::wants_all`]; ignored unless the caller holds the resource's
    /// `_all` permission, so adding it here is behavior-preserving for all
    /// existing `PageQuery` lists.
    #[serde(default)]
    pub show: Option<String>,
}

/// `?show=all` passthrough for tenant-scoped list endpoints that define
/// their own query struct (or none at all).
#[derive(Debug, Default, serde::Deserialize)]
pub struct ShowQuery {
    #[serde(default)]
    pub show: Option<String>,
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    20
}

impl PageQuery {
    pub fn normalized(&self) -> (u64, u64) {
        pbx_db::pagination::paginate(self.page, self.per_page)
    }
}

/// `{data, total, page, per_page}` body built from the shared
/// `pbx_db::pagination::Page` struct (single pagination shape everywhere).
pub fn paged_body<T: serde::Serialize>(
    data: Vec<T>,
    total: u64,
    page: u64,
    per_page: u64,
) -> serde_json::Value {
    serde_json::to_value(pbx_db::pagination::Page::new(data, total, page, per_page)).unwrap_or(
        serde_json::json!({"data": [], "total": total, "page": page, "per_page": per_page}),
    )
}
/// Pagination headers parity (Python routers): `X-Total-Count` always,
/// `Link: <{base}?page={next}&per_page={n}>; rel="next"` when more pages.
/// Returns headers to layer onto the JSON `{data, total, page, per_page}` body.
pub fn paged_headers(base: &str, page: u64, per_page: u64, total: u64) -> axum::http::HeaderMap {
    let mut h = axum::http::HeaderMap::new();
    h.insert("x-total-count", total.to_string().parse().unwrap());
    if page * per_page < total {
        let link = format!(
            "<{base}?page={}&per_page={per_page}>; rel=\"next\"",
            page + 1
        );
        if let Ok(v) = link.parse() {
            h.insert(axum::http::header::LINK, v);
        }
    }
    h.insert(
        "access-control-expose-headers",
        axum::http::HeaderValue::from_static("X-Total-Count, Link"),
    );
    h
}

/// GET /api/v1 root `_links` (parity with app/main.py::api_root).
async fn api_root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "_links": {
            "self": {"href": "/api/v1"},
            "domains": {"href": "/api/v1/domains"},
            "extensions": {"href": "/api/v1/extensions"},
            "dialplans": {"href": "/api/v1/dialplans"},
            "gateways": {"href": "/api/v1/gateways"},
            "sip_profiles": {"href": "/api/v1/sip_profiles"},
            "ivr_menus": {"href": "/api/v1/ivr_menus"},
            "ring_groups": {"href": "/api/v1/ring_groups"},
            "time_conditions": {"href": "/api/v1/time_conditions"},
            "voicemails": {"href": "/api/v1/voicemails"},
            "conferences": {"href": "/api/v1/conferences"},
            "fax": {"href": "/api/v1/fax"},
            "call_flows": {"href": "/api/v1/call_flows"},
            "music_on_hold": {"href": "/api/v1/music_on_hold"},
            "follow_me": {"href": "/api/v1/follow_me"},
            "call_forward": {"href": "/api/v1/call_forward"},
            "call_block": {"href": "/api/v1/call_block"},
            "destinations": {"href": "/api/v1/destinations"},
            "cdr": {"href": "/api/v1/cdr"},
            "recordings": {"href": "/api/v1/recordings"},
            "call_centers": {"href": "/api/v1/call_centers"},
            "devices": {"href": "/api/v1/devices"},
            "groups": {"href": "/api/v1/groups"},
            "permissions": {"href": "/api/v1/permissions"},
            "users": {"href": "/api/v1/users"},
            "user_settings": {"href": "/api/v1/user_settings"},
            "contacts": {"href": "/api/v1/contacts"},
            "email_queue": {"href": "/api/v1/email_queue"},
            "vars": {"href": "/api/v1/vars"},
            "analytics_summary": {"href": "/api/v1/analytics/summary"},
            "sip_profiles_status": {"href": "/api/v1/sip_profiles/status"},
            "freeswitch_status": {"href": "/api/v1/freeswitch/status"},
            "system_status": {"href": "/api/v1/system/status"},
            "system_upgrade": {"href": "/api/v1/system/upgrade"},
            "system_events": {"href": "/api/v1/system/events"},
            "system_info": {"href": "/api/v1/system/info"},
            "services": {"href": "/api/v1/services"},
            "default_settings": {"href": "/api/v1/default_settings"},
            "domain_settings": {"href": "/api/v1/domain_settings"},
            "logs": {"href": "/api/v1/logs"},
            "plugins": {"href": "/api/v1/plugins"},
            "health": {"href": "/health"},
            "openapi": {"href": "/api/v1/openapi.json"},
        }
    }))
}

/// GET /api/v1/openapi.json (parity with FastAPI `openapi_url`).
async fn openapi_json() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "openapi": "3.1.0",
        "info": {"title": "Ohngea PBX", "version": "1.0.0"},
        "paths": {
            "/api/v1": {"get": {"summary": "API root _links"}},
            "/api/v1/auth/login": {"post": {"summary": "Login"}},
            "/api/v1/auth/mfa/enroll": {"post": {"summary": "Enroll TOTP"}},
            "/api/v1/auth/mfa/verify": {"post": {"summary": "Verify TOTP"}},
            "/api/v1/auth/password-reset/request": {"post": {"summary": "Request password reset"}},
            "/api/v1/auth/password-reset/confirm": {"post": {"summary": "Confirm password reset"}},
            "/api/v1/auth/me": {"get": {"summary": "Current user"}},
            "/api/v1/auth/refresh": {"post": {"summary": "Refresh token"}},
            "/api/v1/auth/logout": {"post": {"summary": "Logout"}},
            "/api/v1/extensions": {"get": {"summary": "List extensions"}},
            "/api/v1/extensions:batch": {"post": {"summary": "Bulk create extensions"}},
            "/api/v1/extensions/import": {"post": {"summary": "Import extensions"}},
            "/api/v1/users": {"get": {"summary": "List users"}},
            "/api/v1/user_settings": {"get": {"summary": "List user settings (tenant scope)"}},
            "/api/v1/system/upgrade": {"get": {"summary": "Upgrade status (boot-migrated)"}},
            "/api/v1/system/events": {"get": {"summary": "Event socket status"}},
            "/api/v1/groups": {"get": {"summary": "List groups"}},
            "/api/v1/devices": {"get": {"summary": "List devices"}},
            "/api/v1/voicemails": {"get": {"summary": "List voicemails"}},
            "/api/v1/conferences": {"get": {"summary": "List conferences"}},
        }
    }))
}

/// Fallback pagination parity (Python routers): any JSON `{"data": [...],
/// "total": N}` list that a handler returned *without* `X-Total-Count` gets
/// the header here, plus `page`/`per_page` echo and `Link: rel="next"` when
/// more pages remain. Handlers that already set headers/body are untouched.
async fn pagination_fallback_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::body::Body;
    let path = req.uri().path().to_owned();
    let query = req.uri().query().unwrap_or("").to_owned();
    let res = next.run(req).await;
    if res.headers().contains_key("x-total-count") {
        return res;
    }
    let ctype = res
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    if !ctype.contains("application/json") {
        return res;
    }
    let (mut parts, body) = res.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, 1024 * 1024 * 4).await else {
        return axum::response::Response::from_parts(parts, Body::empty());
    };
    let Ok(mut v) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return axum::response::Response::from_parts(parts, Body::from(bytes));
    };
    let Some(total) = v.get("total").and_then(|t| t.as_u64()) else {
        return axum::response::Response::from_parts(parts, Body::from(bytes));
    };
    if v.get("data").and_then(|d| d.as_array()).is_none() {
        return axum::response::Response::from_parts(parts, Body::from(bytes));
    }
    // page/per_page echo (defaults mirror PageQuery when absent/invalid).
    let mut page: u64 = 1;
    let mut per_page: u64 = 20;
    for kv in query.split('&') {
        let mut it = kv.splitn(2, '=');
        match (it.next(), it.next()) {
            (Some("page"), Some(n)) => {
                page = n.parse().unwrap_or(1).max(1);
            }
            (Some("per_page"), Some(n)) => {
                per_page = n.parse().unwrap_or(20).clamp(1, 100);
            }
            _ => {}
        }
    }
    if v.get("page").is_none() {
        v["page"] = serde_json::json!(page);
    }
    if v.get("per_page").is_none() {
        v["per_page"] = serde_json::json!(per_page);
    }
    parts.headers.insert(
        "x-total-count",
        total
            .to_string()
            .parse()
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("0")),
    );
    if page * per_page < total {
        let link = format!(
            "<{path}?page={}&per_page={per_page}>; rel=\"next\"",
            page + 1
        );
        if let Ok(hv) = link.parse() {
            parts.headers.insert(axum::http::header::LINK, hv);
        }
    }
    parts.headers.insert(
        "access-control-expose-headers",
        axum::http::HeaderValue::from_static("X-Total-Count, Link"),
    );
    let body = Body::from(serde_json::to_vec(&v).unwrap_or_else(|_| bytes.to_vec()));
    axum::response::Response::from_parts(parts, body)
}

fn root_router() -> axum::Router {
    axum::Router::new()
        .route("/api/v1", axum::routing::get(api_root))
        .route("/api/v1/", axum::routing::get(api_root))
        .route("/api/v1/openapi.json", axum::routing::get(openapi_json))
        .route("/openapi.json", axum::routing::get(openapi_json))
}

/// Full production router: health + auth + telephony routers (each with the
/// `require_tenant` guard) + FS XML + Accept compat.
/// `main.rs` serves this; `test_app()` returns this so tests see prod routes.
pub fn build_app() -> axum::Router {
    health_router()
        .merge(auth_router())
        .merge(root_router())
        .merge(extensions::extensions_router())
        .merge(dialplans::dialplans_router())
        .merge(gateways::gateways_router())
        .merge(sip::sip_router())
        .merge(vars_fs::vars_router())
        .merge(fs_xml::fs_xml_router())
        .merge(fs_status::fs_status_router())
        .merge(system::system_router())
        .merge(logs::logs_router())
        .merge(analytics::analytics_router())
        .merge(calls::calls_router())
        .merge(ring_groups::ring_groups_router())
        .merge(call_centers::call_centers_router())
        .merge(ivr_menus::ivr_menus_router())
        .merge(voicemails::voicemails_router())
        .merge(conferences::conferences_router())
        .merge(conference_controls::conference_controls_router())
        .merge(conference_centers::conference_centers_router())
        .merge(fax::fax_router())
        .merge(cdr::cdr_router())
        .merge(recordings::recordings_router())
        .merge(backup::backup_router())
        .merge(users::users_router())
        .merge(settings::settings_router())
        .merge(services::services_router())
        .merge(groups::groups_router())
        .merge(domains::domains_bootstrap_router())
        .merge(domains::domains_router())
        .merge(permissions::permissions_router())
        .merge(contacts::contacts_router())
        .merge(dashboard::dashboard_router())
        .merge(devices::devices_router())
        .merge(devices::device_sub_router())
        .merge(devices::provision_router())
        .merge(email_queue::email_queue_router())
        .merge(admin::admin_router())
        .merge(missing_tables::missing_tables_router())
        .merge(menus::menus_router())
        .merge(remaining::remaining_router())
        .merge(plugins::plugins_router())
        .merge(plugins::plugins_available_router())
        .merge(plugins::example_router())
        .merge(plugins::domain_plugin_router())
        .layer(axum::middleware::from_fn(plugins::plugin_gate))
        .layer(axum::middleware::from_fn(pagination_fallback_middleware))
        .layer(axum::middleware::from_fn(accept_compat_middleware))
}

pub fn test_app() -> axum::Router {
    build_app()
}

/// Migrate on boot: initialize shared SeaORM connection (Postgres
/// `Migrator::up`, sqlite `create_schema`) before serving. Called by
/// `main.rs`; keeps fresh `db` volumes current (see rust/docker-compose.rust.yml).
pub async fn migrate_on_boot() {
    let db = crate::routing_db::conn().await;
    // Seed reference data on boot (idempotent, row-count guarded).
    if let Err(e) = seed_on_boot_inner(db).await {
        eprintln!("migrate_on_boot seed failed: {e:?}");
    }
}

/// Idempotent boot seed for the default tenant (matches the stub-login
/// `domain_id = "default"` → UUIDv5 DNS namespace mapping in
/// `routing_db::domain`). Seeds vars (68) + sofia (6) + dialplans +
/// sip profiles + default/domain settings when their tables are empty.
/// Public so tests prove `GET /api/v1/vars` is non-empty after boot.
pub async fn seed_on_boot() {
    let db = crate::routing_db::conn().await;
    if let Err(e) = seed_on_boot_inner(db).await {
        eprintln!("seed_on_boot failed: {e:?}");
    }
}

async fn seed_on_boot_inner(db: &sea_orm::DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    let domain_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, "default".as_bytes());
    pbx_db::seeds::seed_on_boot(db, domain_id, "default").await
}
