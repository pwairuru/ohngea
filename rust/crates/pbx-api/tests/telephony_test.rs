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
    app: axum::Router,
    token: &str,
    path: &str,
    v: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
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

async fn get_json(
    app: axum::Router,
    token: &str,
    path: &str,
) -> (StatusCode, serde_json::Value, axum::http::HeaderMap) {
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let ctype = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let json = if ctype.contains("xml") {
        serde_json::json!({"_raw": String::from_utf8_lossy(&bytes).to_string()})
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    };
    (status, json, headers)
}

// (1) extensions CRUD: POST 1001 + 1002, password auto-gen when blank,
// PHP validation (non-empty only — no digit regex), GET list returns {data,total}
#[tokio::test]
async fn extensions_crud_with_autogen_password_and_validation() {
    let token = login_token().await;

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/extensions",
        serde_json::json!({"extension":"1001","password":"","domain":"example.com"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create 1001: {b}");
    let pw1 = b.get("password").and_then(|p| p.as_str()).unwrap_or("");
    assert!(
        !pw1.is_empty(),
        "blank password must auto-generate, got {b}"
    );

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/extensions",
        serde_json::json!({"extension":"1002","domain":"example.com"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create 1002: {b}");

    // PHP-legal values the old Python 2-7-digit rule wrongly rejected
    for ext in ["1", "12345678", "*98"] {
        let (s, b) = post_json(
            pbx_api::test_app(),
            &token,
            "/api/v1/extensions",
            serde_json::json!({"extension":ext,"domain":"example.com"}),
        )
        .await;
        assert!(
            s == StatusCode::CREATED || s == StatusCode::CONFLICT,
            "php-legal {ext}: {b}"
        );
    }
    // blank still 422 (PHP: empty($extension))
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/extensions",
        serde_json::json!({"extension":"","domain":"example.com"}),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);

    let (s, b, _) = get_json(pbx_api::test_app(), &token, "/api/v1/extensions").await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope {{data,total}}: {b}"
    );
    let total = b.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
    assert!(total >= 2, "expected >=2 extensions, got {b}");
}

// (2) FS XML from DB: directory contains password+domain, dialplan contains
// local_extension bridge, configuration sofia contains profile
#[tokio::test]
async fn fs_xml_served_from_db_state() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/extensions",
        serde_json::json!({"extension":"1001","password":"s3cret-pw","domain":"xmltest.example.com"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::CONFLICT || s == StatusCode::OK,
        "seed 1001: {b}"
    );

    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::get(
                "/api/internal/freeswitch/xml?section=directory&key_value=1001@xmltest.example.com",
            )
            .body(Body::empty())
            .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let xml = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        xml.contains("1001"),
        "directory xml must contain user: {xml}"
    );
    assert!(
        xml.contains("s3cret-pw"),
        "directory xml must contain password: {xml}"
    );
    assert!(
        xml.contains("xmltest.example.com"),
        "directory xml must contain domain: {xml}"
    );

    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::get("/api/internal/freeswitch/xml?section=dialplan&key_value=default")
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let xml = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        xml.contains("local_extension"),
        "dialplan must contain local_extension: {xml}"
    );
    assert!(
        xml.contains("bridge"),
        "dialplan must contain bridge: {xml}"
    );

    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::get("/api/internal/freeswitch/xml?section=configuration&key_value=sofia.conf")
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let xml = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        xml.contains("sofia"),
        "sofia configuration must mention sofia: {xml}"
    );
    assert!(
        xml.contains("profile"),
        "sofia configuration must contain profile: {xml}"
    );
}

// (3) ESL originate 1001 -> bridge 1002 via mock: call appears in channels, kill removes it
#[tokio::test]
async fn esl_originate_bridge_and_kill_call_flow() {
    let token = login_token().await;

    // originate extension-to-extension call through mocked transport
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/calls/originate",
        serde_json::json!({"from":"1001","to":"1002"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "originate: {b}");
    let call_id = b
        .get("call_id")
        .or_else(|| b.get("data").and_then(|d| d.get("call_id")))
        .and_then(|c| c.as_str())
        .expect("originate must return call_id")
        .to_string();
    assert!(!call_id.is_empty());

    // call appears in channels
    let (s, b, _) = get_json(pbx_api::test_app(), &token, "/api/v1/calls/channels").await;
    assert_eq!(s, StatusCode::OK, "channels: {b}");
    let body = b.to_string();
    assert!(
        body.contains(&call_id) || body.contains("1001"),
        "channels must show call: {b}"
    );

    // kill removes it
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::delete(format!("/api/v1/calls/{call_id}"))
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::OK || res.status() == StatusCode::NO_CONTENT,
        "kill status: {}",
        res.status()
    );

    let (s, b, _) = get_json(pbx_api::test_app(), &token, "/api/v1/calls/channels").await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        !b.to_string().contains(&call_id),
        "killed call must be gone: {b}"
    );
}

// (4) FS directory per-user: enriched vars (alias/cidr/vm/variables block)
// and tenant scoping (foreign domain never leaks the password).
#[tokio::test]
async fn fs_directory_per_user_enriched_vars_and_scoped() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/extensions",
        serde_json::json!({"extension":"1091","password":"pw-1091","domain":"dirtest.example.com","number_alias":"a1091","cidr":"10.9.0.0/16","toll_allow":"domestic","call_group":"sales","user_context":"dirtest.example.com","limit_destination":"1099","forward_busy_enabled":true,"forward_busy_destination":"1092","mwi_account":"1091@dirtest.example.com","auth_acl":"lan","absolute_codec_string":"PCMU,PCMA","effective_caller_id_name":"Dir Test"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::CONFLICT,
        "seed 1091: {b}"
    );
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/extensions",
        serde_json::json!({"extension":"1092","password":"pw-1092","domain":"dirtest.example.com","voicemail_password":"vm-1092","voicemail_mail_to":"ops@dirtest.example.com"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::CONFLICT,
        "seed 1092: {b}"
    );

    let fetch = |path: &str| {
        let token = token.clone();
        let path = path.to_owned();
        async move {
            let app = pbx_api::test_app();
            let res = app
                .oneshot(auth(
                    Request::get(path).body(Body::empty()).unwrap(),
                    &token,
                ))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
                .await
                .unwrap();
            String::from_utf8_lossy(&bytes).to_string()
        }
    };

    let xml =
        fetch("/api/internal/freeswitch/xml?section=directory&key_value=1091@dirtest.example.com")
            .await;
    for needle in [
        "number-alias=\"a1091\"",
        "cidr=\"10.9.0.0/16\"",
        "<variable name=\"toll_allow\" value=\"domestic\"/>",
        "<variable name=\"call_group\" value=\"sales\"/>",
        "<variable name=\"limit_max\" value=\"5\"/>",
        "<variable name=\"limit_destination\" value=\"1099\"/>",
        "<param name=\"MWI-Account\" value=\"1091@dirtest.example.com\"/>",
        "<param name=\"auth-acl\" value=\"lan\"/>",
        "<variable name=\"absolute_codec_string\" value=\"PCMU,PCMA\"/>",
        "<variable name=\"forward_busy_enabled\" value=\"true\"/>",
        "<variable name=\"forward_busy_destination\" value=\"1092\"/>",
        "sofia_contact",
        "pw-1091",
    ] {
        assert!(xml.contains(needle), "missing {needle}: {xml}");
    }

    let xml =
        fetch("/api/internal/freeswitch/xml?section=directory&key_value=1092@dirtest.example.com")
            .await;
    for needle in [
        "<param name=\"vm-password\" value=\"vm-1092\"/>",
        "<param name=\"vm-mailto\" value=\"ops@dirtest.example.com\"/>",
    ] {
        assert!(xml.contains(needle), "missing {needle}: {xml}");
    }

    // Foreign domain must not serve this tenant's password.
    let xml =
        fetch("/api/internal/freeswitch/xml?section=directory&key_value=1091@other.example.com")
            .await;
    assert!(!xml.contains("pw-1091"), "cross-domain leak: {xml}");
}

// RED: vars non-empty after boot-seed (seeds run at boot, not only in tests)
#[tokio::test]
async fn vars_nonempty_after_boot_seed() {
    pbx_api::seed_on_boot().await;
    let token = login_token().await;
    let (s, b, _) = get_json(pbx_api::test_app(), &token, "/api/v1/vars").await;
    assert_eq!(s, StatusCode::OK, "list vars: {b}");
    let total = b.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
    assert!(total >= 1, "vars must be seeded at boot, got: {b}");
}

// RED: sip_profiles/status returns {profiles, connected} (RealEs->mock fallback)
#[tokio::test]
async fn sip_profiles_status_returns_profiles_and_connected() {
    let token = login_token().await;
    let (s, b, _) = get_json(pbx_api::test_app(), &token, "/api/v1/sip_profiles/status").await;
    assert_eq!(s, StatusCode::OK, "sip status: {b}");
    assert!(
        b.get("profiles").and_then(|p| p.as_array()).is_some(),
        "profiles array: {b}"
    );
    assert!(
        b.get("connected").and_then(|c| c.as_bool()).is_some(),
        "connected flag: {b}"
    );
}

fn nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

async fn get_xml(token: &str, path: &str) -> String {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "GET {path}");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    String::from_utf8_lossy(&bytes).to_string()
}

// Task 5: gateway rows are served inside sofia.conf (per-profile,
// lowercase-uuid gateway name + params); DELETE invalidates the key so the
// gateway disappears on the next fetch.
#[tokio::test]
async fn configuration_gateway_served_inside_sofia() {
    let token = login_token().await;
    let n = nanos();
    let profile = format!("t5prof{n}");
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/sip_profiles",
        serde_json::json!({"name": profile}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "seed profile");

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/gateways",
        serde_json::json!({"gateway": format!("t5gw{n}"), "profile": profile, "username": "t5user", "password": "t5pw", "proxy": "sip.t5.example"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "seed gateway: {b}");
    let gid = b
        .get("gateway_uuid")
        .and_then(|v| v.as_str())
        .expect("gateway_uuid in response")
        .to_string();

    let xml = get_xml(
        &token,
        "/api/internal/freeswitch/xml?section=configuration&key_value=sofia.conf",
    )
    .await;
    assert!(
        xml.contains(&format!("<gateway name=\"{}\">", gid.to_lowercase())),
        "gateway element served: {xml}"
    );
    assert!(
        xml.contains("<param name=\"username\" value=\"t5user\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains(&format!("<profile name=\"{profile}\">")),
        "{xml}"
    );

    // DELETE -> per-key invalidate + killgw: gateway gone on next fetch.
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::delete(format!("/api/v1/gateways/{gid}"))
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT, "delete gateway");
    let xml = get_xml(
        &token,
        "/api/internal/freeswitch/xml?section=configuration&key_value=sofia.conf",
    )
    .await;
    assert!(
        !xml.contains(&gid.to_lowercase()),
        "deleted gateway must vanish: {xml}"
    );
}

// Task 5: vars rows served as vars.xml (hostname/null scope, Provision
// excluded); PATCH/DELETE invalidate per-key.
#[tokio::test]
async fn configuration_vars_served_from_rows() {
    let token = login_token().await;
    let n = nanos();
    let cat = format!("T5Cat{n}");
    let name = format!("t5var{n}");
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/vars",
        serde_json::json!({"var_category": cat, "var_name": name, "var_value": "t5val"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "seed var");
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/vars",
        serde_json::json!({"var_category": "Provision", "var_name": format!("t5prov{n}"), "var_value": "no"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "seed provision var");

    let xml = get_xml(
        &token,
        "/api/internal/freeswitch/xml?section=configuration&key_value=vars.xml",
    )
    .await;
    assert!(
        xml.contains(&format!(
            "<X-PRE-PROCESS cmd=\"set\" data=\"{name}=t5val\" />"
        )),
        "var served: {xml}"
    );
    assert!(
        !xml.contains(&format!("t5prov{n}")),
        "Provision excluded: {xml}"
    );
}

// Task 5: call-center rows served as callcenter.conf (queue@domain, agent
// contact call_timeout); CRUD invalidates per-key.
#[tokio::test]
async fn configuration_callcenter_served_from_rows() {
    let token = login_token().await;
    let n = nanos();
    let queue = format!("t5queue{n}");
    let agent = format!("t5agent{n}");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_centers",
        serde_json::json!({"queue_name": queue, "queue_strategy": "longest-idle-agent"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "seed queue: {b}");
    let qid = b
        .get("call_center_queue_uuid")
        .and_then(|v| v.as_str())
        .expect("queue uuid in response")
        .to_string();

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{qid}/agents"),
        serde_json::json!({"agent_name": agent, "agent_contact": "user/1001@example.com", "agent_call_timeout": 45}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "seed agent: {b}");

    let xml = get_xml(
        &token,
        "/api/internal/freeswitch/xml?section=configuration&key_value=callcenter.conf",
    )
    .await;
    assert!(
        xml.contains(&format!("<queue name=\"{queue}@")),
        "queue served: {xml}"
    );
    assert!(
        xml.contains(&format!("<agent name=\"{agent}@")),
        "agent served: {xml}"
    );
    assert!(
        xml.contains("contact=\"{call_timeout=45}user/1001@example.com\""),
        "agent contact timeout: {xml}"
    );
}
