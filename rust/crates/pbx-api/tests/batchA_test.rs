use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

fn token_with(perms: &[&str], domain: &str) -> String {
    pbx_core::auth::encode_claims(
        "00000000-0000-0000-0000-000000000001",
        domain,
        "tester",
        &perms.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    )
}

fn admin_token() -> String {
    pbx_core::auth::encode_test_token()
}

#[tokio::test]
async fn perm_guard_denies_extension_list_without_perm() {
    let app = pbx_api::test_app();
    let tok = token_with(&[], "default");
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .header("authorization", format!("Bearer {tok}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn perm_guard_allows_admin_extension_list() {
    let app = pbx_api::test_app();
    let tok = admin_token();
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .header("authorization", format!("Bearer {tok}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn cross_domain_extension_isolated() {
    let app = pbx_api::test_app();
    let tok_a = token_with(&["superadmin"], "tenant-a-x");
    let tok_b = token_with(&["superadmin"], "tenant-b-x");
    // create in A
    let res = app
        .oneshot(
            Request::post("/api/v1/extensions")
                .header("authorization", format!("Bearer {tok_a}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"extension": "1999"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let id = v
        .get("extension_uuid")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();
    // list in B must not contain it
    let app2 = pbx_api::test_app();
    let res = app2
        .oneshot(
            Request::get("/api/v1/extensions")
                .header("authorization", format!("Bearer {tok_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(data
        .iter()
        .all(|r| r.get("extension_uuid").and_then(|x| x.as_str()) != Some(id.as_str())));
    // direct get in B must be 404
    let app3 = pbx_api::test_app();
    let res = app3
        .oneshot(
            Request::get(format!("/api/v1/extensions/{id}"))
                .header("authorization", format!("Bearer {tok_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn default_settings_crud_roundtrip() {
    let app = pbx_api::test_app();
    let tok = admin_token();
    let res = app
        .oneshot(
            Request::post("/api/v1/default_settings")
                .header("authorization", format!("Bearer {tok}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"default_setting_category": "catA", "default_setting_name": "nameA"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let id = v
        .get("default_setting_uuid")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    let app2 = pbx_api::test_app();
    let res = app2
        .oneshot(
            Request::get("/api/v1/default_settings")
                .header("authorization", format!("Bearer {tok}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let app3 = pbx_api::test_app();
    let res = app3
        .oneshot(
            Request::get(format!("/api/v1/default_settings/{id}"))
                .header("authorization", format!("Bearer {tok}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn default_settings_deny_without_perm() {
    let app = pbx_api::test_app();
    let tok = token_with(&[], "default");
    let res = app
        .oneshot(
            Request::get("/api/v1/default_settings")
                .header("authorization", format!("Bearer {tok}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn perm_guard_denies_users_and_cdr_without_perm() {
    for path in [
        "/api/v1/users",
        "/api/v1/cdr",
        "/api/v1/groups",
        "/api/v1/domains",
    ] {
        let app = pbx_api::test_app();
        let tok = token_with(&[], "default");
        let res = app
            .oneshot(
                Request::get(path)
                    .header("authorization", format!("Bearer {tok}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN, "deny {path}");
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            v.get("detail")
                .and_then(|d| d.as_str())
                .is_some_and(|d| d.starts_with("Missing permission: ")),
            "403 body parity {path}: {v}"
        );
    }
}

#[tokio::test]
async fn perm_guard_denies_users_create_without_perm() {
    let app = pbx_api::test_app();
    let tok = token_with(&["user_view"], "default");
    let res = app
        .oneshot(
            Request::post("/api/v1/users")
                .header("authorization", format!("Bearer {tok}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"username": "noperm"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn user_settings_roundtrip_self_and_admin() {
    let app = pbx_api::test_app();
    let admin = admin_token();
    // create user
    let res = app
        .oneshot(
            Request::post("/api/v1/users")
                .header("authorization", format!("Bearer {admin}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"username": "settings-u1"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let uid = v
        .get("user_uuid")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    // PUT settings (admin)
    let app2 = pbx_api::test_app();
    let res = app2
        .oneshot(
            Request::put(format!("/api/v1/users/{uid}/settings"))
                .header("authorization", format!("Bearer {admin}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"settings": [
                        {"category": "theme", "name": "dark", "value": "1"}
                    ]})
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // GET settings shows the row
    let app3 = pbx_api::test_app();
    let res = app3
        .oneshot(
            Request::get(format!("/api/v1/users/{uid}/settings"))
                .header("authorization", format!("Bearer {admin}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v.get("total").and_then(|t| t.as_u64()), Some(1));
}

#[tokio::test]
async fn user_settings_cross_domain_is_404() {
    let tok_a = token_with(&["superadmin"], "tenant-us-a");
    let tok_b = token_with(&["superadmin"], "tenant-us-b");
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::post("/api/v1/users")
                .header("authorization", format!("Bearer {tok_a}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"username": "xdom-u"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let uid = v
        .get("user_uuid")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    let app2 = pbx_api::test_app();
    let res = app2
        .oneshot(
            Request::get(format!("/api/v1/users/{uid}/settings"))
                .header("authorization", format!("Bearer {tok_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cdr_cross_domain_isolated_and_denied_without_perm() {
    // 403 without perm
    let app = pbx_api::test_app();
    let tok = token_with(&[], "default");
    let res = app
        .oneshot(
            Request::get("/api/v1/cdr")
                .header("authorization", format!("Bearer {tok}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // create in A, invisible from B
    let tok_a = token_with(&["superadmin"], "tenant-cdr-a");
    let tok_b = token_with(&["superadmin"], "tenant-cdr-b");
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::post("/api/v1/cdr")
                .header("authorization", format!("Bearer {tok_a}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"destination_number": "1999"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let id = v
        .get("cdr_uuid")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    let app2 = pbx_api::test_app();
    let res = app2
        .oneshot(
            Request::get(format!("/api/v1/cdr/{id}"))
                .header("authorization", format!("Bearer {tok_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn groups_bulk_toggle_is_tenant_scoped() {
    let tok_a = token_with(&["superadmin"], "tenant-gt-a");
    let tok_b = token_with(&["superadmin"], "tenant-gt-b");
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::post("/api/v1/groups")
                .header("authorization", format!("Bearer {tok_a}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"name": "scoped-g"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let gid = v
        .get("group_uuid")
        .and_then(|x| x.as_str())
        .unwrap()
        .to_string();

    // cross-tenant bulk toggle must not touch the row
    let app2 = pbx_api::test_app();
    let res = app2
        .oneshot(
            Request::post("/api/v1/groups/bulk")
                .header("authorization", format!("Bearer {tok_b}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"action": "toggle", "ids": [gid]}).to_string(),
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
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        data.iter().any(|r| r.get("error").is_some()),
        "cross-tenant toggle must error: {v}"
    );
}
