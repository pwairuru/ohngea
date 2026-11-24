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
