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

// PHP ground truth: fusionpbx/core/services (v_services CRUD pages,
// perms service_view/add/edit/delete/all, global table with no domain_id).
#[tokio::test]
async fn services_crud_roundtrip() {
    let admin = login_token().await;
    let name = uniq("svc");

    // create
    let (s, v) = req(
        "POST",
        "/api/v1/services",
        Some(&admin),
        serde_json::json!({
            "service_name": name,
            "service_category": "switch",
            "service_file": "freeswitch.service",
            "service_enabled": true,
            "service_description": "test service",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create service: {v}");
    let id = v
        .get("service_uuid")
        .and_then(|i| i.as_str())
        .expect("service_uuid")
        .to_owned();

    // get
    let (s, v) = req(
        "GET",
        &format!("/api/v1/services/{id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get service: {v}");
    assert_eq!(
        v.get("service_name").and_then(|n| n.as_str()),
        Some(name.as_str())
    );

    // list contains it
    let (s, v) = req("GET", "/api/v1/services", Some(&admin), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "list services: {v}");
    let names: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("service_name")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(names.contains(&name), "list must contain {name}: {v}");

    // patch
    let (s, v) = req(
        "PATCH",
        &format!("/api/v1/services/{id}"),
        Some(&admin),
        serde_json::json!({"service_enabled": false}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch service: {v}");

    // delete
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/services/{id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert!(
        s == StatusCode::NO_CONTENT || s == StatusCode::OK,
        "delete service: {s}"
    );

    // gone
    let (s, _) = req(
        "GET",
        &format!("/api/v1/services/{id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn services_require_auth_and_permission() {
    // no token -> 401
    let (s, _) = req(
        "GET",
        "/api/v1/services",
        None,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);

    // token without service_view -> 403
    let limited = pbx_core::auth::encode_claims(
        "00000000-0000-0000-0000-000000000099",
        "default",
        "tester",
        &["extension_view".to_string()],
    );
    let (s, v) = req(
        "GET",
        "/api/v1/services",
        Some(&limited),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "missing perm must 403: {v}");
}
