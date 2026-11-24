//! Menus + reference parity: `/menu_items` CRUD (React MenuItems page hits
//! it), `/menus`, `/menu_item_groups`, `/menu_languages`, `/countries`,
//! `/languages` (seeded), `/software`, `/notifications`, `/databases`.

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

async fn call(
    app: axum::Router,
    token: &str,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let b = body.map(|v| v.to_string()).unwrap_or_default();
    let req = match method {
        "POST" => Request::post(path),
        "PATCH" => Request::patch(path),
        "DELETE" => Request::delete(path),
        _ => Request::get(path),
    };
    let res = app
        .oneshot(auth(
            req.header("content-type", "application/json")
                .body(Body::from(b))
                .unwrap(),
            token,
        ))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn reference_tables_seeded_and_served() {
    pbx_api::seed_on_boot().await;
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        "/api/v1/languages",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "languages: {b}");
    assert!(
        b["total"].as_u64().unwrap_or(0) >= 100,
        "156 PHP langs seeded: {b}"
    );
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        "/api/v1/countries",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "countries: {b}");
    assert!(
        b["total"].as_u64().unwrap_or(0) >= 200,
        "245 PHP countries seeded: {b}"
    );
}

#[tokio::test]
async fn menu_items_crud_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/menu_items",
        Some(serde_json::json!({"menu_item_title": "Probe Item", "menu_item_link": "/probe", "menu_item_order": 7})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create item: {b}");
    assert_eq!(b["menu_item_title"], "Probe Item");
    assert_eq!(b["menu_item_order"], 7);
    let id = b["menu_item_uuid"].as_str().unwrap().to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/menu_items/{id}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get item: {g}");
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/menu_items/{id}"),
        Some(serde_json::json!({"title": "Probe Renamed"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch w/ React alias: {p}");
    assert_eq!(p["menu_item_title"], "Probe Renamed");
    let (s, l) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        "/api/v1/menu_items",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        l.get("data").and_then(|d| d.as_array()).is_some(),
        "paged envelope: {l}"
    );
    let (s, _) = call(
        pbx_api::test_app(),
        &token,
        "DELETE",
        &format!("/api/v1/menu_items/{id}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    for path in [
        "/api/v1/menus",
        "/api/v1/menu_item_groups",
        "/api/v1/menu_languages",
        "/api/v1/software",
        "/api/v1/notifications",
        "/api/v1/databases",
    ] {
        let (s, b) = call(pbx_api::test_app(), &token, "GET", path, None).await;
        assert_eq!(s, StatusCode::OK, "{path}: {b}");
    }
}

#[tokio::test]
async fn software_databases_notifications_crud() {
    let token = login_token().await;
    // software
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/software",
        Some(serde_json::json!({"software_name": "fs", "software_version": "1.10.12"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "software: {b}");
    let sw = b["software_uuid"].as_str().unwrap().to_string();
    let (s, _) = call(
        pbx_api::test_app(),
        &token,
        "DELETE",
        &format!("/api/v1/software/{sw}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    // databases
    let (s, b) = call(pbx_api::test_app(), &token, "POST", "/api/v1/databases", Some(serde_json::json!({"database_type": "pgsql", "database_name": "pbx", "database_host": "db"}))).await;
    assert_eq!(s, StatusCode::CREATED, "database: {b}");
    assert_eq!(b["database_name"], "pbx");
    let id = b["database_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/databases/{id}"),
        Some(serde_json::json!({"database_host": "db2"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "db patch: {p}");
    let (s, _) = call(
        pbx_api::test_app(),
        &token,
        "DELETE",
        &format!("/api/v1/databases/{id}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn menus_crud_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/menus",
        Some(serde_json::json!({"menu_name": "Probe Menu", "menu_language": "en-us", "menu_description": "probe menu"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create menu: {b}");
    assert_eq!(b["menu_name"], "Probe Menu");
    assert_eq!(b["menu_language"], "en-us");
    assert_eq!(b["menu_description"], "probe menu");
    let id = b["menu_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/menus/{id}"),
        Some(serde_json::json!({"menu_description": "renamed"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch menu: {p}");
    assert_eq!(p["menu_description"], "renamed");
    let (s, l) = call(pbx_api::test_app(), &token, "GET", "/api/v1/menus", None).await;
    assert_eq!(s, StatusCode::OK, "list menus: {l}");
    assert!(
        l["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["menu_uuid"] == id),
        "list contains created: {l}"
    );
    let (s, _) = call(
        pbx_api::test_app(),
        &token,
        "DELETE",
        &format!("/api/v1/menus/{id}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT, "delete menu");
}
