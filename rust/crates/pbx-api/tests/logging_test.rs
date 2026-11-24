#[test]
fn invalid_log_level_falls_back_to_default() {
    assert_eq!(
        pbx_api::logging::resolve_filter(Some("verbose")),
        pbx_api::logging::default_filter()
    );
}

#[test]
fn redact_esl_masks_secret() {
    let out = pbx_api::logging::redact_esl("sofia profile internal rescan password=hunter2");
    assert!(!out.contains("hunter2"));
}

#[test]
fn init_for_tests_is_idempotent() {
    pbx_api::logging::init_for_tests();
    pbx_api::logging::init_for_tests();
}

use tower::ServiceExt as _;

#[tokio::test]
async fn request_id_generated_and_echoed() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res.headers().contains_key("x-request-id"));
}

#[tokio::test]
async fn query_string_never_logged_path_only() {
    // path source must be MatchedPath or uri.path(), never the query string.
    // NOTE: brief snippet shows a single-arg call; the plan's pinned
    // signature is `logged_path(Option<&str>, &str)` (matched template wins,
    // else raw path), so the test passes `None` + a query-bearing path.
    assert!(
        !pbx_api::logging::logged_path(None, "/api/v1/extensions?token=secret").contains("token")
    );
}
