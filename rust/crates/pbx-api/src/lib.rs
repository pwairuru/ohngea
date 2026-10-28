mod auth;
mod calls;
mod dialplans;
mod extensions;
mod fs_xml;
mod gateways;
mod health;
mod sip;
pub mod tenant;
mod vars_fs;

pub use auth::auth_router;
pub use health::health_router;
pub use tenant::require_tenant;

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
        .layer(axum::middleware::from_fn(accept_compat_middleware))
}

pub fn test_app() -> axum::Router {
    build_app()
}
