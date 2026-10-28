mod auth;
mod health;
pub mod tenant;

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

fn extensions_router() -> axum::Router {
    axum::Router::new()
        .route(
            "/api/v1/extensions",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({"data": [], "total": 0}))
            }),
        )
        .layer(axum::middleware::from_fn(require_tenant))
}

/// Full production router: health + auth + guarded extensions + Accept compat.
/// `main.rs` serves this; `test_app()` returns this so tests see prod routes.
pub fn build_app() -> axum::Router {
    health_router()
        .merge(auth_router())
        .merge(extensions_router())
        .layer(axum::middleware::from_fn(accept_compat_middleware))
}

pub fn test_app() -> axum::Router {
    build_app()
}
