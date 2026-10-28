mod auth;
mod call_centers;
mod calls;
mod cdr;
mod conferences;
mod contacts;
mod devices;
mod dialplans;
mod domains;
mod email_queue;
mod extensions;
mod fax;
mod fs_xml;
mod gateways;
mod groups;
mod health;
mod ivr_menus;
mod permissions;
pub mod plugins;
mod recordings;
mod remaining;
mod remaining_a;
mod remaining_b;
mod remaining_c;
mod ring_groups;
pub(crate) mod routing_db;
mod sip;
pub mod tenant;
mod users;
mod vars_fs;
mod voicemails;

pub use auth::auth_router;
pub use health::health_router;
pub use tenant::require_tenant;
// XML fragment builders (FS `dialplan`/`directory` wiring swap points).
pub use cdr::build_cdr_xml;
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
            "contacts": {"href": "/api/v1/contacts"},
            "email_queue": {"href": "/api/v1/email_queue"},
            "vars": {"href": "/api/v1/vars"},
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
            "/api/v1/auth/me": {"get": {"summary": "Current user"}},
            "/api/v1/auth/refresh": {"post": {"summary": "Refresh token"}},
            "/api/v1/auth/logout": {"post": {"summary": "Logout"}},
            "/api/v1/extensions": {"get": {"summary": "List extensions"}},
            "/api/v1/users": {"get": {"summary": "List users"}},
            "/api/v1/groups": {"get": {"summary": "List groups"}},
            "/api/v1/devices": {"get": {"summary": "List devices"}},
            "/api/v1/voicemails": {"get": {"summary": "List voicemails"}},
            "/api/v1/conferences": {"get": {"summary": "List conferences"}},
        }
    }))
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
        .merge(calls::calls_router())
        .merge(ring_groups::ring_groups_router())
        .merge(call_centers::call_centers_router())
        .merge(ivr_menus::ivr_menus_router())
        .merge(voicemails::voicemails_router())
        .merge(conferences::conferences_router())
        .merge(fax::fax_router())
        .merge(cdr::cdr_router())
        .merge(recordings::recordings_router())
        .merge(users::users_router())
        .merge(groups::groups_router())
        .merge(domains::domains_bootstrap_router())
        .merge(domains::domains_router())
        .merge(permissions::permissions_router())
        .merge(contacts::contacts_router())
        .merge(devices::devices_router())
        .merge(devices::provision_router())
        .merge(email_queue::email_queue_router())
        .merge(remaining::remaining_router())
        .merge(plugins::plugins_router())
        .merge(plugins::plugins_available_router())
        .merge(plugins::example_router())
        .merge(plugins::domain_plugin_router())
        .layer(axum::middleware::from_fn(plugins::plugin_gate))
        .layer(axum::middleware::from_fn(accept_compat_middleware))
}

pub fn test_app() -> axum::Router {
    build_app()
}

/// Migrate on boot: initialize shared SeaORM connection (Postgres
/// `Migrator::up`, sqlite `create_schema`) before serving. Called by
/// `main.rs`; keeps fresh `db` volumes current (see rust/Dockerfile CMD).
pub async fn migrate_on_boot() {
    let _ = crate::routing_db::conn().await;
}
