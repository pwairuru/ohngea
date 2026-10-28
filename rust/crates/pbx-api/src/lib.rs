mod health;

pub use health::health_router;

pub fn test_app() -> axum::Router {
    health_router()
}
