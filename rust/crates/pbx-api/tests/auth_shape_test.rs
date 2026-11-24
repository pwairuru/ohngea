use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

async fn login(app: axum::Router, body: serde_json::Value) -> axum::response::Response {
    app.oneshot(
        Request::post("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn legacy_login_and_me_roundtrip() {
    let app = pbx_api::test_app();
    let login_body = serde_json::json!({"username":"admin","password":"x","domain":"default"});
    let res = login(app.clone(), login_body).await;
    assert_eq!(res.status(), StatusCode::OK);
    // Cookie parity with Python router (httpOnly access_token, Path=/, SameSite=Lax).
    let set_cookie = res
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("missing Set-Cookie")
        .to_string();
    assert!(
        set_cookie.starts_with("access_token="),
        "cookie: {set_cookie}"
    );
    assert!(set_cookie.contains("HttpOnly"), "cookie: {set_cookie}");
    assert!(set_cookie.contains("Path=/"), "cookie: {set_cookie}");
    assert!(set_cookie.contains("SameSite=Lax"), "cookie: {set_cookie}");
    // Accept-compat middleware: Vary: Accept on every response, no 406.
    assert_eq!(
        res.headers().get("vary").and_then(|v| v.to_str().ok()),
        Some("Accept")
    );
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = v
        .get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token");
    assert_eq!(v.get("token_type").and_then(|t| t.as_str()), Some("Bearer"));
    assert_eq!(v.get("expires_in").and_then(|t| t.as_u64()), Some(3600));

    let res = app
        .oneshot(
            Request::get("/api/v1/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn legacy_auth_paths_and_unauthorized_me() {
    let app = pbx_api::test_app();
    let login_body = serde_json::json!({"username":"admin","password":"x","domain":"default"});
    let res = app
        .clone()
        .oneshot(
            Request::post("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
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
        v.get("access_token").and_then(|t| t.as_str()).is_some(),
        "missing access_token"
    );
    assert_eq!(v.get("token_type").and_then(|t| t.as_str()), Some("Bearer"));
    assert_eq!(v.get("expires_in").and_then(|t| t.as_u64()), Some(3600));

    let res = app
        .oneshot(Request::get("/auth/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_missing_or_empty_credentials_is_401() {
    // Missing password field -> 401 (not 422).
    let res = login(pbx_api::test_app(), serde_json::json!({"username":"admin"})).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Empty username -> 401.
    let res = login(
        pbx_api::test_app(),
        serde_json::json!({"username":"","password":"x"}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Empty password -> 401.
    let res = login(
        pbx_api::test_app(),
        serde_json::json!({"username":"admin","password":"  "}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn accept_header_compat_no_406() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/health")
                .header("accept", "application/vnd.pbx.v1+json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("vary").and_then(|v| v.to_str().ok()),
        Some("Accept")
    );
}

// ---- Task 1 (TDD RED): domain-scoped login + real perms + no admin bypass ----

async fn authed(
    method: &str,
    path: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = pbx_api::test_app();
    let mut b = match method {
        "POST" => Request::post(path),
        "GET" => Request::get(path),
        "PUT" => Request::put(path),
        "PATCH" => Request::patch(path),
        _ => Request::post(path),
    };
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    let has_body = method != "GET";
    let req = b
        .header("content-type", "application/json")
        .body(if has_body {
            Body::from(body.to_string())
        } else {
            Body::empty()
        })
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, v)
}

async fn stub_token() -> String {
    let (s, v) = authed(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username":"admin","password":"x"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "stub login: {v}");
    v["access_token"].as_str().expect("token").to_owned()
}

fn uniq(prefix: &str) -> String {
    format!(
        "{}{}",
        prefix,
        uuid::Uuid::now_v7().simple().to_string()[..8].to_owned()
    )
}

async fn create_user(token: &str, username: &str, password: &str) -> String {
    let (s, v) = authed(
        "POST",
        "/api/v1/users",
        Some(token),
        serde_json::json!({"username": username, "password": password}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create user {username}: {v}");
    v["user_uuid"].as_str().expect("user_uuid").to_owned()
}

#[tokio::test]
async fn login_user_at_domain_split_resolves_tenant() {
    let admin = stub_token().await;
    let username = uniq("atdom");
    let pw = "Secret123!";
    let uid = create_user(&admin, &username, pw).await;
    // user@domain split must resolve to the real user, not the stub fallback.
    let (s, v) = authed(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": format!("{username}@default"), "password": pw}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "user@domain login: {v}");
    let token = v["access_token"].as_str().expect("token").to_owned();
    let claims = pbx_core::auth::decode_test_token(&token).expect("decode");
    assert_eq!(claims.sub, uid, "expected real user, got stub");
}

#[tokio::test]
async fn login_unknown_domain_is_401() {
    let admin = stub_token().await;
    let username = uniq("unkdom");
    let pw = "Secret123!";
    create_user(&admin, &username, pw).await;
    let (s, _) = authed(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": username, "password": pw, "domain": "no-such-domain-xyz"}),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
    let (s, _) = authed(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": format!("{username}@no-such-domain-xyz"), "password": pw}),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_token_carries_db_permissions_not_hardcoded_admin() {
    let admin = stub_token().await;
    let username = uniq("permdb");
    let pw = "Secret123!";
    let uid = create_user(&admin, &username, pw).await;
    // group + permission assignment via API.
    let gname = uniq("tgrp");
    let (s, v) = authed(
        "POST",
        "/api/v1/groups",
        Some(&admin),
        serde_json::json!({"name": gname}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create group: {v}");
    let gid = v["group_uuid"].as_str().expect("group_uuid").to_owned();
    let (s, _) = authed(
        "GET",
        "/api/v1/permissions",
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, v) = authed(
        "PUT",
        &format!("/api/v1/groups/{gid}/permissions"),
        Some(&admin),
        serde_json::json!({"permission_names": ["extension_view"]}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "assign perms: {v}");
    let (s, v) = authed(
        "POST",
        &format!("/api/v1/groups/{gid}/members"),
        Some(&admin),
        serde_json::json!({"user_uuid": uid}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "assign member: {v}");
    // login must carry the DB permission, not hardcoded ["admin"].
    let (s, v) = authed(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": username, "password": pw}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "login: {v}");
    let token = v["access_token"].as_str().expect("token").to_owned();
    let claims = pbx_core::auth::decode_test_token(&token).expect("decode");
    assert!(
        claims.permissions.iter().any(|p| p == "extension_view"),
        "expected extension_view in {:?}",
        claims.permissions
    );
    assert!(
        !claims.permissions.iter().any(|p| p == "admin"),
        "must not hardcode admin in {:?}",
        claims.permissions
    );
    // user with no groups must NOT see extensions (no admin bypass).
    let username2 = uniq("noperm");
    create_user(&admin, &username2, pw).await;
    let (s, v) = authed(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": username2, "password": pw}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "login2: {v}");
    let t2 = v["access_token"].as_str().expect("token").to_owned();
    let (s, _) = authed(
        "GET",
        "/api/v1/extensions",
        Some(&t2),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "no-perm user must be forbidden");
}

#[tokio::test]
async fn admin_claim_does_not_bypass_permission() {
    let crafted = pbx_core::auth::encode_claims(
        "00000000-0000-0000-0000-000000000099",
        "default",
        "crafted",
        &["admin".to_string()],
    );
    let (s, _) = authed(
        "GET",
        "/api/v1/extensions",
        Some(&crafted),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "admin must not bypass");
}

#[tokio::test]
async fn me_and_refresh_reload_disabled_user() {
    let admin = stub_token().await;
    let username = uniq("disuser");
    let pw = "Secret123!";
    let uid = create_user(&admin, &username, pw).await;
    let (s, v) = authed(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": username, "password": pw}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "login: {v}");
    let token = v["access_token"].as_str().expect("token").to_owned();
    let (s, _) = authed(
        "GET",
        "/api/v1/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    // disable via stub admin.
    let (s, v) = authed(
        "PATCH",
        &format!("/api/v1/users/{uid}"),
        Some(&admin),
        serde_json::json!({"enabled": false}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "disable: {v}");
    let (s, _) = authed(
        "GET",
        "/api/v1/auth/me",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "disabled user me must be 401");
    let (s, _) = authed(
        "POST",
        "/api/v1/auth/refresh",
        Some(&token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(
        s,
        StatusCode::UNAUTHORIZED,
        "disabled user refresh must be 401"
    );
}
