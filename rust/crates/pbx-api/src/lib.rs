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
mod recordings;
mod remaining;
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
    res
}

/// Full production router: health + auth + telephony routers (each with the
/// `require_tenant` guard) + FS XML + Accept compat.
/// `main.rs` serves this; `test_app()` returns this so tests see prod routes.
pub fn build_app() -> axum::Router {
    health_router()
        .merge(auth_router())
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
