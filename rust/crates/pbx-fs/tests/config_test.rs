//! Task 5 (PHP `fusionpbx/resources/switch.php` + Lua `xml_handler`
//! parity) configuration-builder tests.
//!
//! Ground truth:
//! - gateway: `switch.php::save_gateway_xml` (`v_<uuid>.xml` file semantics)
//! - vars: `switch.php::save_var_xml` (hostname-scoped, Provision excluded,
//!   `Exec-Set` -> `exec-set`)
//! - callcenter: `switch.php::save_call_center_xml` (`queue@domain`,
//!   `agent@domain`, contact `call_timeout`/`group_confirm`, `tone_stream`)
//! - ivr: `configuration/ivr.conf.lua` (menu per uuid + entries)
//! - conference: `configuration/conference.conf.lua`
//! - local_stream: `configuration/local_stream.conf.lua`
//! - sofia: `configuration/sofia.conf.lua` (gateways embedded per profile)

use pbx_fs::config::*;

// ---- gateway (switch.php:161-262) ----

fn full_gateway() -> GatewayInput {
    GatewayInput {
        gateway_uuid: "ABCDEFAB-1111-2222-3333-444444444444".to_owned(),
        username: Some("gwuser".to_owned()),
        password: Some("gwpw".to_owned()),
        proxy: Some("sip.provider.example".to_owned()),
        register: true,
        profile: "external".to_owned(),
        context: Some("public".to_owned()),
        enabled: true,
        distinct_to: Some(false),
        auth_username: Some("authu".to_owned()),
        realm: Some("provider.example".to_owned()),
        from_user: Some("fromu".to_owned()),
        from_domain: Some("fromd.example".to_owned()),
        register_proxy: Some("reg.proxy.example".to_owned()),
        outbound_proxy: Some("out.proxy.example".to_owned()),
        expire_seconds: Some(800),
        register_transport: Some("tls".to_owned()),
        contact_params: Some("tport=tls".to_owned()),
        retry_seconds: Some(30),
        extension: Some("1000".to_owned()),
        ping: Some("60".to_owned()),
        caller_id_in_from: Some(true),
        supress_cng: Some(true),
        sip_cid_type: Some("pid".to_owned()),
        extension_in_contact: Some("true".to_owned()),
        ..Default::default()
    }
}

#[test]
fn gateway_wraps_include_with_lowercase_uuid_name() {
    let xml = build_gateway_xml(&full_gateway());
    assert!(
        xml.contains("<gateway name=\"abcdefab-1111-2222-3333-444444444444\">"),
        "lowercase uuid name: {xml}"
    );
    assert!(xml.starts_with("<include>") && xml.trim_end().ends_with("</include>"));
}

#[test]
fn gateway_renders_all_params_in_php_order() {
    let xml = build_gateway_xml(&full_gateway());
    let order = [
        "username",
        "distinct-to",
        "auth-username",
        "password",
        "realm",
        "from-user",
        "from-domain",
        "proxy",
        "register-proxy",
        "outbound-proxy",
        "expire-seconds",
        "register",
        "register-transport",
        "contact-params",
        "retry-seconds",
        "extension",
        "ping",
        "context",
        "caller-id-in-from",
        "supress-cng",
        "sip_cid_type",
        "extension-in-contact",
    ];
    let mut last = 0;
    for name in order {
        let needle = format!("<param name=\"{name}\"");
        let pos = xml
            .find(&needle)
            .unwrap_or_else(|| panic!("missing {name}: {xml}"));
        assert!(pos >= last, "param {name} out of PHP order: {xml}");
        last = pos;
    }
    // register-transport switch value passes through
    assert!(
        xml.contains("<param name=\"register-transport\" value=\"tls\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"supress-cng\" value=\"true\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"caller-id-in-from\" value=\"true\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"extension-in-contact\" value=\"true\"/>"),
        "{xml}"
    );
    // distinct-to false renders (PHP !empty("false") is true)
    assert!(
        xml.contains("<param name=\"distinct-to\" value=\"false\"/>"),
        "{xml}"
    );
}

#[test]
fn gateway_register_transport_switch() {
    for (inp, want) in [
        ("udp", "udp"),
        ("tcp", "tcp"),
        ("tls", "tls"),
        ("sctp", "sctp"),
    ] {
        let mut g = GatewayInput::default();
        g.gateway_uuid = "11111111-1111-1111-1111-111111111111".to_owned();
        g.register_transport = Some(inp.to_owned());
        let xml = build_gateway_xml(&g);
        assert!(
            xml.contains(&format!(
                "<param name=\"register-transport\" value=\"{want}\"/>"
            )),
            "transport {inp}: {xml}"
        );
    }
}

#[test]
fn gateway_omits_empty_params() {
    let mut g = GatewayInput::default();
    g.gateway_uuid = "22222222-2222-2222-2222-222222222222".to_owned();
    g.register = false;
    let xml = build_gateway_xml(&g);
    for absent in [
        "username",
        "password",
        "proxy",
        "realm",
        "register-transport",
        "caller-id-in-from",
        "supress-cng",
        "extension-in-contact",
    ] {
        assert!(
            !xml.contains(&format!("<param name=\"{absent}\"")),
            "must omit {absent}: {xml}"
        );
    }
    assert!(
        xml.contains("<param name=\"register\" value=\"false\"/>"),
        "{xml}"
    );
}

// ---- vars (switch.php:277-338) ----

fn var(cat: &str, name: &str, value: &str, host: Option<&str>, cmd: Option<&str>) -> VarInput {
    VarInput {
        category: cat.to_owned(),
        name: name.to_owned(),
        value: value.to_owned(),
        command: cmd.map(str::to_owned),
        hostname: host.map(str::to_owned),
        enabled: true,
        order: None,
    }
}

#[test]
fn vars_hostname_scoped_provision_excluded_exec_set_mapped() {
    let rows = vec![
        var("General", "domain", "example.com", None, None),
        var("General", "hostvar", "h1", Some("myhost"), None),
        var("General", "othervar", "no", Some("otherhost"), None),
        var("Provision", "provvar", "no", None, None),
        var("Exec-Set", "dynvar", "cmd", None, None),
    ];
    let xml = build_vars_xml(&rows, "myhost");
    assert!(
        xml.contains("<X-PRE-PROCESS cmd=\"set\" data=\"domain=example.com\" />"),
        "{xml}"
    );
    assert!(
        xml.contains("<X-PRE-PROCESS cmd=\"set\" data=\"hostvar=h1\" />"),
        "{xml}"
    );
    assert!(
        !xml.contains("othervar"),
        "foreign hostname excluded: {xml}"
    );
    assert!(!xml.contains("provvar"), "Provision excluded: {xml}");
    assert!(
        xml.contains("<X-PRE-PROCESS cmd=\"exec-set\" data=\"dynvar=cmd\" />"),
        "Exec-Set maps to exec-set: {xml}"
    );
    assert!(
        xml.contains("<!-- General -->") && xml.contains("<!-- Exec-Set -->"),
        "{xml}"
    );
}

#[test]
fn vars_disabled_rows_excluded() {
    let mut v = var("General", "off", "0", None, None);
    v.enabled = false;
    let xml = build_vars_xml(&[v], "myhost");
    assert!(!xml.contains("off="), "{xml}");
}

// ---- callcenter (switch.php:647-868) ----

fn queue(name: &str) -> CallcenterQueueInput {
    CallcenterQueueInput {
        name: name.to_owned(),
        extension: Some("7000".to_owned()),
        strategy: "longest-idle-agent".to_owned(),
        moh_sound: None,
        record_template: None,
        ..Default::default()
    }
}

fn agent(name: &str, contact: &str) -> CallcenterAgentInput {
    CallcenterAgentInput {
        uuid: format!("agent-uuid-{name}"),
        name: name.to_owned(),
        agent_type: Some("callback".to_owned()),
        contact: contact.to_owned(),
        status: "Available".to_owned(),
        call_timeout: 60,
        ..Default::default()
    }
}

#[test]
fn callcenter_queue_at_domain_with_tone_stream() {
    let q = CallcenterQueueInput {
        name: "sales queue".to_owned(),
        moh_sound: Some("${us-ring}".to_owned()),
        ..queue("ignored")
    };
    let xml = build_callcenter_xml(&[(q, "example.com".to_owned())], &[], &[]);
    assert!(
        xml.contains("<queue name=\"sales_queue@example.com\">"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"strategy\" value=\"longest-idle-agent\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"moh-sound\" value=\"tone_stream://${us-ring};loops=-1\"/>"),
        "tone_stream: {xml}"
    );
    assert!(xml.contains("name=\"callcenter.conf\""), "{xml}");
}

#[test]
fn callcenter_moh_defaults_and_passthrough() {
    let xml = build_callcenter_xml(&[(queue("q1"), "d.example".to_owned())], &[], &[]);
    assert!(
        xml.contains("<param name=\"moh-sound\" value=\"local_stream://default\"/>"),
        "{xml}"
    );
    let mut q = queue("q2");
    q.moh_sound = Some("local_stream://moh".to_owned());
    let xml = build_callcenter_xml(&[(q, "d.example".to_owned())], &[], &[]);
    assert!(
        xml.contains("<param name=\"moh-sound\" value=\"local_stream://moh\"/>"),
        "{xml}"
    );
}

#[test]
fn callcenter_agent_contact_call_timeout_and_confirm() {
    let plain = agent("bob", "user/1001@example.com");
    let gw = agent("alice", "sofia/gateway/mygw/15551234567");
    let xml = build_callcenter_xml(
        &[(queue("q1"), "example.com".to_owned())],
        &[
            (plain, "example.com".to_owned()),
            (gw, "example.com".to_owned()),
        ],
        &[],
    );
    assert!(xml.contains("<agent name=\"bob@example.com\""), "{xml}");
    assert!(
        xml.contains("contact=\"{call_timeout=60}user/1001@example.com\""),
        "plain contact timeout: {xml}"
    );
    assert!(
        xml.contains("group_confirm_file=custom/press_1_to_accept_this_call.wav"),
        "gateway contact confirm: {xml}"
    );
    assert!(xml.contains("group_confirm_key=1"), "{xml}");
    assert!(xml.contains("leg_timeout=60"), "{xml}");
}

#[test]
fn callcenter_tier_agent_and_queue_at_domain() {
    let t = CallcenterTierInput {
        agent_name: "bob".to_owned(),
        queue_name: "sales_queue".to_owned(),
        ..Default::default()
    };
    let xml = build_callcenter_xml(
        &[(queue("sales queue"), "example.com".to_owned())],
        &[],
        &[(t, "example.com".to_owned())],
    );
    assert!(
        xml.contains("<tier agent=\"bob@example.com\" queue=\"sales_queue@example.com\""),
        "{xml}"
    );
}

// ---- ivr (ivr.conf.lua) ----

#[test]
fn ivr_menu_attrs_entries_and_direct_dial() {
    let menu = IvrMenuInput {
        uuid: "ivr-uuid-1".to_owned(),
        name: "main".to_owned(),
        greet_long: Some("welcome.wav".to_owned()),
        timeout: Some(10),
        direct_dial: true,
        context: Some("example.com".to_owned()),
        domain: "example.com".to_owned(),
        ..Default::default()
    };
    let opts = vec![IvrOptionInput {
        action: "menu-exec-app".to_owned(),
        digits: "1".to_owned(),
        param: Some("transfer 1001 XML example.com".to_owned()),
        description: None,
    }];
    let xml = build_ivr_xml(&[(&menu, opts.as_slice())]);
    assert!(
        xml.contains("<menu name=\"ivr-uuid-1\" description=\"main\""),
        "{xml}"
    );
    assert!(xml.contains("greet-long=\"welcome.wav\""), "{xml}");
    assert!(
        xml.contains("<entry action=\"menu-exec-app\" digits=\"1\""),
        "{xml}"
    );
    assert!(xml.contains("transfer 1001 XML example.com"), "{xml}");
    assert!(
        xml.contains("transfer $1 XML example.com"),
        "direct dial: {xml}"
    );
    assert!(xml.contains("name=\"ivr.conf\""), "{xml}");
}

// ---- conference (conference.conf.lua) ----

#[test]
fn conference_profiles_and_controls() {
    let profiles = vec![ConferenceProfileInput {
        name: "default".to_owned(),
        params: vec![("max-members".to_owned(), "50".to_owned())],
    }];
    let controls = vec![ConferenceControlInput {
        name: "default".to_owned(),
        details: vec![("1".to_owned(), "mute".to_owned(), "toggle".to_owned())],
    }];
    let xml = build_conference_xml(&profiles, &controls);
    assert!(xml.contains("<profile name=\"default\">"), "{xml}");
    assert!(
        xml.contains("<param name=\"max-members\" value=\"50\"/>"),
        "{xml}"
    );
    assert!(xml.contains("<group name=\"default\">"), "{xml}");
    assert!(
        xml.contains("<control digits=\"1\" action=\"mute\" data=\"toggle\"/>"),
        "{xml}"
    );
    assert!(xml.contains("name=\"conference.conf\""), "{xml}");
}

// ---- local_stream (local_stream.conf.lua) ----

#[test]
fn local_stream_directory_shape() {
    let s = LocalStreamInput {
        name: "example.com/moh".to_owned(),
        uuid: "ls-uuid-1".to_owned(),
        path: "/srv/moh".to_owned(),
        rate: None,
        shuffle: Some("true".to_owned()),
        timer_name: None,
        chime_list: None,
        chime_freq: None,
        chime_max: None,
    };
    let xml = build_local_stream_xml(&[s]);
    assert!(
        xml.contains("<directory name=\"example.com/moh\" uuid=\"ls-uuid-1\" path=\"/srv/moh\">"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"rate\" value=\"48000\"/>"),
        "default rate: {xml}"
    );
    assert!(
        xml.contains("<param name=\"channels\" value=\"1\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"interval\" value=\"20\"/>"),
        "{xml}"
    );
    assert!(xml.contains("name=\"local_stream.conf\""), "{xml}");
}

// ---- sofia full (sofia.conf.lua: gateways + settings + domains + globals) ----

#[test]
fn sofia_full_embeds_gateways_settings_domains_globals() {
    let mut g = GatewayInput::default();
    g.gateway_uuid = "33333333-3333-3333-3333-333333333333".to_owned();
    g.username = Some("u".to_owned());
    g.password = Some("p".to_owned());
    g.register = true;
    let p = pbx_fs::sofia::SofiaProfileInput {
        name: "external".to_owned(),
        settings: vec![("auth-calls".to_owned(), "true".to_owned())],
        domains: vec![(
            "example.com".to_owned(),
            "true".to_owned(),
            "true".to_owned(),
        )],
        gateways: vec![g],
        ..Default::default()
    };
    let xml =
        pbx_fs::sofia::build_sofia_xml_with_globals(&[p], &[("debug".to_owned(), "0".to_owned())]);
    assert!(xml.contains("<global_settings>"), "{xml}");
    assert!(xml.contains("<param name=\"debug\" value=\"0\"/>"), "{xml}");
    assert!(xml.contains("<profile name=\"external\">"), "{xml}");
    assert!(
        xml.contains("<gateway name=\"33333333-3333-3333-3333-333333333333\">"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"username\" value=\"u\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<domain name=\"example.com\" alias=\"true\" parse=\"true\"/>"),
        "{xml}"
    );
    assert!(
        xml.contains("<param name=\"auth-calls\" value=\"true\"/>"),
        "{xml}"
    );
}
