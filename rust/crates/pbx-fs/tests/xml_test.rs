#[test]
fn directory_xml_contains_user_and_domain() {
    let out = pbx_fs::xml::build_directory_xml("example.com", "1000");
    assert!(
        out.contains("1000") && out.contains("example.com"),
        "got: {out}"
    );
    assert!(out.starts_with("<?xml"), "must be xml doc");
}

#[test]
fn dialplan_xml_contains_domain() {
    let out = pbx_fs::xml::build_dialplan_xml("example.com");
    assert!(out.contains("example.com"), "got: {out}");
    assert!(out.starts_with("<?xml"), "must be xml doc");
}
