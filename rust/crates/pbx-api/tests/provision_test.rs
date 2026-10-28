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

async fn provision_for(vendor: &str, mac: &str, model: Option<&str>) -> String {
    let token = login_token().await;
    let app = pbx_api::test_app();
    let mut payload = serde_json::json!({
        "mac": mac,
        "vendor": vendor,
        "extension": "1001",
        "domain": "example.com",
    });
    if let Some(m) = model {
        payload["model"] = serde_json::Value::String(m.to_owned());
    }
    let res = app
        .oneshot(
            Request::post("/api/v1/devices")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::CREATED
            || res.status() == StatusCode::OK
            || res.status() == StatusCode::CONFLICT,
        "create {vendor} {mac}: {}",
        res.status()
    );
    let app2 = pbx_api::test_app();
    let res = app2
        .oneshot(
            Request::get(format!("/provision/{mac}.cfg"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "provision {vendor} {mac}");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    String::from_utf8_lossy(&bytes).to_string()
}

// (vendor, mac_suffix, identifier)
const VENDORS: &[(&str, &str, &str)] = &[
    ("2n", "02", "<device vendor=\"2n\""),
    ("aastra", "03", "<device vendor=\"aastra\""),
    ("acrobits", "04", "<device vendor=\"acrobits\""),
    ("algo", "05", "<device vendor=\"algo\""),
    ("atcom", "06", "<device vendor=\"atcom\""),
    ("avaya", "07", "<device vendor=\"avaya\""),
    ("cisco", "08", "<device vendor=\"cisco\""),
    ("digium", "09", "<device vendor=\"digium\""),
    ("escene", "0a", "<device vendor=\"escene\""),
    ("fanvil", "0b", "<device vendor=\"fanvil\""),
    ("flyingvoice", "0c", "<device vendor=\"flyingvoice\""),
    ("grandstream", "0d", "<device vendor=\"grandstream\""),
    ("groundwire", "0e", "<device vendor=\"groundwire\""),
    ("htek", "0f", "<device vendor=\"htek\""),
    ("linksys", "10", "<device vendor=\"linksys\""),
    ("linphone", "11", "<device vendor=\"linphone\""),
    ("mitel", "12", "<device vendor=\"mitel\""),
    ("obihai", "13", "<device vendor=\"obihai\""),
    ("panasonic", "14", "<device vendor=\"panasonic\""),
    ("poly", "15", "<poly>"),
    // polycom aliases to poly per app/devices/provision.py (_vendor_candidates)
    ("polycom", "16", "<poly>"),
    ("sangoma", "17", "<device vendor=\"sangoma\""),
    ("sipnetic", "18", "<device vendor=\"sipnetic\""),
    ("snom", "19", "<device vendor=\"snom\""),
    ("spectralink", "1a", "<device vendor=\"spectralink\""),
    ("swissvoice", "1b", "<device vendor=\"swissvoice\""),
    ("telekonnectors", "1c", "<device vendor=\"telekonnectors\""),
    ("vtech", "1d", "<device vendor=\"vtech\""),
    ("yealink", "1e", "account.1"),
    ("yeastar", "1f", "<device vendor=\"yeastar\""),
    ("zoiper", "20", "<device vendor=\"zoiper\""),
];

#[tokio::test]
async fn all_31_vendors_render() {
    for (vendor, suffix, ident) in VENDORS {
        let mac = format!("aa:bb:cc:dd:ee:{suffix}");
        let text = provision_for(vendor, &mac, None).await;
        assert!(
            text.contains(ident),
            "vendor {vendor} must contain identifier {ident:?}, got:\n{text}"
        );
        assert!(
            text.contains(&mac) || text.contains(&mac.replace(':', "-")),
            "vendor {vendor} must contain MAC, got:\n{text}"
        );
    }
}

#[tokio::test]
async fn model_overrides_render() {
    let cases = [
        ("yealink", "T46S", "aa:bb:cc:dd:f1:01", "T46S"),
        ("yealink", "T29G", "aa:bb:cc:dd:f1:02", "T29G"),
        ("cisco", "SPA122", "aa:bb:cc:dd:f1:03", "flat-profile"),
        ("grandstream", "GXP2170", "aa:bb:cc:dd:f1:04", "P2 ="),
        ("poly", "VVX250", "aa:bb:cc:dd:f1:05", "poly-vvx250"),
        ("poly", "VVX450", "aa:bb:cc:dd:f1:06", "poly-vvx450"),
    ];
    for (vendor, model, mac, ident) in cases {
        let text = provision_for(vendor, mac, Some(model)).await;
        assert!(
            text.contains(ident),
            "model override {vendor}/{model} must contain {ident:?}, got:\n{text}"
        );
    }
}
