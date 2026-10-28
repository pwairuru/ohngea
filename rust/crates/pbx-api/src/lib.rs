mod auth;
mod health;

pub use auth::auth_router;
pub use health::health_router;

pub fn test_app() -> axum::Router {
    health_router().merge(auth_router())
}
