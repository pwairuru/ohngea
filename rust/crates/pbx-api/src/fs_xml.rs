//! FS XML handler — mirrors `app/freeswitch/xml_router.py`
//! (directory/dialplan/configuration) with a 60s in-memory cache
//! (mirrors `app/freeswitch/cache.py`) and IP-fallback note.
//!
//! Cache sits in front of the SeaORM/postgres render; `invalidate_cache`
//! is the invalidation hook vars PATCH/DELETE already call.

use axum::{http::StatusCode, response::IntoResponse, Extension as AxExtension, Router};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn cache() -> &'static Mutex<pbx_fs::cache::XmlCache> {
    static C: OnceLock<Mutex<pbx_fs::cache::XmlCache>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(pbx_fs::cache::XmlCache::new()))
}

/// Invalidate the FS XML cache (called by vars PATCH/DELETE).
pub fn invalidate_cache() {
    *cache().lock().expect("cache lock") = pbx_fs::cache::XmlCache::new();
}

fn split_user_domain(key_value: &str) -> (String, String) {
    match key_value.split_once('@') {
        // IP fallback note: when the domain part parses as an IP, the
        // extension lookup falls back to any-domain match.
        Some((user, domain)) => (user.to_owned(), domain.to_owned()),
        None => (key_value.to_owned(), "example.com".to_owned()),
    }
}

fn cached_or_render(key: &str, render: impl FnOnce() -> String) -> String {
    if let Some(hit) = cache()
        .lock()
        .expect("cache lock")
        .get(key)
        .map(str::to_owned)
    {
        return hit;
    }
    let doc = render();
    cache()
        .lock()
        .expect("cache lock")
        .insert(key.to_owned(), doc.clone());
    doc
}

async fn cached_or_render_async<F, Fut>(key: &str, render: F) -> String
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = String>,
{
    if let Some(hit) = cache()
        .lock()
        .expect("cache lock")
        .get(key)
        .map(str::to_owned)
    {
        return hit;
    }
    let doc = render().await;
    cache()
        .lock()
        .expect("cache lock")
        .insert(key.to_owned(), doc.clone());
    doc
}

async fn fs_xml_handler(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Per-request tenant scoping for RLS (no-op on sqlite unit tests).
    let db = crate::routing_db::conn().await;
    crate::routing_db::set_tenant(db, &claims).await;
    let section = params.get("section").cloned().unwrap_or_default();
    let key_value = params.get("key_value").cloned().unwrap_or_default();
    match section.as_str() {
        "directory" => {
            let (user, domain) = split_user_domain(&key_value);
            let key = format!("directory:{domain}:{user}");
            let doc = cached_or_render_async(&key, || async {
                // SeaORM/postgres render source.
                match crate::extensions::find_extension(&user, &domain).await {
                    Some(rec) => pbx_fs::xml::build_directory_xml_with_password(
                        &rec.domain,
                        &rec.extension,
                        &rec.password,
                    ),
                    // IP fallback: unknown user still renders a shell with the domain.
                    None => pbx_fs::xml::build_directory_xml(&domain, &user),
                }
            })
            .await;
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/xml")],
                doc,
            )
                .into_response()
        }
        "dialplan" => {
            let key = format!("dialplan:{key_value}");
            let doc = cached_or_render(&key, || pbx_fs::xml::build_dialplan_xml(&key_value));
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/xml")],
                doc,
            )
                .into_response()
        }
        "configuration" => {
            let key = format!("configuration:{key_value}");
            let doc = cached_or_render(&key, || {
                pbx_fs::xml::build_configuration_xml(&key_value, &key_value)
            });
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/xml")],
                doc,
            )
                .into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            JsonNotFound(serde_json::json!({"detail": "unknown section"})),
        )
            .into_response(),
    }
}

struct JsonNotFound(serde_json::Value);

impl IntoResponse for JsonNotFound {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::NOT_FOUND, axum::Json(self.0)).into_response()
    }
}

pub fn fs_xml_router() -> Router {
    Router::new()
        .route(
            "/api/internal/freeswitch/xml",
            axum::routing::get(fs_xml_handler),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
