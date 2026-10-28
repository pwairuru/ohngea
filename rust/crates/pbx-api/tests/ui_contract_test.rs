//! UI contract parity — TDD RED.
//! Ground truth: frontend/src/lib/api.ts, Login.tsx, Users.tsx,
//! Extensions.tsx, Devices.tsx, plugins voicemail/conferences App.tsx,
//! nginx.conf, Python main.py, auth/router.py + schemas + service.py,
//! extensions/devices/routers.

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

async fn req(
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<serde_json::Value>,
) -> (StatusCode, axum::http::HeaderMap, serde_json::Value) {
    let app = pbx_api::test_app();
    let mut b = match method {
        "PUT" => Request::put(uri),
        "POST" => Request::post(uri),
        "PATCH" => Request::patch(uri),
        "DELETE" => Request::delete(uri),
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
    let headers = res.headers().clone();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
        .await
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({"_raw": "<non-json>"}));
    (status, headers, v)
}

// ---- 1. /me returns UserRead shape (user_uuid not sub) ----
#[tokio::test]
async fn me_returns_userread_shape() {
    let token = login_token().await;
    let (status, _, v) = req("GET", "/api/v1/auth/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(v.get("user_uuid").is_some(), "missing user_uuid in {v}");
    assert!(v.get("username").is_some(), "missing username in {v}");
    assert!(v.get("domain_id").is_some(), "missing domain_id in {v}");
    assert!(v.get("enabled").is_some(), "missing enabled in {v}");
    assert!(v.get("sub").is_none(), "must not expose sub, got {v}");
    assert!(v.get("perms").is_none(), "must not expose perms, got {v}");
}

// ---- 2. claims use permissions plural + username + iat ----
#[tokio::test]
async fn claims_use_permissions_plural_username_iat() {
    let token = login_token().await;
    let claims = pbx_core::auth::decode_test_token(&token).expect("decode");
    let raw: serde_json::Value = serde_json::to_value(&claims).unwrap();
    assert!(
        raw.get("permissions").is_some(),
        "claims need permissions plural: {raw}"
    );
    assert!(
        raw.get("perms").is_none(),
        "claims must not use perms: {raw}"
    );
    assert!(raw.get("username").is_some(), "claims need username: {raw}");
    assert!(raw.get("iat").is_some(), "claims need iat: {raw}");
    assert!(raw.get("sub").is_some(), "claims need sub: {raw}");
    assert!(
        raw.get("domain_id").is_some(),
        "claims need domain_id: {raw}"
    );
    assert!(raw.get("exp").is_some(), "claims need exp: {raw}");
}

// ---- 3. GET /api/v1 root 200 with _links ----
#[tokio::test]
async fn api_root_has_links() {
    let (status, _, v) = req("GET", "/api/v1", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let links = v.get("_links").expect("_links missing");
    for key in ["self", "users", "groups", "extensions", "devices"] {
        assert!(links.get(key).is_some(), "missing _links.{key} in {v}");
    }
}

// ---- 4. GET /api/v1/openapi.json 200 ----
#[tokio::test]
async fn openapi_json_served() {
    let (status, _, v) = req("GET", "/api/v1/openapi.json", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(v.get("openapi").is_some(), "missing openapi in {v}");
    assert!(v.get("paths").is_some(), "missing paths in {v}");
}

// ---- 5. POST /auth/refresh + /auth/logout exist ----
#[tokio::test]
async fn refresh_and_logout_exist() {
    let token = login_token().await;
    let (status, _, v) = req("POST", "/api/v1/auth/refresh", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "refresh: {v}");
    assert!(
        v.get("access_token").is_some(),
        "refresh missing token: {v}"
    );

    let (status, _, v) = req("POST", "/auth/refresh", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "legacy refresh: {v}");

    let (status, _, v) = req("POST", "/api/v1/auth/logout", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "logout: {v}");

    let (status, _, _) = req("POST", "/auth/logout", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
}

// ---- 6. list endpoints send X-Total-Count + Link + page/per_page ----
#[tokio::test]
async fn list_pagination_headers() {
    let token = login_token().await;
    for uri in [
        "/api/v1/extensions?page=1&per_page=1",
        "/api/v1/users?page=1&per_page=1",
        "/api/v1/groups?page=1&per_page=1",
        "/api/v1/devices?page=1&per_page=1",
        "/api/v1/voicemails?page=1&per_page=1",
        "/api/v1/conferences?page=1&per_page=1",
    ] {
        let (status, headers, v) = req("GET", uri, Some(&token), None).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {v}");
        assert!(
            headers.contains_key("x-total-count"),
            "{uri} missing X-Total-Count: {headers:?}"
        );
        assert_eq!(
            v.get("page"),
            Some(&serde_json::json!(1)),
            "{uri} missing page: {v}"
        );
        assert_eq!(
            v.get("per_page"),
            Some(&serde_json::json!(1)),
            "{uri} missing per_page: {v}"
        );
        // Link header present when more pages OR at least expose-headers; require
        // Access-Control-Expose-Headers whenever X-Total-Count present (main.py parity).
        let expose = headers.get("access-control-expose-headers");
        assert!(
            expose.is_some(),
            "{uri} missing Access-Control-Expose-Headers: {headers:?}"
        );
    }
}

// ---- 7. PATCH+DELETE /devices/{id} ----
#[tokio::test]
async fn devices_patch_delete() {
    let token = login_token().await;
    let (status, _, v) = req(
        "POST",
        "/api/v1/devices",
        Some(&token),
        Some(serde_json::json!({"mac":"00:11:22:33:44:99","vendor":"poly"})),
    )
    .await;
    assert!(
        status == StatusCode::CREATED || status == StatusCode::CONFLICT || status == StatusCode::OK,
        "create: {v}"
    );
    // fetch id via list (match the mac we just created)
    let (_, _, list) = req(
        "GET",
        "/api/v1/devices?page=1&per_page=50",
        Some(&token),
        None,
    )
    .await;
    let id = list
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|a| {
            a.iter()
                .find(|r| r.get("mac").and_then(|m| m.as_str()) == Some("00:11:22:33:44:99"))
                .or_else(|| a.first())
        })
        .and_then(|r| {
            r.get("device_uuid")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string())
        })
        .expect("need device id");
    let (status, _, _) = req(
        "PATCH",
        &format!("/api/v1/devices/{id}"),
        Some(&token),
        Some(serde_json::json!({"label":"desk"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "patch device");
    let (status, _, _) = req(
        "DELETE",
        &format!("/api/v1/devices/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert!(
        status == StatusCode::NO_CONTENT || status == StatusCode::OK,
        "delete device: {status}"
    );
}

// ---- 8. POST /users/bulk + /users/import ----
#[tokio::test]
async fn users_bulk_and_import() {
    let token = login_token().await;
    let (status, _, v) = req(
        "POST",
        "/api/v1/users/bulk",
        Some(&token),
        Some(serde_json::json!({"action":"toggle","ids":[]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "users bulk: {v}");
    let (status, _, v) = req(
        "POST",
        "/api/v1/users/import",
        Some(&token),
        Some(serde_json::json!({"csv":"username,email,password\nalice,a@b.c,Secret123!xxxx\n"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "users import: {v}");
}

// ---- 9. groups bulk/copy/toggle/users/permissions ----
#[tokio::test]
async fn groups_bulk_copy_toggle_users_permissions() {
    let token = login_token().await;
    let (status, _, v) = req(
        "POST",
        "/api/v1/groups",
        Some(&token),
        Some(serde_json::json!({"name":"uiteam1"})),
    )
    .await;
    assert!(
        status == StatusCode::CREATED || status == StatusCode::OK,
        "create group: {v}"
    );
    let gid = v
        .get("group_uuid")
        .and_then(|u| u.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // fallback via list
            String::new()
        });
    let gid = if gid.is_empty() {
        let (_, _, list) = req(
            "GET",
            "/api/v1/groups?page=1&per_page=50",
            Some(&token),
            None,
        )
        .await;
        // placeholder to avoid panic; bulk test still validates route exists
        list.get("data")
            .and_then(|d| d.as_array())
            .and_then(|a| a.first())
            .and_then(|r| r.get("group_uuid"))
            .and_then(|u| u.as_str())
            .unwrap_or("00000000-0000-0000-0000-000000000000")
            .to_string()
    } else {
        gid
    };
    let (status, _, _) = req(
        "POST",
        "/api/v1/groups/bulk",
        Some(&token),
        Some(serde_json::json!({"action":"toggle","ids":[]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "groups bulk");
    let (status, _, _) = req(
        "POST",
        &format!("/api/v1/groups/{gid}/copy"),
        Some(&token),
        None,
    )
    .await;
    assert!(
        status == StatusCode::CREATED || status == StatusCode::OK,
        "copy {status}"
    );
    let (status, _, _) = req(
        "POST",
        &format!("/api/v1/groups/{gid}/toggle"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "toggle {status}");
    let (status, _, _) = req(
        "GET",
        &format!("/api/v1/groups/{gid}/users"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "group users");
    let (status, _, _) = req(
        "GET",
        &format!("/api/v1/groups/{gid}/permissions"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "group permissions");
}

// ---- 10. GET /provision/{mac}.cfg (with .cfg suffix) ----
#[tokio::test]
async fn provision_cfg_suffix() {
    let token = login_token().await;
    let _ = req(
        "POST",
        "/api/v1/devices",
        Some(&token),
        Some(serde_json::json!({"mac":"00:11:22:33:44:AA","vendor":"yealink","extension":"1001","domain":"example.com"})),
    )
    .await;
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/provision/00:11:22:33:44:aa.cfg")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    assert!(!bytes.is_empty());
}
