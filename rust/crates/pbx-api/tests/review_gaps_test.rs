//! Coverage for review-gap fixes: standalone user_settings list,
//! system upgrade/events endpoints, API index links.

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
    v["access_token"].as_str().unwrap().to_string()
}

fn auth(req: Request<Body>, token: &str) -> Request<Body> {
    let (mut parts, body) = req.into_parts();
    parts
        .headers
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    Request::from_parts(parts, body)
}

async fn get_json(path: &str, token: &str) -> (StatusCode, serde_json::Value) {
    let res = pbx_api::test_app()
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let s = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (s, v)
}

async fn post_json(
    path: &str,
    token: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let res = pbx_api::test_app()
        .oneshot(auth(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
            token,
        ))
        .await
        .unwrap();
    let s = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (s, v)
}

#[tokio::test]
async fn user_settings_standalone_list() {
    let token = login_token().await;
    let (s, v) = get_json("/api/v1/user_settings", &token).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert!(v.get("data").and_then(|d| d.as_array()).is_some(), "{v}");
    assert!(v.get("total").and_then(|t| t.as_u64()).is_some(), "{v}");
}

#[tokio::test]
async fn system_upgrade_status() {
    let token = login_token().await;
    let (s, v) = get_json("/api/v1/system/upgrade", &token).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(
        v.get("boot_migrate").and_then(|b| b.as_bool()),
        Some(true),
        "{v}"
    );
    assert_eq!(
        v.get("pending").and_then(|b| b.as_bool()),
        Some(false),
        "{v}"
    );
}

#[tokio::test]
async fn system_events_status() {
    let token = login_token().await;
    let (s, v) = get_json("/api/v1/system/events", &token).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(
        v.get("transport").and_then(|t| t.as_str()),
        Some("esl"),
        "{v}"
    );
    assert!(v.get("freeswitch").is_some(), "{v}");
}

#[tokio::test]
async fn api_index_links_new_routes() {
    let token = login_token().await;
    let (s, v) = get_json("/api/v1", &token).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    let links = &v["_links"];
    for key in ["user_settings", "system_upgrade", "system_events"] {
        assert!(links.get(key).is_some(), "missing _links.{key}: {v}");
    }
}

// PHP extension parity: copy + range + import (extension_copy.php,
// extension_edit.php range loop, extension_imports.php).
#[tokio::test]
async fn extension_copy_duplicates_source() {
    let token = login_token().await;
    let uniq = uuid::Uuid::now_v7().simple().to_string().replace('-', "")[..8].to_owned();
    let src = format!("7{uniq}");
    let (s, v) = post_json(
        "/api/v1/extensions",
        &token,
        serde_json::json!({"extension": src, "description": "orig"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "{v}");
    let id = v
        .get("extension_uuid")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_owned();
    assert!(!id.is_empty(), "{v}");
    let dst = format!("8{uniq}");
    let (s, v) = post_json(
        &format!("/api/v1/extensions/{id}/copy"),
        &token,
        serde_json::json!({"extension": dst}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "{v}");
    assert_eq!(
        v.get("extension").and_then(|e| e.as_str()),
        Some(dst.as_str()),
        "{v}"
    );
    // duplicate target → 409
    let (s, _) = post_json(
        &format!("/api/v1/extensions/{id}/copy"),
        &token,
        serde_json::json!({"extension": dst}),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);
}

#[tokio::test]
async fn extension_create_upserts_voicemail() {
    // PHP parity: every extension add creates the linked mailbox.
    let token = login_token().await;
    let uniq = uuid::Uuid::now_v7().simple().to_string().replace('-', "")[..8].to_owned();
    let ext = format!("5{uniq}");
    let (s, _) = post_json(
        "/api/v1/extensions",
        &token,
        serde_json::json!({"extension": ext}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let (s, v) = get_json("/api/v1/voicemails", &token).await;
    assert_eq!(s, StatusCode::OK, "{v}");
    let found = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .any(|r| r.get("voicemail_id").and_then(|i| i.as_str()) == Some(ext.as_str()))
        })
        .unwrap_or(false);
    assert!(
        found,
        "mailbox {ext} must exist after extension create: {v}"
    );
}

#[tokio::test]
async fn extension_range_batch_skips_existing() {
    let token = login_token().await;
    let base: i64 = 91000 + (uuid::Uuid::now_v7().as_u128() % 800) as i64;
    let (s, v) = post_json(
        "/api/v1/extensions:batch",
        &token,
        serde_json::json!({"start": base.to_string(), "range": 3}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "{v}");
    assert_eq!(v.get("created").and_then(|c| c.as_u64()), Some(3), "{v}");
    // rerun skips all three
    let (s, v) = post_json(
        "/api/v1/extensions:batch",
        &token,
        serde_json::json!({"start": base.to_string(), "range": 3}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "{v}");
    assert_eq!(v.get("created").and_then(|c| c.as_u64()), Some(0), "{v}");
    assert_eq!(v.get("skipped").and_then(|c| c.as_u64()), Some(3), "{v}");
}

#[tokio::test]
async fn extension_import_collects_row_errors() {
    let token = login_token().await;
    let uniq = uuid::Uuid::now_v7().simple().to_string().replace('-', "")[..8].to_owned();
    let (s, v) = post_json(
        "/api/v1/extensions/import",
        &token,
        serde_json::json!({"extensions": [
            {"extension": format!("6{uniq}")},
            {"extension": ""},
            {"extension": format!("6{uniq}")},
        ]}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "{v}");
    assert_eq!(v.get("imported").and_then(|c| c.as_u64()), Some(1), "{v}");
    assert_eq!(
        v.get("errors").and_then(|e| e.as_array()).map(|a| a.len()),
        Some(2),
        "{v}"
    );
}
