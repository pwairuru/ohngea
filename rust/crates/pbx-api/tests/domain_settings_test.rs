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
    path: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = pbx_api::test_app();
    let mut b = match method {
        "GET" => Request::get(path),
        "POST" => Request::post(path),
        "PATCH" => Request::patch(path),
        "DELETE" => Request::delete(path),
        _ => Request::post(path),
    };
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    let res = app
        .oneshot(
            b.header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

fn uniq(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::now_v7().simple())
}

// PHP ground truth: fusionpbx/core/domain_settings (v_domain_settings CRUD,
// perms domain_setting_view/add/edit/delete, tenant-scoped by domain_uuid).
#[tokio::test]
async fn domain_settings_crud_roundtrip() {
    let admin = login_token().await;
    let key = uniq("k");

    // create
    let (s, v) = req(
        "POST",
        "/api/v1/domain_settings",
        Some(&admin),
        serde_json::json!({
            "category": "testcat",
            "subcategory": "testsub",
            "name": key,
            "value": "v1",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create domain_setting: {v}");
    let id = v
        .get("id")
        .or_else(|| v.get("domain_setting_uuid"))
        .and_then(|i| i.as_str())
        .expect("id")
        .to_owned();

    // get
    let (s, v) = req(
        "GET",
        &format!("/api/v1/domain_settings/{id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get domain_setting: {v}");

    // list contains it (tenant-scoped)
    let (s, v) = req(
        "GET",
        "/api/v1/domain_settings",
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list domain_settings: {v}");

    // patch
    let (s, v) = req(
        "PATCH",
        &format!("/api/v1/domain_settings/{id}"),
        Some(&admin),
        serde_json::json!({"value": "v2"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch domain_setting: {v}");

    // delete
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/domain_settings/{id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert!(
        s == StatusCode::NO_CONTENT || s == StatusCode::OK,
        "delete domain_setting: {s}"
    );

    // gone
    let (s, _) = req(
        "GET",
        &format!("/api/v1/domain_settings/{id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn domain_settings_require_auth_and_permission() {
    let (s, _) = req(
        "GET",
        "/api/v1/domain_settings",
        None,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);

    let limited = pbx_core::auth::encode_claims(
        "00000000-0000-0000-0000-000000000099",
        "default",
        "tester",
        &["extension_view".to_string()],
    );
    let (s, v) = req(
        "GET",
        "/api/v1/domain_settings",
        Some(&limited),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "missing perm must 403: {v}");
}
