use pbx_fs::esl::{is_connected_body, strip_esl_body};

#[test]
fn strip_esl_body_removes_headers() {
    let raw = "Content-Type: api/response\nContent-Length: 54\n\nUP 0 years";
    assert_eq!(strip_esl_body(raw), "UP 0 years");
}

#[test]
fn is_connected_body_accepts_up_status() {
    assert!(is_connected_body(
        "UP 0 years, 10 days, 1 hour, 2 minutes, 30 seconds"
    ));
}

#[test]
fn is_connected_body_rejects_empty() {
    assert!(!is_connected_body(""));
}
