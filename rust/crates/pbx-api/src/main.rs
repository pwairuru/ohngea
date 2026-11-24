use pbx_api::banner::{build_mode, database_kind, render_banner, resolve_port, BannerInfo};
use pbx_api::build_app;

/// 16k header tolerance: axum 0.8 `serve` has no `max_buf_size` hook, so the
/// 16384-byte header budget (see `pbx_api::MAX_HEADER_BYTES`) is enforced at
/// the edge reverse proxy (nginx `large_client_header_buffers 4 16k`).
/// This server keeps hyper defaults and relies on that proxy in production.

#[tokio::main]
async fn main() {
    // Production gate: release binaries refuse to boot on a missing or
    // well-known JWT_SECRET. Debug keeps the test-secret fallback.
    #[cfg(not(debug_assertions))]
    {
        match std::env::var("JWT_SECRET") {
            Ok(s)
                if !s.trim().is_empty()
                    && s.trim() != "test-secret"
                    && s.trim() != "dev-secret-change" => {}
            _ => {
                eprintln!(
                    "FATAL: JWT_SECRET must be set to a strong random value in release builds"
                );
                std::process::exit(1);
            }
        }
    }
    // Migrate on boot (Postgres Migrator up / sqlite bootstrap) before serving.
    pbx_api::migrate_on_boot().await;
    let port = resolve_port();
    let bind_addr = "0.0.0.0".to_string();
    let info = BannerInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        bind_addr: bind_addr.clone(),
        port,
        database_kind: database_kind(),
        build_mode: build_mode(),
    };
    println!("{}", render_banner(&info));
    let app = build_app();
    let listener = tokio::net::TcpListener::bind(format!("{bind_addr}:{port}"))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
