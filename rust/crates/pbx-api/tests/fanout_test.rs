//! Task 6: app → dialplan fan-out (RG/IVR/conference/call-center) + tones.
//!
//! PHP ground truth:
//! - `fusionpbx/app/ring_groups/ring_group_edit.php:535-560`
//!   (app `1d61fb65-…`, order 101, `lua app.lua ring_groups`)
//! - `fusionpbx/app/ivr_menus/ivr_menu_edit.php:323-462`
//!   (app `a5788e9b-…`, order 101, `lua ivr_menu.lua`)
//! - `fusionpbx/app/conferences/conference_edit.php:234-252`
//!   (app `b81412e8-…`, order 333, `conference ext@domain@profile+pin`)
//! - `fusionpbx/app/call_centers/call_center_queue_edit.php:446-508`
//!   (app `95788e50-…`, order 230, `callcenter ext@domain`)
//! - `fusionpbx/app/tones/app_config.php` (Tones = `v_vars` category)

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

const RG_APP: &str = "1d61fb65-1eec-bc73-a6ee-a6203b4fe6f2";
const IVR_APP: &str = "a5788e9b-58bc-bd1b-df59-fff5d51253ab";
const CONF_APP: &str = "b81412e8-7253-91f4-e48e-42fc2c9a38d9";
const CC_APP: &str = "95788e50-9500-079e-2807-fd530b0ea370";

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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn get_json(
    app: axum::Router,
    token: &str,
    path: &str,
) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn delete_req(app: axum::Router, token: &str, path: &str) -> StatusCode {
    let res = app
        .oneshot(
            auth(
                Request::delete(path).body(Body::empty()).unwrap(),
                token,
            ),
        )
        .await
        .unwrap();
    res.status()
}

async fn dialplan_xml(token: &str) -> String {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::get("/api/internal/freeswitch/xml?section=dialplan&key_value=default")
                .body(Body::empty())
                .unwrap(),
            token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    String::from_utf8_lossy(&bytes).to_string()
}

async fn dialplan_for_number(token: &str, number: &str) -> serde_json::Value {
    let (s, b) = get_json(pbx_api::test_app(), token, "/api/v1/dialplans").await;
    assert_eq!(s, StatusCode::OK, "list dialplans: {b}");
    b["data"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .find(|d| d.get("dialplan_number").and_then(|n| n.as_str()) == Some(number))
        .unwrap_or(serde_json::Value::Null)
}

// (1) Ring group fan-out: create mints dialplan 101 + lua glue,
// update refreshes it, delete removes it.
#[tokio::test]
async fn rg_fanout_mints_updates_deletes_dialplan() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/ring_groups",
        serde_json::json!({
            "name": "sales-rg-6101", "extension": "6101",
            "members": [{"destination": "1001"}],
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create rg: {b}");
    let rg_id = b["ring_group_uuid"].as_str().unwrap().to_string();

    let (s, rg) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ring_groups/{rg_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get rg: {rg}");
    let dp_id = rg["dialplan_uuid"].as_str().unwrap_or("").to_string();
    assert!(!dp_id.is_empty(), "rg must link a dialplan: {rg}");

    let (s, dp) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{dp_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "linked dialplan: {dp}");
    assert_eq!(dp["app_uuid"].as_str().unwrap_or(""), RG_APP, "rg app_uuid: {dp}");
    assert_eq!(dp["dialplan_order"].as_i64().unwrap_or(-1), 101, "rg order: {dp}");
    assert_eq!(dp["dialplan_number"].as_str().unwrap_or(""), "6101");
    let det = dp["details"].as_array().cloned().unwrap_or_default();
    assert!(
        det.iter().any(|d| d["data"].as_str().unwrap_or("").contains("app.lua ring_groups")),
        "rg lua glue missing: {dp}"
    );

    let xml = dialplan_xml(&token).await;
    assert!(xml.contains("6101"), "served dialplan must contain 6101");
    assert!(xml.contains("app.lua ring_groups"), "served dialplan must contain lua glue");

    // update refreshes the linked row
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ring_groups/{rg_id}"),
        serde_json::json!({"name": "sales-rg-6101b"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, dp2) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{dp_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(dp2["name"].as_str().unwrap_or(""), "sales-rg-6101b");

    // delete removes the linked row
    assert_eq!(
        delete_req(pbx_api::test_app(), &token, &format!("/api/v1/ring_groups/{rg_id}")).await,
        StatusCode::NO_CONTENT
    );
    let (s, _) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{dp_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "dialplan must die with the ring group");
}

// (2) IVR fan-out: lua ivr_menu.lua, order 101, extension update re-points.
#[tokio::test]
async fn ivr_fanout_mints_updates_deletes_dialplan() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/ivr_menus",
        serde_json::json!({"name": "main-ivr-6201", "extension": "6201"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create ivr: {b}");
    let ivr_id = b["ivr_menu_uuid"].as_str().unwrap().to_string();

    let dp = dialplan_for_number(&token, "6201").await;
    assert!(dp.is_object(), "ivr must mint dialplan 6201");
    let dp_id = dp["dialplan_uuid"].as_str().unwrap().to_string();
    assert_eq!(dp["app_uuid"].as_str().unwrap_or(""), IVR_APP);
    assert_eq!(dp["dialplan_order"].as_i64().unwrap_or(-1), 101);
    let det = dp["details"].as_array().cloned().unwrap_or_default();
    assert!(
        det.iter().any(|d| d["data"].as_str().unwrap_or("").contains("ivr_menu.lua")),
        "ivr lua glue missing: {dp}"
    );
    let xml = dialplan_xml(&token).await;
    assert!(xml.contains("6201") && xml.contains("ivr_menu.lua"));

    // extension update re-points the linked row
    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ivr_menus/{ivr_id}"),
        serde_json::json!({"extension": "6202"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let dp_new = dialplan_for_number(&token, "6202").await;
    assert!(dp_new.is_object(), "ivr dialplan must follow extension 6202: {dp_new}");
    assert_eq!(dp_new["dialplan_uuid"].as_str().unwrap_or(""), dp_id);

    assert_eq!(
        delete_req(pbx_api::test_app(), &token, &format!("/api/v1/ivr_menus/{ivr_id}")).await,
        StatusCode::NO_CONTENT
    );
    let (s, _) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{dp_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "dialplan must die with the ivr");
}

// (3) Conference fan-out: conference ext@domain@profile, order 333 (PHP).
#[tokio::test]
async fn conference_fanout_mints_updates_deletes_dialplan() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/conferences",
        serde_json::json!({
            "conference_name": "team-6301", "conference_extension": "6301",
            "conference_profile": "default",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create conf: {b}");
    let conf_id = b["conference_uuid"].as_str().unwrap().to_string();

    let dp = dialplan_for_number(&token, "6301").await;
    assert!(dp.is_object(), "conference must mint dialplan 6301");
    let dp_id = dp["dialplan_uuid"].as_str().unwrap().to_string();
    assert_eq!(dp["app_uuid"].as_str().unwrap_or(""), CONF_APP);
    assert_eq!(dp["dialplan_order"].as_i64().unwrap_or(-1), 333, "PHP conference order is 333: {dp}");
    let det = dp["details"].as_array().cloned().unwrap_or_default();
    let conf_action = det.iter().find(|d| {
        d["tag"].as_str() == Some("action")
            && d["type"].as_str() == Some("conference")
    });
    assert!(conf_action.is_some(), "conference app action missing: {dp}");
    let data = conf_action.unwrap()["data"].as_str().unwrap_or("");
    assert!(data.contains("6301@") && data.contains("@default"), "conference data ext@domain@profile: {data}");
    let xml = dialplan_xml(&token).await;
    assert!(xml.contains("6301") && xml.contains("conference"));

    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conferences/{conf_id}"),
        serde_json::json!({"conference_extension": "6302"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let dp_new = dialplan_for_number(&token, "6302").await;
    assert!(dp_new.is_object(), "conference dialplan must follow 6302");

    assert_eq!(
        delete_req(pbx_api::test_app(), &token, &format!("/api/v1/conferences/{conf_id}")).await,
        StatusCode::NO_CONTENT
    );
    let (s, _) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{dp_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "dialplan must die with the conference");
}

// (4) Call-center fan-out only: queue create mints order-230 callcenter
// dialplan; delete removes it (no config-builder changes here).
#[tokio::test]
async fn callcenter_fanout_mints_and_deletes_dialplan() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_centers",
        serde_json::json!({"queue_name": "support-6401", "queue_extension": "6401"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create queue: {b}");
    let qid = b["call_center_queue_uuid"].as_str().unwrap().to_string();

    let (s, q) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{qid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get queue: {q}");
    let dp_id = q["dialplan_uuid"].as_str().unwrap_or("").to_string();
    assert!(!dp_id.is_empty(), "queue must link a dialplan: {q}");

    let (s, dp) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{dp_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "linked dialplan: {dp}");
    assert_eq!(dp["app_uuid"].as_str().unwrap_or(""), CC_APP);
    assert_eq!(dp["dialplan_order"].as_i64().unwrap_or(-1), 230);
    let det = dp["details"].as_array().cloned().unwrap_or_default();
    assert!(
        det.iter().any(|d| d["type"].as_str() == Some("callcenter")),
        "callcenter app action missing: {dp}"
    );
    let xml = dialplan_xml(&token).await;
    assert!(xml.contains("6401") && xml.contains("callcenter"));

    assert_eq!(
        delete_req(pbx_api::test_app(), &token, &format!("/api/v1/call_centers/{qid}")).await,
        StatusCode::NO_CONTENT
    );
    let (s, _) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{dp_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "dialplan must die with the queue");
}

// (5) Tones: GET list from v_vars category=Tones (PHP app_config parity).
#[tokio::test]
async fn tones_lists_tone_category_vars() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/vars",
        serde_json::json!({
            "var_category": "Tones", "var_name": "us-ring-6601",
            "var_value": "tone_stream://%(2000,4000,440,480)",
            "var_description": "fanout test tone",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "seed tone var: {b}");

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/tones").await;
    assert_eq!(s, StatusCode::OK, "list tones: {b}");
    let data = b["data"].as_array().cloned().unwrap_or_default();
    let hit = data.iter().find(|t| t["name"].as_str() == Some("us-ring-6601"));
    assert!(hit.is_some(), "tones must include Tones-category var: {b}");
    let hit = hit.unwrap();
    assert_eq!(
        hit["destination"].as_str().unwrap_or(""),
        "tone_stream://%(2000,4000,440,480)"
    );
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);

    // trailing-slash alias
    let (s, _) = get_json(pbx_api::test_app(), &token, "/api/v1/tones/").await;
    assert_eq!(s, StatusCode::OK);
}

// (6) Voicemail CRUD keeps serving (invalidate path exercised, no join change).
#[tokio::test]
async fn voicemail_crud_ok_with_invalidate() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/voicemails",
        serde_json::json!({"voicemail_id": "6501", "voicemail_password": "1234"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create vm: {b}");
    let vm_id = b["voicemail_uuid"].as_str().unwrap().to_string();

    let (s, _) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{vm_id}"),
        serde_json::json!({"description": "fanout vm"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);

    assert_eq!(
        delete_req(pbx_api::test_app(), &token, &format!("/api/v1/voicemails/{vm_id}")).await,
        StatusCode::NO_CONTENT
    );
}
