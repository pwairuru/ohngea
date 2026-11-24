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

// ---- Task 2 helpers ----
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
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

fn uniq(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::now_v7().simple())
}

fn mint_token(sub: &str, domain_id: &str, perms: &[&str]) -> String {
    let owned: Vec<String> = perms.iter().map(|s| s.to_string()).collect();
    pbx_core::auth::encode_claims(sub, domain_id, "tester", &owned)
}

fn domain_names(v: &serde_json::Value) -> Vec<String> {
    v.get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("domain_name")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn missing_token_is_401_with_login_redirect_hint() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_token_is_401() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .header("authorization", "Bearer invalid.token.here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn valid_token_returns_envelope() {
    let token = login_token().await;
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .header("authorization", format!("Bearer {token}"))
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        v.get("data").and_then(|d| d.as_array()).map(|a| a.len()),
        Some(0)
    );
    assert_eq!(v.get("total").and_then(|t| t.as_u64()), Some(0));
}

/// Task 2: read path must NOT auto-create a domains row for unknown tenants.
#[tokio::test]
async fn task2_read_path_does_not_autocreate_domain() {
    let probe = uniq("probe");
    let token = mint_token(&uuid::Uuid::now_v7().to_string(), &probe, &["user_view"]);
    // Any tenant-scoped read (users list calls tenant_domain) must succeed
    // without a domain row.
    let (s, _) = req("GET", "/api/v1/users", Some(&token), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    // And no domains row may have been auto-inserted for the probe tenant:
    // a superadmin list must not contain it.
    let admin = login_token().await;
    let (s, v) = req(
        "GET",
        "/api/v1/domains",
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "superadmin list: {v}");
    assert!(
        !domain_names(&v).contains(&probe),
        "probe domain was auto-created on read path: {v}"
    );
}

/// Task 2: bootstrap seeds domain + global groups/perms + admin user;
/// list/get enforce isolation; delete cascades or 409s with blockers.
#[tokio::test]
async fn task2_bootstrap_scoping_and_delete() {
    let admin = login_token().await;
    let suffix = uuid::Uuid::now_v7().simple().to_string();
    let domain_a = format!("task2a-{suffix}");
    let boss = format!("boss-{suffix}");
    let secret = "Secret123!";

    // 1. bootstrap creates domain + seeds.
    let (s, v) = req(
        "POST",
        "/api/v1/domains/bootstrap",
        None,
        serde_json::json!({
            "domain_name": domain_a,
            "admin_username": boss,
            "admin_password": secret,
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "bootstrap: {v}");
    let domain_a_id = v
        .get("domain_id")
        .and_then(|i| i.as_str())
        .expect("domain_id")
        .to_owned();

    // second bootstrap attempt conflicts while any domain exists.
    let (s, _) = req(
        "POST",
        "/api/v1/domains/bootstrap",
        None,
        serde_json::json!({"domain_name": uniq("task2dup")}),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT);

    // 2. seeded admin can log in scoped to the new domain with real perms.
    let (s, v) = req(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": boss, "password": secret, "domain": domain_a}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "seeded admin login: {v}");
    let boss_token = v["access_token"].as_str().unwrap().to_owned();
    let (s, v) = req(
        "GET",
        "/api/v1/auth/me",
        Some(&boss_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "me: {v}");
    assert_eq!(
        v.get("username").and_then(|u| u.as_str()),
        Some(boss.as_str())
    );

    // 3. isolation: second tenant via POST /domains; domain_view sees own only.
    let domain_b = format!("task2b-{suffix}");
    let (s, v) = req(
        "POST",
        "/api/v1/domains",
        Some(&admin),
        serde_json::json!({"domain_name": domain_b}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create domain B: {v}");
    let domain_b_id = v["domain_id"].as_str().unwrap().to_owned();
    let token_b = mint_token(
        &uuid::Uuid::now_v7().to_string(),
        &domain_b_id,
        &["domain_view"],
    );
    let (s, v) = req(
        "GET",
        "/api/v1/domains",
        Some(&token_b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "B list: {v}");
    assert_eq!(
        domain_names(&v),
        vec![domain_b.clone()],
        "B must see only own domain: {v}"
    );
    // cross-tenant get is hidden.
    let (s, _) = req(
        "GET",
        &format!("/api/v1/domains/{domain_a_id}"),
        Some(&token_b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    // domain_all (or superadmin) sees every tenant.
    let token_all = mint_token(
        &uuid::Uuid::now_v7().to_string(),
        &domain_b_id,
        &["domain_view", "domain_all"],
    );
    let (s, v) = req(
        "GET",
        "/api/v1/domains",
        Some(&token_all),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "domain_all list: {v}");
    let names = domain_names(&v);
    assert!(
        names.contains(&domain_a) && names.contains(&domain_b),
        "domain_all sees all: {v}"
    );

    // 4. delete with live tenant rows → 409 with blockers (never silent orphan).
    let (s, v) = req(
        "POST",
        "/api/v1/extensions",
        Some(&boss_token),
        serde_json::json!({"extension": "1101", "password": "secret123456"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create extension in A: {v}");
    let ext_id = v["extension_uuid"].as_str().unwrap().to_owned();
    let (s, v) = req(
        "DELETE",
        &format!("/api/v1/domains/{domain_a_id}"),
        Some(&boss_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::CONFLICT, "delete with rows must 409: {v}");
    assert!(v.get("blockers").is_some(), "409 must name blockers: {v}");

    // 5. after removing rows, delete cascades tenant rows.
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/extensions/{ext_id}"),
        Some(&boss_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/domains/{domain_a_id}"),
        Some(&boss_token),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, _) = req(
        "GET",
        &format!("/api/v1/domains/{domain_a_id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    // cascaded admin can no longer log in.
    let (s, _) = req(
        "POST",
        "/api/v1/auth/login",
        None,
        serde_json::json!({"username": boss, "password": secret, "domain": domain_a}),
    )
    .await;
    assert_eq!(s, StatusCode::UNAUTHORIZED);

    // 6. empty domain B deletes cleanly too.
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/domains/{domain_b_id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
}
