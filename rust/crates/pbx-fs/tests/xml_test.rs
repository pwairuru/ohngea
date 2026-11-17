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

fn enriched_user() -> pbx_fs::xml::DirectoryUser {
    pbx_fs::xml::DirectoryUser {
        id: "1001".to_string(),
        password: "s3cret-pw".to_string(),
        number_alias: Some("1001-alias".to_string()),
        cidr: Some("10.0.0.0/8".to_string()),
        dial_string: None,
        call_timeout: Some(30),
        user_context: Some("example.com".to_string()),
        domain_uuid: Some("dom-uuid-1".to_string()),
        extension_uuid: Some("ext-uuid-1".to_string()),
        accountcode: Some("acct1".to_string()),
        toll_allow: Some("domestic".to_string()),
        call_group: Some("sales".to_string()),
        call_screen_enabled: false,
        effective_caller_id_name: Some("Alice".to_string()),
        effective_caller_id_number: Some("1001".to_string()),
        outbound_caller_id_name: None,
        outbound_caller_id_number: None,
        emergency_caller_id_name: None,
        emergency_caller_id_number: None,
        directory_first_name: Some("Alice".to_string()),
        directory_last_name: Some("Smith".to_string()),
        directory_visible: true,
        directory_exten_visible: false,
        limit_max: None,
        limit_destination: Some("9999".to_string()),
        missed_call_app: None,
        missed_call_data: None,
        user_record: None,
        hold_music: None,
        auth_acl: Some("lan".to_string()),
        sip_force_contact: None,
        sip_force_expires: None,
        nibble_account: None,
        mwi_account: Some("1001@example.com".to_string()),
        sip_bypass_media: Some("bypass-media".to_string()),
        absolute_codec_string: Some("PCMU,PCMA".to_string()),
        force_ping: false,
        max_registrations: Some("1".to_string()),
        forward_all_enabled: false,
        forward_all_destination: None,
        forward_busy_enabled: true,
        forward_busy_destination: Some("1002".to_string()),
        forward_no_answer_enabled: false,
        forward_no_answer_destination: None,
        forward_user_not_registered_enabled: false,
        forward_user_not_registered_destination: None,
        follow_me_enabled: true,
        do_not_disturb: true,
        extension_language: Some("en".to_string()),
        extension_dialect: None,
        extension_voice: None,
        vm_password: Some("1234".to_string()),
        vm_enabled: "true".to_string(),
        vm_mail_to: Some("alice@example.com".to_string()),
        vm_attach_file: Some("true".to_string()),
        vm_keep_local_after_email: Some("true".to_string()),
        settings: vec![
            pbx_fs::xml::ExtSetting {
                setting_type: "param".to_string(),
                name: "custom-param".to_string(),
                value: "pval".to_string(),
            },
            pbx_fs::xml::ExtSetting {
                setting_type: "variable".to_string(),
                name: "custom-var".to_string(),
                value: "vval".to_string(),
            },
        ],
    }
}

#[test]
fn directory_enriched_user_attrs_alias_cidr() {
    let out = pbx_fs::xml::build_directory_xml_users("example.com", &[enriched_user()]);
    assert!(out.contains("number-alias=\"1001-alias\""), "alias attr: {out}");
    assert!(out.contains("cidr=\"10.0.0.0/8\""), "cidr attr: {out}");
}

#[test]
fn directory_enriched_vm_params_present() {
    let out = pbx_fs::xml::build_directory_xml_users("example.com", &[enriched_user()]);
    for needle in [
        "<param name=\"vm-password\" value=\"1234\"/>",
        "<param name=\"vm-mailto\" value=\"alice@example.com\"/>",
        "<param name=\"vm-attach-file\" value=\"true\"/>",
        "<param name=\"vm-keep-local-after-email\" value=\"true\"/>",
        "<param name=\"vm-enabled\" value=\"true\"/>",
        "<param name=\"MWI-Account\" value=\"1001@example.com\"/>",
        "<param name=\"auth-acl\" value=\"lan\"/>",
        "<param name=\"custom-param\" value=\"pval\"/>",
    ] {
        assert!(out.contains(needle), "missing {needle}: {out}");
    }
}

#[test]
fn directory_enriched_variables_block() {
    let out = pbx_fs::xml::build_directory_xml_users("example.com", &[enriched_user()]);
    for needle in [
        "<variable name=\"toll_allow\" value=\"domestic\"/>",
        "<variable name=\"accountcode\" value=\"acct1\"/>",
        "<variable name=\"call_group\" value=\"sales\"/>",
        "<variable name=\"user_context\" value=\"example.com\"/>",
        "<variable name=\"limit_max\" value=\"5\"/>",
        "<variable name=\"limit_destination\" value=\"9999\"/>",
        "<variable name=\"effective_caller_id_name\" value=\"Alice\"/>",
        "<variable name=\"forward_busy_enabled\" value=\"true\"/>",
        "<variable name=\"forward_busy_destination\" value=\"1002\"/>",
        "<variable name=\"do_not_disturb\" value=\"true\"/>",
        "<variable name=\"follow_me_enabled\" value=\"true\"/>",
        "<variable name=\"bypass_media\" value=\"true\"/>",
        "<variable name=\"absolute_codec_string\" value=\"PCMU,PCMA\"/>",
        "<variable name=\"directory_full_name\" value=\"Alice Smith\"/>",
        "<variable name=\"call_timeout\" value=\"30\"/>",
        "<variable name=\"custom-var\" value=\"vval\"/>",
        "<variable name=\"domain_uuid\" value=\"dom-uuid-1\"/>",
        "<variable name=\"extension_uuid\" value=\"ext-uuid-1\"/>",
    ] {
        assert!(out.contains(needle), "missing {needle}: {out}");
    }
}

#[test]
fn directory_default_dial_string_exact_shape() {
    let mut u = pbx_fs::xml::DirectoryUser::minimal("1001", "pw");
    u.number_alias = Some("a1001".to_string());
    let out = pbx_fs::xml::build_directory_xml_users("example.com", &[u]);
    assert!(
        out.contains("<param name=\"dial-string\" value=\"{sip_invite_domain=example.com,presence_id=a1001@example.com}${sofia_contact(*/a1001@example.com)}\"/>"),
        "default dial-string shape: {out}"
    );
    let mut u2 = pbx_fs::xml::DirectoryUser::minimal("1002", "pw");
    u2.call_timeout = Some(30);
    let out2 = pbx_fs::xml::build_directory_xml_users("example.com", &[u2]);
    assert!(
        out2.contains("<param name=\"dial-string\" value=\"{sip_invite_domain=example.com,leg_timeout=30,presence_id=1002@example.com}${sofia_contact(*/1002@example.com)}\"/>"),
        "leg_timeout dial-string shape: {out2}"
    );
}

#[test]
fn directory_enriched_per_ext_dial_string_defaults() {
    let out = pbx_fs::xml::build_directory_xml_users("example.com", &[enriched_user()]);
    assert!(
        out.contains("sofia_contact") && out.contains("1001@example.com"),
        "default dial-string must target ext@domain: {out}"
    );
    let mut u = enriched_user();
    u.do_not_disturb = false;
    u.dial_string = Some("loopback/9999".to_string());
    let out2 = pbx_fs::xml::build_directory_xml_users("example.com", &[u]);
    assert!(
        out2.contains("<param name=\"dial-string\" value=\"loopback/9999\"/>"),
        "per-ext dial-string wins: {out2}"
    );
}
