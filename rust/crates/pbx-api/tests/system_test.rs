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

async fn post_json(
    token: &str,
    path: &str,
    v: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(v.to_string()))
                .unwrap(),
            token,
        ))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn get_json(token: &str, path: &str) -> (StatusCode, serde_json::Value) {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

// (1) users CRUD + groups assign
#[tokio::test]
async fn users_crud_and_groups_assign() {
    let token = login_token().await;
    let uname = format!("u{}", &uuid::Uuid::now_v7().to_string()[..8]);
    let (s, b) = post_json(
        &token,
        "/api/v1/users",
        serde_json::json!({"username": uname, "email": "a@b.c", "password": "secret123"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create user: {b}");
    let uid = b
        .get("user_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!uid.is_empty(), "user_uuid missing: {b}");

    let (s, b) = get_json(&token, "/api/v1/users").await;
    assert_eq!(s, StatusCode::OK, "list users: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope {{data,total}}: {b}"
    );
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);

    let gname = format!(
        "g{}",
        uuid::Uuid::now_v7().to_string()[..8].replace('-', "")
    );
    let (s, b) = post_json(&token, "/api/v1/groups", serde_json::json!({"name": gname})).await;
    assert_eq!(s, StatusCode::CREATED, "create group: {b}");
    let gid = b
        .get("group_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!gid.is_empty(), "group_uuid missing: {b}");

    let (s, b) = post_json(
        &token,
        &format!("/api/v1/groups/{gid}/members"),
        serde_json::json!({"user_uuid": uid}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::OK,
        "assign member: {b}"
    );

    let (s, b) = get_json(&token, &format!("/api/v1/groups/{gid}")).await;
    assert_eq!(s, StatusCode::OK, "get group: {b}");
    let body = b.to_string();
    assert!(body.contains(&uid), "group must list member {uid}: {b}");
}

// (2) domains bootstrap/status
#[tokio::test]
async fn domains_bootstrap_and_status() {
    // public status (no auth)
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/api/v1/domains/bootstrap/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        v.get("needs_setup").is_some() || v.get("has_domains").is_some(),
        "status shape: {v}"
    );

    let token = login_token().await;
    let (s, b) = get_json(&token, "/api/v1/domains").await;
    assert_eq!(s, StatusCode::OK, "list domains: {b}");
    assert!(
        b.get("data").is_some() || b.get("domains").is_some() || b.as_array().is_some(),
        "domains envelope: {b}"
    );
}

// (3) permissions list
#[tokio::test]
async fn permissions_list() {
    let token = login_token().await;
    let (s, b) = get_json(&token, "/api/v1/permissions").await;
    assert_eq!(s, StatusCode::OK, "list permissions: {b}");
    let data = b
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(!data.is_empty(), "permissions must be non-empty: {b}");
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);
}

// (4) contacts CRUD
#[tokio::test]
async fn contacts_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        &token,
        "/api/v1/contacts",
        serde_json::json!({"name": "Alice", "number": "1001"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create contact: {b}");
    let cid = b
        .get("contact_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!cid.is_empty(), "contact_uuid missing: {b}");

    let (s, b) = get_json(&token, "/api/v1/contacts").await;
    assert_eq!(s, StatusCode::OK, "list contacts: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );

    let (s, b) = get_json(&token, &format!("/api/v1/contacts/{cid}")).await;
    assert_eq!(s, StatusCode::OK, "get contact: {b}");
    assert!(b.to_string().contains("Alice"), "contact body: {b}");
}

// (5) devices CRUD + provision render (yealink)
#[tokio::test]
async fn devices_crud_and_provision_render() {
    let token = login_token().await;
    let mac = "00:11:22:33:44:55";
    let (s, b) = post_json(
        &token,
        "/api/v1/devices",
        serde_json::json!({"mac": mac, "vendor": "yealink", "extension": "1001", "domain": "example.com"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::OK || s == StatusCode::CONFLICT,
        "create device: {b}"
    );

    let (s, b) = get_json(&token, "/api/v1/devices").await;
    assert_eq!(s, StatusCode::OK, "list devices: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );

    // public provision fetch
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get(format!("/provision/{mac}.cfg"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "provision status");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        text.contains(mac) || text.contains("00-11-22"),
        "provision must contain MAC: {text}"
    );
    assert!(
        text.to_lowercase().contains("example.com") || text.to_lowercase().contains("domain"),
        "provision must contain domain: {text}"
    );
    assert!(
        text.to_lowercase().contains("yealink") || text.contains("account.1"),
        "yealink render expected: {text}"
    );
}

// (6) email_queue submit
#[tokio::test]
async fn email_queue_submit() {
    let token = login_token().await;
    let (s, b) = post_json(
        &token,
        "/api/v1/email_queue",
        serde_json::json!({"email_from": "a@x.y", "email_to": "b@x.y", "email_subject": "hi"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "submit email: {b}");
    assert!(
        b.get("email_queue_uuid").is_some() || b.get("id").is_some(),
        "queue id missing: {b}"
    );

    let (s, b) = get_json(&token, "/api/v1/email_queue").await;
    assert_eq!(s, StatusCode::OK, "list queue: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );
}

// RED: system/info carries version (crate versions + uptime)
#[tokio::test]
async fn system_info_has_version() {
    let token = login_token().await;
    let (s, b) = get_json(&token, "/api/v1/system/info").await;
    assert_eq!(s, StatusCode::OK, "system info: {b}");
    assert!(
        b.get("version").and_then(|v| v.as_str()).is_some(),
        "version: {b}"
    );
}

// RED: system/status is live (freeswitch + system keys, not static stub)
#[tokio::test]
async fn system_status_is_live() {
    let token = login_token().await;
    let (s, b) = get_json(&token, "/api/v1/system/status").await;
    assert_eq!(s, StatusCode::OK, "system status: {b}");
    assert!(b.get("freeswitch").is_some(), "freeswitch key: {b}");
    assert!(b.get("system").is_some(), "system key: {b}");
}

// RED: freeswitch status + logs endpoints exist (RealEs->mock fallback)
#[tokio::test]
async fn freeswitch_status_and_logs_exist() {
    let token = login_token().await;
    let (s, b) = get_json(&token, "/api/v1/freeswitch/status").await;
    assert_eq!(s, StatusCode::OK, "fs status: {b}");
    assert!(
        b.get("connected").and_then(|c| c.as_bool()).is_some(),
        "connected: {b}"
    );
    let (s, b) = get_json(&token, "/api/v1/logs").await;
    assert_eq!(s, StatusCode::OK, "logs: {b}");
    let (s, b) = get_json(&token, "/api/v1/logs/freeswitch").await;
    assert_eq!(s, StatusCode::OK, "fs logs: {b}");
    assert!(b.get("fs_status").is_some(), "fs_status: {b}");
}

// Contract: load_avg must be a 3-element [1m,5m,15m] array (frontend System.tsx does .map).
// Never a bare number, never null-when-possible (fallback [0,0,0]).
#[tokio::test]
async fn load_avg_is_triple_array() {
    let token = login_token().await;
    for path in ["/api/v1/system/metrics", "/api/v1/system/info"] {
        let (s, b) = get_json(&token, path).await;
        assert_eq!(s, StatusCode::OK, "{path}: {b}");
        let arr = b
            .get("load_avg")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("{path} load_avg must be array, got: {b}"));
        assert_eq!(arr.len(), 3, "{path} load_avg len 3: {b}");
        for (i, v) in arr.iter().enumerate() {
            assert!(v.is_number(), "{path} load_avg[{i}] must be number: {b}");
        }
    }
}
