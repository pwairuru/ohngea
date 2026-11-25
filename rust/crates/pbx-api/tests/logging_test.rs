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

use axum::response::IntoResponse as _;
use tower::ServiceExt as _;

/// Capturing layer for the probe tests: records `(level, message)` of every
/// event dispatched through the thread-local test subscriber.
#[derive(Clone, Default)]
struct ProbeCapture {
    events: std::sync::Arc<std::sync::Mutex<Vec<(tracing::Level, String)>>>,
}

struct ProbeVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for ProbeVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        }
    }
}

impl<S> tracing_subscriber::Layer<S> for ProbeCapture
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = ProbeVisitor { message: None };
        event.record(&mut visitor);
        self.events.lock().unwrap().push((
            *event.metadata().level(),
            visitor.message.unwrap_or_default(),
        ));
    }
}

fn probe_subscriber(capture: ProbeCapture) -> impl tracing::Subscriber {
    use tracing_subscriber::layer::SubscriberExt as _;
    tracing_subscriber::registry().with(capture)
}

#[tokio::test]
async fn inbound_request_id_echoed_verbatim() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .header("x-request-id", "probe-123")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        res.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("probe-123")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn inner_layer_403_carries_request_id_and_log_line() {
    // Inner-layer denial through the real production choke point
    // (`tenant::deny_if_missing` -> `forbidden`, no DB involved): a
    // zero-permission caller is denied 403 with a WARN log line, and the
    // outermost middleware still echoes `x-request-id` on the response.
    // (Deliberately DB-free: the shared single-connection sqlite pool
    // flakes under parallel load; the middleware echo is what is probed.)
    async fn deny() -> impl axum::response::IntoResponse {
        let claims = pbx_core::auth::Claims {
            sub: "u1".to_string(),
            domain_id: "default".to_string(),
            username: "tester".to_string(),
            permissions: vec![],
            exp: 0,
            iat: 0,
            jti: None,
        };
        match pbx_api::tenant::deny_if_missing(&claims, "extension_view") {
            Some((s, j)) => (s, j).into_response(),
            None => axum::http::StatusCode::OK.into_response(),
        }
    }
    let app = axum::Router::new()
        .route("/api/v1/extensions", axum::routing::get(deny))
        .layer(axum::middleware::from_fn(pbx_api::request_log_middleware));
    let capture = ProbeCapture::default();
    let sub = probe_subscriber(capture.clone());
    let _guard = tracing::subscriber::set_default(sub);
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/extensions")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::FORBIDDEN);
    assert!(res.headers().contains_key("x-request-id"));
    let events = capture.events.lock().unwrap();
    assert!(
        events
            .iter()
            .any(|(level, msg)| *level == tracing::Level::WARN && msg == "permission denied"),
        "expected WARN permission-denied event, got: {events:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn server_error_emits_error_event() {
    use axum::http::StatusCode;
    let failing = axum::Router::new()
        .route(
            "/boom",
            axum::routing::get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
        )
        .layer(axum::middleware::from_fn(pbx_api::request_log_middleware));
    let capture = ProbeCapture::default();
    let sub = probe_subscriber(capture.clone());
    let _guard = tracing::subscriber::set_default(sub);
    let res = failing
        .oneshot(
            axum::http::Request::builder()
                .uri("/boom")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(res.headers().contains_key("x-request-id"));
    let events = capture.events.lock().unwrap();
    assert!(
        events
            .iter()
            .any(|(level, msg)| *level == tracing::Level::ERROR && msg == "request failed"),
        "expected ERROR request-failed event, got: {events:?}"
    );
}

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
