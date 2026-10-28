mod auth;
mod health;
pub mod tenant;

pub use auth::auth_router;
pub use health::health_router;
pub use tenant::require_tenant;

pub fn test_app() -> axum::Router {
    let extensions = axum::Router::new()
        .route(
            "/api/v1/extensions",
            axum::routing::get(|| async { axum::Json(serde_json::json!([])) }),
        )
        .layer(axum::middleware::from_fn(require_tenant));
    health_router().merge(auth_router()).merge(extensions)
}
