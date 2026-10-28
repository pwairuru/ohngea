//! Plugin registry port — TDD RED.
//! Ground truth: app/plugins/base.py, app/plugins/registry.py,
//! app/registry.py (discover_apps 3-pass), app/plugins/router.py,
//! app/main.py:286-320, app/domains/models.py, docs/plugins.md.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

async fn login_token() -> String {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::post("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"username":"admin","password":"x"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v.get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token")
        .to_string()
}

async fn get_json(
    app: axum::Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let mut b = match method {
        "PUT" => Request::put(uri),
        "POST" => Request::post(uri),
        _ => Request::get(uri),
    };
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    let req = if let Some(v) = body {
        b.header("content-type", "application/json")
            .body(Body::from(v.to_string()))
            .unwrap()
    } else {
        b.body(Body::empty()).unwrap()
    };
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({"_raw": "<non-json>"}));
    (status, v)
}

// ---- 1. manifest parse: example manifest fields ----
#[test]
fn manifest_parses_example_fields() {
    let m = pbx_api::plugins::parse_example_manifest();
    assert_eq!(m.name, "example");
    assert_eq!(m.version, "1.0.0");
    assert_eq!(m.license, "MIT");
    assert_eq!(m.description, "Example plugin");
    assert_eq!(m.routes, vec!["/example".to_string()]);
}

// ---- 2. license gate ----
#[test]
fn license_gate_rejects_copyleft_allows_permissive() {
    for bad in ["GPL-3.0", "AGPL-3.0", "SSPL-1.0", "GPLv2", "gpl-3.0-only"] {
        assert!(
            !pbx_api::plugins::is_license_allowed(bad),
            "must reject {bad}"
        );
    }
    for good in [
        "MIT",
        "Apache-2.0",
        "BSD-3-Clause",
        "BSD-2-Clause",
        "ISC",
        "MPL-1.1",
        "CDDL-1.0",
        "PostgreSQL",
    ] {
        assert!(
            pbx_api::plugins::is_license_allowed(good),
            "must allow {good}"
        );
    }
}

// ---- 3. GET /api/v1/plugins enriched shape ----
#[tokio::test]
async fn list_plugins_enriched_shape() {
    let token = login_token().await;
    let (status, v) = get_json(
        pbx_api::test_app(),
        "GET",
        "/api/v1/plugins",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .expect("data array");
    assert!(data.len() >= 9, "all 9 plugins: got {}", data.len());
    let ex = data
        .iter()
        .find(|p| p.get("name").and_then(|n| n.as_str()) == Some("example"))
        .expect("example plugin listed");
    for key in [
        "name",
        "version",
        "license",
        "description",
        "routes",
        "category",
        "_links",
    ] {
        assert!(ex.get(key).is_some(), "missing key {key} in {ex}");
    }
    assert_eq!(
        v.get("total").and_then(|t| t.as_u64()),
        Some(data.len() as u64)
    );
}

// ---- 4. GET /api/v1/plugins/available public, no auth ----
#[tokio::test]
async fn available_plugins_public_no_auth() {
    let (status, v) = get_json(
        pbx_api::test_app(),
        "GET",
        "/api/v1/plugins/available",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .expect("data array");
    assert!(data.len() >= 9);
}

// ---- 5. tenant toggle + disabled route 403 ----
#[tokio::test]
async fn toggle_disables_plugin_route_with_403() {
    let token = login_token().await;
    // tenant domain id for the "default" test claims (UUIDv5 DNS, routing parity)
    let tenant_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"default").to_string();

    // disable example for this tenant
    let (status, v) = get_json(
        pbx_api::test_app(),
        "PUT",
        &format!("/api/v1/domains/{tenant_id}/plugins/example"),
        Some(&token),
        Some(serde_json::json!({"enabled": false})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v.get("enabled").and_then(|e| e.as_bool()), Some(false));

    // disabled plugin route -> 403 (Python lacks this enforcement; Rust fixes it)
    let (status, _) = get_json(
        pbx_api::test_app(),
        "GET",
        "/api/v1/example",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // re-enable -> 200 again
    let (status, _) = get_json(
        pbx_api::test_app(),
        "PUT",
        &format!("/api/v1/domains/{tenant_id}/plugins/example"),
        Some(&token),
        Some(serde_json::json!({"enabled": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, v) = get_json(
        pbx_api::test_app(),
        "GET",
        "/api/v1/example",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v.get("plugin").and_then(|p| p.as_str()), Some("example"));
}
