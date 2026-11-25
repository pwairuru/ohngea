//! Plugin registry port — mirrors `app/plugins/registry.py`,
//! `app/registry.py::discover_apps`, `app/plugins/router.py`,
//! `app/main.py:286-320` (dynamic mount + skip list) and
//! `app/domains/models.py` (`domain_plugins` / `domain_apps`).
//!
//! Order: manifest → license gate → registry → gating → routers.
//!
//! License gate is an exact port of `is_license_allowed` (exact match,
//! contains-variation, explicit GPL/AGPL/SSPL reject).
//!
//! Mount parity with `app/main.py` skip list: core routers already serve
//! `conferences`, `recordings`, `ring_groups`, `voicemail(s)`, `fax`,
//! `contacts`, `time_conditions` — this module does NOT duplicate them, it
//! only adds the `example` reference plugin plus singular/plural aliases
//! (`/call_center` → `call_centers`, `/voicemail` → `voicemails`; aliases
//! live on the core routers). The registry still lists all 9 manifests.
//!
//! WASM sandboxing of plugin code lives in [`crate::plugin_sandbox`]
//! (`PluginSandbox`: manifest-gated, WASI off, fuel + timeout + memory cap,
//! `pbx_log` / `db_get`-stub host fns). The `example` reference route below
//! runs its guest through the sandbox to prove the path.
//!
//! BUG FIX vs Python: `app/domains/router.py::toggle_plugin` disables a
//! plugin per-tenant but nothing enforces it — disabled plugin routes still
//! return 200. `plugin_gate` (global middleware) + the per-handler check on
//! `example` return 403 instead. Covered by `tests/plugins_test.rs`.

use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---- manifest ----

/// Allowed licenses — exact port of `ALLOWED_LICENSES`.
pub const ALLOWED_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    "BSD-3",
    "BSD-2-Clause",
    "MPL-1.1",
    "CDDL-1.0",
    "ISC",
    "PostgreSQL",
    "MIT License",
    "Apache 2.0",
];

fn default_category() -> String {
    "advanced".to_string()
}

/// Plugin manifest — contract from `docs/plugins.md`:
/// `{name, version, license, description, routes, permissions, category}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub license: String,
    pub description: String,
    #[serde(default)]
    pub routes: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled_by_default: bool,
    /// Registry provenance: `app/registry.py` sets `source` to "app"/"plugin".
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub has_router: bool,
}

fn default_true() -> bool {
    true
}

fn default_source() -> String {
    "plugin".to_string()
}

// ---- license gate (exact port of app/plugins/registry.py) ----

/// Exact port of `normalize_license`.
#[must_use]
pub fn normalize_license(lic: &str) -> String {
    lic.trim().to_string()
}

/// Exact port of `is_license_allowed`.
#[must_use]
pub fn is_license_allowed(lic: &str) -> bool {
    let lic = normalize_license(lic);
    if ALLOWED_LICENSES.contains(&lic.as_str()) {
        return true;
    }
    let lower = lic.to_lowercase();
    for allowed in ALLOWED_LICENSES {
        let a = allowed.to_lowercase();
        if a.contains(lower.as_str()) || lower.contains(a.as_str()) {
            return true;
        }
    }
    if lower.contains("gpl") || lower.contains("agpl") || lower.contains("sspl") {
        return false;
    }
    false
}

/// Parse + license-validate one `manifest.toml` body.
/// Mirrors `load_plugins` raising `ValueError` on disallowed licenses.
pub fn parse_manifest_str(s: &str) -> Result<PluginManifest, String> {
    let m: PluginManifest = toml::from_str(s).map_err(|e| format!("manifest parse: {e}"))?;
    if !is_license_allowed(&m.license) {
        return Err(format!(
            "license {} not allowed for plugin {}",
            m.license, m.name
        ));
    }
    Ok(m)
}

// ---- registry ----

macro_rules! builtin {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../plugins/",
            $name,
            "/manifest.toml"
        ))
    };
}

/// The 9 ground-truth plugin manifests, embedded at compile time so the
/// registry is deterministic regardless of process cwd.
const BUILTIN_MANIFESTS: &[&str] = &[
    builtin!("call_center"),
    builtin!("conferences"),
    builtin!("contacts"),
    builtin!("example"),
    builtin!("fax"),
    builtin!("recordings"),
    builtin!("ring_groups"),
    builtin!("time_conditions"),
    builtin!("voicemail"),
];

/// Parse the embedded `example` manifest (used by tests + reference).
pub fn parse_example_manifest() -> PluginManifest {
    parse_manifest_str(builtin!("example")).expect("example manifest valid")
}

/// Candidate plugin dirs: explicit env override first, then the repo layout
/// relative to this crate, then cwd-relative (mirrors `load_plugins("plugins")`).
fn plugin_dir_candidates() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(d) = std::env::var("PBX_PLUGIN_DIR") {
        out.push(std::path::PathBuf::from(d));
    }
    out.push(std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../plugins"
    )));
    out.push(std::path::PathBuf::from("../../../plugins"));
    out.push(std::path::PathBuf::from("plugins"));
    out
}

/// Discover plugins: embedded builtins + filesystem `*/manifest.toml` scan
/// (dir entries override builtins on name clash, sorted by name — mirrors
/// `discover_apps` pass 3 dedupe + sort). License-invalid files are skipped
/// at serve time (strict `parse_manifest_str` still errors for tests).
pub fn discover_plugins() -> Vec<PluginManifest> {
    let mut by_name: HashMap<String, PluginManifest> = HashMap::new();
    for s in BUILTIN_MANIFESTS {
        if let Ok(m) = parse_manifest_str(s) {
            by_name.insert(m.name.clone(), m);
        }
    }
    for dir in plugin_dir_candidates() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let manifest = entry.path().join("manifest.toml");
            if !manifest.is_file() {
                continue;
            }
            let body = match std::fs::read_to_string(&manifest) {
                Ok(b) => b,
                Err(_) => continue,
            };
            match parse_manifest_str(&body) {
                Ok(mut m) => {
                    m.source = "plugin".to_string();
                    m.has_router = entry.path().join("router.py").is_file();
                    by_name.insert(m.name.clone(), m);
                }
                Err(e) => {
                    tracing::warn!(manifest = %manifest.display(), error = %e, "plugin registry: skipping manifest")
                }
            }
        }
        // First existing candidate dir wins (avoids double-scan of same tree
        // via both absolute and relative candidates).
        break;
    }
    let mut all: Vec<PluginManifest> = by_name.into_values().collect();
    all.sort_by(|a, b| a.name.cmp(&b.name));
    all
}

// ---- tenant gating ----

/// Map request path → owning plugin name. Singular/plural aliases share one
/// plugin (`call_center(s)`, `voicemail(s)`). Core-served plugins are gated
/// here too (Python lacks this enforcement — we fix it).
fn plugin_for_path(path: &str) -> Option<&'static str> {
    let first = path
        .strip_prefix("/api/v1/")?
        .split('/')
        .next()
        .unwrap_or("");
    match first {
        "example" => Some("example"),
        "call_center" | "call_centers" => Some("call_center"),
        "voicemail" | "voicemails" => Some("voicemail"),
        "conferences" => Some("conferences"),
        "recordings" => Some("recordings"),
        "ring_groups" => Some("ring_groups"),
        "fax" => Some("fax"),
        "contacts" => Some("contacts"),
        "time_conditions" => Some("time_conditions"),
        _ => None,
    }
}

fn bearer_or_cookie_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("access_token=") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

async fn plugin_disabled_for_tenant(
    db: &sea_orm::DatabaseConnection,
    tenant: uuid::Uuid,
    plugin: &str,
) -> bool {
    use pbx_db::entities::{domain_apps, domain_plugins};
    if let Ok(Some(row)) = domain_plugins::Entity::find()
        .filter(domain_plugins::Column::DomainId.eq(tenant))
        .filter(domain_plugins::Column::PluginName.eq(plugin))
        .one(db)
        .await
    {
        if !row.enabled {
            return true;
        }
    }
    if let Ok(Some(row)) = domain_apps::Entity::find()
        .filter(domain_apps::Column::DomainId.eq(tenant))
        .filter(domain_apps::Column::AppName.eq(plugin))
        .one(db)
        .await
    {
        if !row.enabled {
            return true;
        }
    }
    false
}

/// Global enforcement middleware: disabled plugin route → 403.
/// No/invalid token → pass through (inner `require_tenant` answers 401).
pub async fn plugin_gate(req: Request<Body>, next: Next) -> Response {
    let plugin = match plugin_for_path(req.uri().path()) {
        Some(p) => p,
        None => return next.run(req).await,
    };
    let tenant = bearer_or_cookie_token(req.headers())
        .and_then(|t| pbx_core::auth::decode_test_token(&t).ok())
        .map(|claims| crate::routing_db::domain(&claims));
    let Some(tenant) = tenant else {
        return next.run(req).await;
    };
    let db = crate::routing_db::conn().await;
    if plugin_disabled_for_tenant(db, tenant, plugin).await {
        tracing::warn!(plugin = plugin, "plugin disabled for tenant");
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"detail": format!("plugin {plugin} disabled for tenant")})),
        )
            .into_response();
    }
    next.run(req).await
}

// ---- handlers ----

fn enriched(m: &PluginManifest) -> serde_json::Value {
    serde_json::json!({
        "name": m.name,
        "version": m.version,
        "license": m.license,
        "description": m.description,
        "routes": m.routes,
        "category": m.category,
        "_links": {"self": {"href": format!("/api/v1/{}", m.name)}},
    })
}

/// `GET /api/v1/plugins` — auth required, enriched shape (registry port).
async fn list_plugins(Extension(_claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let all = discover_plugins();
    let total = all.len();
    let data: Vec<serde_json::Value> = all.iter().map(enriched).collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

/// `GET /api/v1/plugins/available` — public (login page parity).
async fn available_plugins() -> impl IntoResponse {
    let all = discover_plugins();
    let data: Vec<serde_json::Value> = all
        .iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "description": m.description,
                "category": m.category,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data}))
}

/// `GET /api/v1/example` — reference plugin route (mirrors
/// `plugins/example/router.py`). Per-handler gate check backs the global one.
/// Sandbox hook: after the gate, the reference guest
/// (`plugins/example/plugin.wasm`) runs through `PluginSandbox` and its
/// output is embedded additively (`sandbox`) — proves the exec path without
/// changing the `{plugin, status}` contract.
#[tracing::instrument(skip(claims), fields(username = %claims.username))]
async fn example_handler(
    Extension(claims): Extension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    if plugin_disabled_for_tenant(db, crate::routing_db::domain(&claims), "example").await {
        tracing::warn!(username = %claims.username, "plugin example disabled for tenant");
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"detail": "plugin example disabled for tenant"})),
        )
            .into_response();
    }
    let sandbox = {
        // Live tenant snapshot for the `db_get` host fn (sync closure over
        // owned counts — extism host fns cannot await SeaORM).
        let domain = crate::routing_db::domain(&claims);
        let ext_n = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain))
            .count(db)
            .await
            .unwrap_or(0);
        let gw_n = pbx_db::entities::gateways::Entity::find()
            .filter(pbx_db::entities::gateways::Column::DomainId.eq(domain))
            .count(db)
            .await
            .unwrap_or(0);
        let dial_n = pbx_db::entities::dialplans::Entity::find()
            .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain))
            .count(db)
            .await
            .unwrap_or(0);
        let mut sb = crate::plugin_sandbox::PluginSandbox::new();
        match sb.register(
            &parse_example_manifest(),
            crate::plugin_sandbox::EXAMPLE_GUEST_WASM.to_vec(),
        ) {
            Ok(()) => {
                match sb.call_plugin_with_lookup("example", "handle", b"example", move |key| {
                    i64::from(match key {
                        1 => ext_n > 0,
                        2 => gw_n > 0,
                        3 => dial_n > 0,
                        _ => false,
                    })
                }) {
                    Ok(out) => serde_json::json!({
                        "ok": true,
                        "output": String::from_utf8_lossy(&out),
                    }),
                    Err(e) => serde_json::json!({"ok": false, "error": e.to_string()}),
                }
            }
            Err(e) => serde_json::json!({"ok": false, "error": e.to_string()}),
        }
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "plugin": "example",
            "status": "ok",
            "sandbox": sandbox,
            "_links": {"self": {"href": "/api/v1/example"}},
        })),
    )
        .into_response()
}

/// Ensure the parent `domains` row exists (same insert-if-missing pattern as
/// `routing_db::tenant_domain`; avoids FK surprises on fresh sqlite DBs).
async fn ensure_domain(db: &sea_orm::DatabaseConnection, id: uuid::Uuid) {
    use pbx_db::entities::domains;
    let name = id.to_string();
    let now = crate::routing_db::now();
    let am = domains::ActiveModel {
        domain_id: Set(id),
        domain_name: Set(name),
        enabled: Set(true),
        description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    use sea_orm::EntityTrait;
    let _ = domains::Entity::insert(am).exec(db).await;
}

/// `PUT /api/v1/domains/{id}/plugins/{name}` — tenant toggle (Python parity:
/// query `?enabled=` default true; also accepts JSON `{"enabled": bool}`).
#[tracing::instrument(skip(query, body), fields(domain_id = %domain_id, plugin_name = %plugin_name))]
async fn toggle_plugin(
    Extension(_claims): Extension<pbx_core::auth::Claims>,
    Path((domain_id, plugin_name)): Path<(uuid::Uuid, String)>,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    let enabled = body
        .as_ref()
        .and_then(|b| b.get("enabled"))
        .and_then(|e| e.as_bool())
        .or_else(|| query.get("enabled").map(|s| s == "true" || s == "1"))
        .unwrap_or(true);
    let db = crate::routing_db::conn().await;
    ensure_domain(db, domain_id).await;
    use pbx_db::entities::domain_plugins;
    let existing = domain_plugins::Entity::find()
        .filter(domain_plugins::Column::DomainId.eq(domain_id))
        .filter(domain_plugins::Column::PluginName.eq(plugin_name.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    match existing {
        Some(row) => {
            let mut am: domain_plugins::ActiveModel = row.into();
            am.enabled = Set(enabled);
            let _ = am.update(db).await;
        }
        None => {
            let am = domain_plugins::ActiveModel {
                id: Set(uuid::Uuid::now_v7()),
                domain_id: Set(domain_id),
                plugin_name: Set(plugin_name.clone()),
                enabled: Set(enabled),
                created_at: Set(crate::routing_db::now()),
            };
            let _ = am.insert(db).await;
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"plugin_name": plugin_name, "enabled": enabled})),
    )
}

/// `GET /api/v1/domains/{id}/plugins` — list tenant toggles.
async fn list_domain_plugins(
    Extension(_claims): Extension<pbx_core::auth::Claims>,
    Path(domain_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    use pbx_db::entities::domain_plugins;
    let rows = domain_plugins::Entity::find()
        .filter(domain_plugins::Column::DomainId.eq(domain_id))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| serde_json::json!({"plugin_name": r.plugin_name, "enabled": r.enabled}))
        .collect();
    Json(data)
}

/// `PUT /api/v1/domains/{id}/apps/{name}` — app toggle (`DomainApp` parity).
#[tracing::instrument(skip(query, body), fields(domain_id = %domain_id, app_name = %app_name))]
async fn toggle_app(
    Extension(_claims): Extension<pbx_core::auth::Claims>,
    Path((domain_id, app_name)): Path<(uuid::Uuid, String)>,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Json<serde_json::Value>>,
) -> impl IntoResponse {
    let enabled = body
        .as_ref()
        .and_then(|b| b.get("enabled"))
        .and_then(|e| e.as_bool())
        .or_else(|| query.get("enabled").map(|s| s == "true" || s == "1"))
        .unwrap_or(true);
    let db = crate::routing_db::conn().await;
    ensure_domain(db, domain_id).await;
    use pbx_db::entities::domain_apps;
    let existing = domain_apps::Entity::find()
        .filter(domain_apps::Column::DomainId.eq(domain_id))
        .filter(domain_apps::Column::AppName.eq(app_name.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    match existing {
        Some(row) => {
            let mut am: domain_apps::ActiveModel = row.into();
            am.enabled = Set(enabled);
            let _ = am.update(db).await;
        }
        None => {
            let am = domain_apps::ActiveModel {
                id: Set(uuid::Uuid::now_v7()),
                domain_id: Set(domain_id),
                app_name: Set(app_name.clone()),
                enabled: Set(enabled),
                created_at: Set(crate::routing_db::now()),
            };
            let _ = am.insert(db).await;
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"app_name": app_name, "enabled": enabled})),
    )
}

/// `GET /api/v1/domains/{id}/apps` — list tenant app toggles.
async fn list_domain_apps(
    Extension(_claims): Extension<pbx_core::auth::Claims>,
    Path(domain_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    use pbx_db::entities::domain_apps;
    let rows = domain_apps::Entity::find()
        .filter(domain_apps::Column::DomainId.eq(domain_id))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| serde_json::json!({"app_name": r.app_name, "enabled": r.enabled}))
        .collect();
    Json(data)
}

// ---- routers ----

pub fn plugins_router() -> Router {
    Router::new()
        .route("/api/v1/plugins", axum::routing::get(list_plugins))
        .route("/api/v1/plugins/", axum::routing::get(list_plugins))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

pub fn plugins_available_router() -> Router {
    Router::new().route(
        "/api/v1/plugins/available",
        axum::routing::get(available_plugins),
    )
}

pub fn example_router() -> Router {
    Router::new()
        .route("/api/v1/example", axum::routing::get(example_handler))
        .route("/api/v1/example/", axum::routing::get(example_handler))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

pub fn domain_plugin_router() -> Router {
    Router::new()
        .route(
            "/api/v1/domains/{id}/plugins",
            axum::routing::get(list_domain_plugins),
        )
        .route(
            "/api/v1/domains/{id}/plugins/{name}",
            axum::routing::put(toggle_plugin),
        )
        .route(
            "/api/v1/domains/{id}/apps",
            axum::routing::get(list_domain_apps),
        )
        .route(
            "/api/v1/domains/{id}/apps/{name}",
            axum::routing::put(toggle_app),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
