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

async fn get(
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
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json, headers)
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

async fn patch_json(
    app: axum::Router,
    token: &str,
    path: &str,
    v: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(auth(
            Request::patch(path)
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

async fn delete(app: axum::Router, token: &str, path: &str) -> StatusCode {
    app.oneshot(auth(
        Request::delete(path).body(Body::empty()).unwrap(),
        token,
    ))
    .await
    .unwrap()
    .status()
}

// 1. voicemails: messages/options CRUD + playback + health
#[tokio::test]
async fn gaps2_voicemail_messages_options_playback_health() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/voicemails",
        serde_json::json!({"voicemail_id":"1001","voicemail_password":"1234"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create vm: {b}");
    let vm = b["voicemail_uuid"].as_str().unwrap_or("").to_string();
    assert!(!vm.is_empty(), "vm uuid: {b}");
    let (s, b, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get vm: {b}");
    let (s, b, _) = get(pbx_api::test_app(), &token, "/api/v1/voicemails/health").await;
    assert_eq!(s, StatusCode::OK, "vm health: {b}");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}/messages"),
        serde_json::json!({"file_name":"msg1.wav","message_length":60}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create msg: {b}");
    let msg = b["voicemail_message_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!msg.is_empty(), "msg uuid: {b}");
    let (s, b, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}/messages"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list msgs: {b}");
    assert!(b["total"].as_u64().unwrap_or(0) >= 1, "msgs total: {b}");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}/messages/{msg}"),
        serde_json::json!({"message_length":120}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch msg");
    let (s, b, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}/messages/{msg}/playback"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "playback: {b}");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}/options"),
        serde_json::json!({"digits":"0","action":"transfer","param":"1000"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create opt: {b}");
    let opt = b["voicemail_option_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!opt.is_empty(), "opt uuid: {b}");
    let (s, b, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}/options"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list opts: {b}");
    assert!(b["total"].as_u64().unwrap_or(0) >= 1, "opts total: {b}");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}/options/{opt}"),
        serde_json::json!({"param":"1002"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch opt");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/voicemails/{vm}/options/{opt}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete opt"
    );
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/voicemails/{vm}/messages/{msg}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete msg"
    );
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm}"),
        serde_json::json!({"description":"hi"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch vm");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/voicemails/{vm}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete vm"
    );
}

// 1b. voicemail_greetings standalone CRUD
#[tokio::test]
async fn gaps2_voicemail_greetings_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/voicemails",
        serde_json::json!({"voicemail_id":"1009","voicemail_password":"1234"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create vm: {b}");
    let vm = b["voicemail_uuid"].as_str().unwrap_or("").to_string();
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/voicemail_greetings",
        serde_json::json!({"voicemail_uuid": vm, "voicemail_id":"1009","greeting_name":"busy"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create greeting: {b}");
    let id = b["voicemail_greeting_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!id.is_empty(), "greeting uuid: {b}");
    let (s, _, _) = get(pbx_api::test_app(), &token, "/api/v1/voicemail_greetings").await;
    assert_eq!(s, StatusCode::OK, "list greetings");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemail_greetings/{id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get greeting");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemail_greetings/{id}"),
        serde_json::json!({"greeting_name":"unavail"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch greeting");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/voicemail_greetings/{id}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete greeting"
    );
}

// 2. call_centers: queue get/patch/delete + agents/tiers get/patch/delete
#[tokio::test]
async fn gaps2_call_centers_full() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_centers",
        serde_json::json!({"queue_name":"support-q","queue_extension":"5000"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create queue: {b}");
    let q = b["call_center_queue_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!q.is_empty(), "queue uuid: {b}");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get queue");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}"),
        serde_json::json!({"queue_name":"support-q2"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch queue");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}/agents"),
        serde_json::json!({"agent_name":"agent1","agent_contact":"1001"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create agent: {b}");
    let a = b["call_center_agent_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!a.is_empty(), "agent uuid: {b}");
    let (s, b, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}/agents"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list agents: {b}");
    assert!(b["total"].as_u64().unwrap_or(0) >= 1, "agents total: {b}");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}/agents/{a}"),
        serde_json::json!({"agent_status":"busy"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch agent");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}/tiers"),
        serde_json::json!({"agent_name":"agent1","tier_level":1,"tier_position":1}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create tier: {b}");
    let t = b["call_center_tier_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!t.is_empty(), "tier uuid: {b}");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}/tiers"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list tiers");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{q}/tiers/{t}"),
        serde_json::json!({"tier_level":2}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch tier");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/call_centers/{q}/tiers/{t}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete tier"
    );
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/call_centers/{q}/agents/{a}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete agent"
    );
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/call_centers/{q}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete queue"
    );
}

// 3. conference_centers full resource
#[tokio::test]
async fn gaps2_conference_centers_full() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/conference_centers",
        serde_json::json!({"conference_center_name":"main","conference_center_extension":"3000"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create center: {b}");
    let c = b["conference_center_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!c.is_empty(), "center uuid: {b}");
    let (s, _, _) = get(pbx_api::test_app(), &token, "/api/v1/conference_centers").await;
    assert_eq!(s, StatusCode::OK, "list centers");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get center");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}"),
        serde_json::json!({"conference_center_description":"d"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch center");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/rooms"),
        serde_json::json!({"conference_room_name":"room1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create room: {b}");
    let r = b["conference_room_uuid"].as_str().unwrap_or("").to_string();
    assert!(!r.is_empty(), "room uuid: {b}");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/rooms"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list rooms");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/rooms/{r}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get room");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/rooms/{r}"),
        serde_json::json!({"max_members":10}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch room");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/sessions"),
        serde_json::json!({"conference_room_uuid": r}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create session: {b}");
    let sess = b["conference_session_uuid"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(!sess.is_empty(), "session uuid: {b}");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/sessions"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list sessions");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/sessions/{sess}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get session");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_centers/{c}/sessions/{sess}/details"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "session details");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/conference_centers/{c}/sessions/{sess}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete session"
    );
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/conference_centers/{c}/rooms/{r}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete room"
    );
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/conference_centers/{c}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete center"
    );
}

// 4. fax + fax_queue
#[tokio::test]
async fn gaps2_fax_full() {
    let token = login_token().await;
    let (s, _, _) = get(pbx_api::test_app(), &token, "/api/v1/fax/health").await;
    assert_eq!(s, StatusCode::OK, "fax health");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/fax",
        serde_json::json!({"fax_extension":"2001","fax_name":"fax1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create fax: {b}");
    let f = b["fax_uuid"].as_str().unwrap_or("").to_string();
    assert!(!f.is_empty(), "fax uuid: {b}");
    let (s, _, _) = get(pbx_api::test_app(), &token, "/api/v1/fax").await;
    assert_eq!(s, StatusCode::OK, "list fax");
    let (s, _, _) = get(pbx_api::test_app(), &token, &format!("/api/v1/fax/{f}")).await;
    assert_eq!(s, StatusCode::OK, "get fax");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax/{f}"),
        serde_json::json!({"description":"d"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch fax");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax/{f}/files"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "fax files");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax/{f}/logs"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "fax logs");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax/{f}/users"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "fax users");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax/{f}/files"),
        serde_json::json!({"file_name":"a.pdf"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "upload file: {b}");
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax/{f}/send"),
        serde_json::json!({"destination":"15551234567"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::OK,
        "send fax: {s}"
    );
    // fax_queue
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/fax_queue",
        serde_json::json!({"fax_extension":"2001","destination":"15551234567"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create queue: {b}");
    let qid = b
        .get("fax_queue_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!qid.is_empty(), "queue uuid: {b}");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax_queue/{qid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get queue");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax_queue/{qid}"),
        serde_json::json!({"status":"sending"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch queue");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax_queue/{qid}/status"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "queue status");
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/fax_queue/{qid}/retry"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "queue retry");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/fax_queue/{qid}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete queue"
    );
    assert_eq!(
        delete(pbx_api::test_app(), &token, &format!("/api/v1/fax/{f}")).await,
        StatusCode::NO_CONTENT,
        "delete fax"
    );
}

// 5. operator_panel
#[tokio::test]
async fn gaps2_operator_panel_actions() {
    let token = login_token().await;
    let (s, b, _) = get(pbx_api::test_app(), &token, "/api/v1/operator_panel/calls").await;
    assert_eq!(s, StatusCode::OK, "op calls: {b}");
    assert!(b.get("data").is_some(), "envelope: {b}");
    for action in ["hangup", "eavesdrop", "coach", "record"] {
        let (s, b) = post_json(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/operator_panel/{action}/some-uid"),
            serde_json::json!({}),
        )
        .await;
        assert!(
            s == StatusCode::OK || s == StatusCode::CREATED,
            "{action}: {s} {b}"
        );
    }
}

// 6. contacts children
#[tokio::test]
async fn gaps2_contacts_children() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/contacts",
        serde_json::json!({"name":"Bob","number":"1002"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create contact: {b}");
    let c = b["contact_uuid"].as_str().unwrap_or("").to_string();
    for child in ["addresses", "phones", "emails", "urls", "notes"] {
        let (s, b) = post_json(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/contacts/{c}/{child}"),
            serde_json::json!({"value":"x","phone_number":"1002"}),
        )
        .await;
        assert_eq!(s, StatusCode::CREATED, "create {child}: {b}");
        let (s, b, _) = get(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/contacts/{c}/{child}"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "list {child}: {b}");
        assert!(b.get("total").is_some(), "total: {b}");
    }
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/contacts/{c}"),
        serde_json::json!({"organization":"acme"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch contact");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/contacts/{c}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete contact"
    );
}

// 7. devices sub-resources + profiles
#[tokio::test]
async fn gaps2_devices_subresources() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/devices",
        serde_json::json!({"mac":"AA:BB:CC:DD:EE:99","vendor":"yealink"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::CONFLICT,
        "create dev: {b}"
    );
    // fetch id from list (conflict-safe)
    let (_, bl, _) = get(pbx_api::test_app(), &token, "/api/v1/devices").await;
    let id = bl["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|d| {
            if d["mac"].as_str() == Some("aa:bb:cc:dd:ee:99") {
                d["device_uuid"].as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();
    assert!(!id.is_empty(), "device id: {bl}");
    for child in ["keys", "lines", "settings"] {
        let (s, b) = post_json(pbx_api::test_app(), &token, &format!("/api/v1/devices/{id}/{child}"), serde_json::json!({"device_key_value":"v","line_number":"2","device_setting_name":"n","device_setting_value":"v"})).await;
        assert_eq!(s, StatusCode::CREATED, "create {child}: {b}");
        let key = format!("device_{}_uuid", child.trim_end_matches('s'));
        // lines use device_line_uuid, settings device_setting_uuid
        let cid = b
            .get(&key)
            .or_else(|| b.get("device_key_uuid"))
            .or_else(|| b.get("device_line_uuid"))
            .or_else(|| b.get("device_setting_uuid"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let (s, _, _) = get(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/devices/{id}/{child}"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "list {child}");
        if !cid.is_empty() {
            let (s, _) = patch_json(
                pbx_api::test_app(),
                &token,
                &format!("/api/v1/devices/{id}/{child}/{cid}"),
                serde_json::json!({"enabled":true}),
            )
            .await;
            assert_eq!(s, StatusCode::OK, "patch {child}");
            assert_eq!(
                delete(
                    pbx_api::test_app(),
                    &token,
                    &format!("/api/v1/devices/{id}/{child}/{cid}")
                )
                .await,
                StatusCode::NO_CONTENT,
                "delete {child}"
            );
        }
    }
    // device_profiles
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/device_profiles",
        serde_json::json!({"device_profile_name":"prof1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create profile: {b}");
    let p = b["device_profile_uuid"].as_str().unwrap_or("").to_string();
    assert!(!p.is_empty(), "profile uuid: {b}");
    let (s, _, _) = get(pbx_api::test_app(), &token, "/api/v1/device_profiles").await;
    assert_eq!(s, StatusCode::OK, "list profiles");
    let (s, _, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/device_profiles/{p}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get profile");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/device_profiles/{p}"),
        serde_json::json!({"device_profile_description":"d"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch profile");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/device_profiles/{p}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete profile"
    );
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/devices/{id}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete device"
    );
}

// 8. ivr menus patch/delete + options
#[tokio::test]
async fn gaps2_ivr_full() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/ivr_menus",
        serde_json::json!({"name":"main-ivr"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create ivr: {b}");
    let m = b["ivr_menu_uuid"].as_str().unwrap_or("").to_string();
    assert!(!m.is_empty(), "ivr uuid: {b}");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ivr_menus/{m}"),
        serde_json::json!({"description":"d"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch ivr");
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ivr_menus/{m}/options"),
        serde_json::json!({"digits":"1","action":"transfer","param":"1001"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create opt: {b}");
    let o = b
        .get("option_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!o.is_empty(), "opt uuid: {b}");
    let (s, b, _) = get(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ivr_menus/{m}/options"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list opts: {b}");
    assert!(b["total"].as_u64().unwrap_or(0) >= 1, "opts total: {b}");
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ivr_menus/{m}/options/{o}"),
        serde_json::json!({"param":"1002"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch opt");
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/ivr_menus/{m}/options/{o}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete opt"
    );
    assert_eq!(
        delete(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/ivr_menus/{m}")
        )
        .await,
        StatusCode::NO_CONTENT,
        "delete ivr"
    );
}

// 9. webrtc credentials + registrations provision/reboot
#[tokio::test]
async fn gaps2_webrtc_registrations() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/webrtc/credentials",
        serde_json::json!({"extension":"1001"}),
    )
    .await;
    assert!(
        s == StatusCode::OK || s == StatusCode::BAD_REQUEST,
        "credentials: {s} {b}"
    );
    for action in ["provision", "reboot"] {
        let (s, b) = post_json(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/registrations/some-uid/{action}"),
            serde_json::json!({}),
        )
        .await;
        assert!(
            s == StatusCode::OK || s == StatusCode::CREATED,
            "{action}: {s} {b}"
        );
    }
}

// 10. systemic: PATCH/DELETE + headers on representative resources
#[tokio::test]
async fn gaps2_systemic_patch_delete_headers() {
    let token = login_token().await;
    // headers on lists
    for path in [
        "/api/v1/call_block",
        "/api/v1/bridges",
        "/api/v1/destinations",
        "/api/v1/contacts",
        "/api/v1/ivr_menus",
        "/api/v1/voicemails",
        "/api/v1/conferences",
        "/api/v1/call_centers",
    ] {
        let (s, b, h) = get(pbx_api::test_app(), &token, path).await;
        assert_eq!(s, StatusCode::OK, "{path}: {b}");
        assert!(
            h.contains_key("x-total-count"),
            "{path} missing X-Total-Count"
        );
        assert!(
            b.get("page").is_some() && b.get("per_page").is_some(),
            "{path} missing page/per_page: {b}"
        );
    }
    // PATCH/DELETE on gateways, dialplans, ring_groups, music_on_hold, time_conditions
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/gateways",
        serde_json::json!({"gateway_name":"gw1","proxy":"1.2.3.4"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "gw create: {b}");
    let id = b
        .get("gateway_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !id.is_empty() {
        let (s, _) = patch_json(
            pbx_api::test_app(),
            &token,
            &format!("/api/v1/gateways/{id}"),
            serde_json::json!({"description":"d"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "gw patch");
        assert_eq!(
            delete(
                pbx_api::test_app(),
                &token,
                &format!("/api/v1/gateways/{id}")
            )
            .await,
            StatusCode::NO_CONTENT,
            "gw delete"
        );
    }
}
