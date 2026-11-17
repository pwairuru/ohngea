use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use std::io::Cursor;

fn decl() -> Event<'static> {
    Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None))
}

fn document(writer: &mut Writer<Cursor<Vec<u8>>>) {
    let mut root = BytesStart::new("document");
    root.push_attribute(("type", "freeswitch/xml"));
    writer
        .write_event(Event::Start(root))
        .expect("write document");
}

fn end_document(writer: &mut Writer<Cursor<Vec<u8>>>) {
    writer
        .write_event(Event::End(BytesEnd::new("document")))
        .expect("close document");
}

/// FreeSWITCH's switch_xml parser rejects documents whose XML declaration
/// is immediately followed by `<document` (no whitespace). lxml emits a
/// trailing newline (python parity); every builder must too. Call right
/// after writing `decl()`.
fn decl_nl<W: std::io::Write>(w: &mut Writer<W>) {
    w.write_event(Event::Text(BytesText::new("\n")))
        .expect("write decl newline");
}

fn finish(writer: Writer<Cursor<Vec<u8>>>) -> String {
    String::from_utf8(writer.into_inner().into_inner()).expect("xml is utf-8")
}

/// Pure-XML directory skeleton. Later tasks back this with SeaORM
/// queries; the `(domain, user) -> String` signature stays.
pub fn build_directory_xml(domain: &str, user: &str) -> String {
    build_directory_xml_with_password(domain, user, "")
}

/// Directory XML with stored password (mirrors
/// `app/freeswitch/xml_builder.py::build_directory_xml`: `<user id>`
/// with `<params><param name="password" .../>` under `<domain name>`).
pub fn build_directory_xml_with_password(domain: &str, user: &str, password: &str) -> String {
    build_directory_xml_multi(domain, &[(user, password)])
}

/// One extension row for directory rendering (PHP `extension.php::xml()` +
/// lua `directory.lua` parity: alias/cidr, voicemail join, MWI/auth-acl,
/// per-ext dial-string, full `<variables>` block, `extension_settings` rows).
#[derive(Clone, Debug, Default)]
pub struct ExtSetting {
    pub setting_type: String,
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct DirectoryUser {
    pub id: String,
    pub password: String,
    pub number_alias: Option<String>,
    pub cidr: Option<String>,
    pub dial_string: Option<String>,
    pub call_timeout: Option<i32>,
    pub user_context: Option<String>,
    pub domain_uuid: Option<String>,
    pub extension_uuid: Option<String>,
    pub accountcode: Option<String>,
    pub toll_allow: Option<String>,
    pub call_group: Option<String>,
    pub call_screen_enabled: bool,
    pub effective_caller_id_name: Option<String>,
    pub effective_caller_id_number: Option<String>,
    pub outbound_caller_id_name: Option<String>,
    pub outbound_caller_id_number: Option<String>,
    pub emergency_caller_id_name: Option<String>,
    pub emergency_caller_id_number: Option<String>,
    pub directory_first_name: Option<String>,
    pub directory_last_name: Option<String>,
    pub directory_visible: bool,
    pub directory_exten_visible: bool,
    pub limit_max: Option<String>,
    pub limit_destination: Option<String>,
    pub missed_call_app: Option<String>,
    pub missed_call_data: Option<String>,
    pub user_record: Option<String>,
    pub hold_music: Option<String>,
    pub auth_acl: Option<String>,
    pub sip_force_contact: Option<String>,
    pub sip_force_expires: Option<String>,
    pub nibble_account: Option<String>,
    pub mwi_account: Option<String>,
    pub sip_bypass_media: Option<String>,
    pub absolute_codec_string: Option<String>,
    pub force_ping: bool,
    pub max_registrations: Option<String>,
    pub forward_all_enabled: bool,
    pub forward_all_destination: Option<String>,
    pub forward_busy_enabled: bool,
    pub forward_busy_destination: Option<String>,
    pub forward_no_answer_enabled: bool,
    pub forward_no_answer_destination: Option<String>,
    pub forward_user_not_registered_enabled: bool,
    pub forward_user_not_registered_destination: Option<String>,
    pub follow_me_enabled: bool,
    pub do_not_disturb: bool,
    pub extension_language: Option<String>,
    pub extension_dialect: Option<String>,
    pub extension_voice: Option<String>,
    // Voicemail join (`v_voicemails` on alias-or-extension, lua parity).
    pub vm_password: Option<String>,
    pub vm_enabled: String,
    pub vm_mail_to: Option<String>,
    pub vm_attach_file: Option<String>,
    pub vm_keep_local_after_email: Option<String>,
    // `v_extension_settings` rows (enabled only, caller-filtered).
    pub settings: Vec<ExtSetting>,
}

impl DirectoryUser {
    /// Minimal user (legacy tuple path): password + reverse-auth + vm on.
    pub fn minimal(id: &str, password: &str) -> Self {
        DirectoryUser {
            id: id.to_owned(),
            password: password.to_owned(),
            vm_enabled: "true".to_owned(),
            ..Default::default()
        }
    }
}

impl Default for DirectoryUser {
    fn default() -> Self {
        DirectoryUser {
            id: String::new(),
            password: String::new(),
            number_alias: None,
            cidr: None,
            dial_string: None,
            call_timeout: None,
            user_context: None,
            domain_uuid: None,
            extension_uuid: None,
            accountcode: None,
            toll_allow: None,
            call_group: None,
            call_screen_enabled: false,
            effective_caller_id_name: None,
            effective_caller_id_number: None,
            outbound_caller_id_name: None,
            outbound_caller_id_number: None,
            emergency_caller_id_name: None,
            emergency_caller_id_number: None,
            directory_first_name: None,
            directory_last_name: None,
            directory_visible: false,
            directory_exten_visible: false,
            limit_max: None,
            limit_destination: None,
            missed_call_app: None,
            missed_call_data: None,
            user_record: None,
            hold_music: None,
            auth_acl: None,
            sip_force_contact: None,
            sip_force_expires: None,
            nibble_account: None,
            mwi_account: None,
            sip_bypass_media: None,
            absolute_codec_string: None,
            force_ping: false,
            max_registrations: None,
            forward_all_enabled: false,
            forward_all_destination: None,
            forward_busy_enabled: false,
            forward_busy_destination: None,
            forward_no_answer_enabled: false,
            forward_no_answer_destination: None,
            forward_user_not_registered_enabled: false,
            forward_user_not_registered_destination: None,
            follow_me_enabled: false,
            do_not_disturb: false,
            extension_language: None,
            extension_dialect: None,
            extension_voice: None,
            vm_password: None,
            vm_enabled: "true".to_owned(),
            vm_mail_to: None,
            vm_attach_file: None,
            vm_keep_local_after_email: None,
            settings: Vec::new(),
        }
    }
}

fn non_empty(v: &Option<String>) -> Option<&str> {
    v.as_deref().filter(|s| !s.is_empty())
}

fn write_param(w: &mut Writer<Cursor<Vec<u8>>>, name: &str, value: &str) {
    let mut p = BytesStart::new("param");
    p.push_attribute(("name", name));
    p.push_attribute(("value", value));
    w.write_event(Event::Empty(p)).expect("write param");
}

fn write_variable(w: &mut Writer<Cursor<Vec<u8>>>, name: &str, value: &str) {
    let mut v = BytesStart::new("variable");
    v.push_attribute(("name", name));
    v.push_attribute(("value", value));
    w.write_event(Event::Empty(v)).expect("write variable");
}

fn write_variable_opt(w: &mut Writer<Cursor<Vec<u8>>>, name: &str, value: &Option<String>) {
    if let Some(v) = non_empty(value) {
        write_variable(w, name, v);
    }
}

/// Full-domain directory: all users for mod_sofia REGISTER lookups, which
/// request the domain document (`key_value=<domain>`, no user part) and
/// resolve the user from it — not a per-user document.
/// Shape parity with `app/freeswitch/xml_builder.py::build_directory_xml`:
/// `domain > params(dial-string) + groups > group@default > users > user`.
/// mod_sofia navigates the groups nesting; a flat `domain > user` document
/// parses but yields "Can't find user" + REGISTER 403.
pub fn build_directory_xml_multi(domain: &str, users: &[(&str, &str)]) -> String {
    let full: Vec<DirectoryUser> = users
        .iter()
        .map(|(u, p)| DirectoryUser::minimal(u, p))
        .collect();
    build_directory_xml_users(domain, &full)
}

/// Enriched directory document (lua `directory.lua` + PHP `extension.php`
/// parity): per-user alias/cidr attrs, voicemail params, MWI/auth-acl,
/// per-ext dial-string, verto/max-registration params, extension_settings
/// params, and the full `<variables>` block.
pub fn build_directory_xml_users(domain: &str, users: &[DirectoryUser]) -> String {
    let mut w = Writer::new(Cursor::new(Vec::with_capacity(
        2048 + users.len() * 256,
    )));
    w.write_event(decl()).expect("write decl");
    decl_nl(&mut w);
    document(&mut w);

    let mut section = BytesStart::new("section");
    section.push_attribute(("name", "directory"));
    w.write_event(Event::Start(section)).expect("write section");

    let mut dom = BytesStart::new("domain");
    dom.push_attribute(("name", domain));
    w.write_event(Event::Start(dom)).expect("write domain");

    // dial-string default (python parity).
    let dparams = BytesStart::new("params");
    w.write_event(Event::Start(dparams)).expect("write params");
    let mut ds = BytesStart::new("param");
    ds.push_attribute(("name", "dial-string"));
    ds.push_attribute((
        "value",
        "{presence_id=${dialed_user}@${dialed_domain}}${sofia_contact(*/${dialed_user}@${dialed_domain})}",
    ));
    w.write_event(Event::Empty(ds)).expect("write dial-string");
    w.write_event(Event::End(BytesEnd::new("params")))
        .expect("close params");

    let groups = BytesStart::new("groups");
    w.write_event(Event::Start(groups)).expect("write groups");
    let mut group = BytesStart::new("group");
    group.push_attribute(("name", "default"));
    w.write_event(Event::Start(group)).expect("write group");
    let users_el = BytesStart::new("users");
    w.write_event(Event::Start(users_el)).expect("write users");

    for user in users {
        // quick_xml attributes borrow &str; alias/cidr live on `user`.
        let cidr_s = non_empty(&user.cidr);
        let alias_s = non_empty(&user.number_alias);
        let mut u = BytesStart::new("user");
        u.push_attribute(("id", user.id.as_str()));
        match (cidr_s, alias_s) {
            (Some(c), Some(a)) => {
                u.push_attribute(("cidr", c));
                u.push_attribute(("number-alias", a));
            }
            (Some(c), None) => {
                u.push_attribute(("cidr", c));
            }
            (None, Some(a)) => {
                u.push_attribute(("number-alias", a));
            }
            (None, None) => {}
        }
        w.write_event(Event::Start(u)).expect("write user");

        let params = BytesStart::new("params");
        w.write_event(Event::Start(params)).expect("write params");
        write_param(&mut w, "password", &user.password);
        write_param(&mut w, "reverse-auth-user", &user.id);
        write_param(&mut w, "reverse-auth-pass", &user.password);
        // Voicemail join (lua: vm_enabled defaults true; mail params only
        // when vm_mail_to is set).
        let vm_enabled = if user.vm_enabled.is_empty() {
            "true"
        } else {
            user.vm_enabled.as_str()
        };
        write_param(&mut w, "vm-enabled", vm_enabled);
        if let Some(mailto) = non_empty(&user.vm_mail_to) {
            if let Some(vp) = non_empty(&user.vm_password) {
                write_param(&mut w, "vm-password", vp);
            }
            write_param(&mut w, "vm-email-all-messages", vm_enabled);
            write_param(
                &mut w,
                "vm-attach-file",
                user.vm_attach_file.as_deref().unwrap_or("true"),
            );
            write_param(
                &mut w,
                "vm-keep-local-after-email",
                user.vm_keep_local_after_email.as_deref().unwrap_or("true"),
            );
            write_param(&mut w, "vm-mailto", mailto);
        }
        if let Some(mwi) = non_empty(&user.mwi_account) {
            write_param(&mut w, "MWI-Account", mwi);
        }
        if let Some(acl) = non_empty(&user.auth_acl) {
            write_param(&mut w, "auth-acl", acl);
        }
        // Per-ext dial-string (lua: DND routes to error/user_busy; stored
        // dial_string wins; else presence default with leg_timeout).
        let sip_id = alias_s.unwrap_or(user.id.as_str());
        let dial_string;
        let dial_value: &str = if user.do_not_disturb {
            "error/user_busy"
        } else if let Some(ds) = non_empty(&user.dial_string) {
            ds
        } else {
            dial_string = match user.call_timeout {
                Some(ct) => format!(
                    "{{sip_invite_domain={domain},leg_timeout={ct},presence_id={sip_id}@{domain}}}${{sofia_contact(*/{sip_id}@{domain})}}"
                ),
                None => format!(
                    "{{sip_invite_domain={domain},presence_id={sip_id}@{domain}}}${{sofia_contact(*/{sip_id}@{domain})}}"
                ),
            };
            dial_string.as_str()
        };
        write_param(&mut w, "dial-string", dial_value);
        write_param(
            &mut w,
            "verto-context",
            user.user_context.as_deref().filter(|s| !s.is_empty()).unwrap_or(domain),
        );
        write_param(&mut w, "verto-dialplan", "XML");
        write_param(&mut w, "jsonrpc-allowed-methods", "verto");
        write_param(
            &mut w,
            "jsonrpc-allowed-event-channels",
            "demo,conference,presence",
        );
        if let Some(mr) = non_empty(&user.max_registrations) {
            write_param(&mut w, "max-registrations-per-extension", mr);
        }
        for s in &user.settings {
            if s.setting_type == "param" && !s.name.is_empty() {
                write_param(&mut w, &s.name, &s.value);
            }
        }
        w.write_event(Event::End(BytesEnd::new("params")))
            .expect("close params");

        // `<variables>` block (lua ordering).
        let variables = BytesStart::new("variables");
        w.write_event(Event::Start(variables))
            .expect("write variables");
        if let Some(du) = non_empty(&user.domain_uuid) {
            write_variable(&mut w, "domain_uuid", du);
        }
        write_variable(&mut w, "domain_name", domain);
        if let Some(eu) = non_empty(&user.extension_uuid) {
            write_variable(&mut w, "extension_uuid", eu);
        }
        if let Some(ct) = user.call_timeout {
            write_variable(&mut w, "call_timeout", &ct.to_string());
        }
        write_variable(&mut w, "caller_id_name", &user.id);
        write_variable(&mut w, "caller_id_number", sip_id);
        write_variable(&mut w, "presence_id", &format!("{sip_id}@{domain}"));
        write_variable_opt(&mut w, "call_group", &user.call_group);
        if user.call_screen_enabled {
            // Stored as bool; lua emits only the "true" case.
            write_variable(&mut w, "call_screen_enabled", "true");
        }
        write_variable_opt(&mut w, "user_record", &user.user_record);
        write_variable_opt(&mut w, "hold_music", &user.hold_music);
        write_variable_opt(&mut w, "toll_allow", &user.toll_allow);
        write_variable_opt(&mut w, "accountcode", &user.accountcode);
        write_variable(
            &mut w,
            "user_context",
            user.user_context.as_deref().filter(|s| !s.is_empty()).unwrap_or(domain),
        );
        write_variable_opt(&mut w, "effective_caller_id_name", &user.effective_caller_id_name);
        write_variable_opt(
            &mut w,
            "effective_caller_id_number",
            &user.effective_caller_id_number,
        );
        write_variable_opt(&mut w, "outbound_caller_id_name", &user.outbound_caller_id_name);
        write_variable_opt(
            &mut w,
            "outbound_caller_id_number",
            &user.outbound_caller_id_number,
        );
        write_variable_opt(&mut w, "emergency_caller_id_name", &user.emergency_caller_id_name);
        write_variable_opt(
            &mut w,
            "emergency_caller_id_number",
            &user.emergency_caller_id_number,
        );
        write_variable_opt(&mut w, "missed_call_app", &user.missed_call_app);
        write_variable_opt(&mut w, "missed_call_data", &user.missed_call_data);
        let full_name = match (
            non_empty(&user.directory_first_name),
            non_empty(&user.directory_last_name),
        ) {
            (Some(f), Some(l)) => Some(format!("{f} {l}")),
            (Some(f), None) => Some(f.to_owned()),
            _ => None,
        };
        write_variable_opt(&mut w, "directory_full_name", &full_name);
        if user.directory_visible {
            write_variable(&mut w, "directory-visible", "true");
        }
        if user.directory_exten_visible {
            write_variable(&mut w, "directory-exten-visible", "true");
        }
        write_variable(
            &mut w,
            "limit_max",
            non_empty(&user.limit_max).unwrap_or("5"),
        );
        write_variable_opt(&mut w, "limit_destination", &user.limit_destination);
        write_variable_opt(&mut w, "sip-force-contact", &user.sip_force_contact);
        write_variable_opt(&mut w, "sip-force-expires", &user.sip_force_expires);
        write_variable_opt(&mut w, "nibble_account", &user.nibble_account);
        write_variable_opt(&mut w, "absolute_codec_string", &user.absolute_codec_string);
        if user.force_ping {
            write_variable(&mut w, "force_ping", "true");
        }
        match user.sip_bypass_media.as_deref() {
            Some("bypass-media") => write_variable(&mut w, "bypass_media", "true"),
            Some("bypass-media-after-bridge") => {
                write_variable(&mut w, "bypass_media_after_bridge", "true")
            }
            Some("proxy-media") => write_variable(&mut w, "proxy_media", "true"),
            _ => {}
        }
        if user.forward_all_enabled {
            write_variable(&mut w, "forward_all_enabled", "true");
        }
        write_variable_opt(&mut w, "forward_all_destination", &user.forward_all_destination);
        if user.forward_busy_enabled {
            write_variable(&mut w, "forward_busy_enabled", "true");
        }
        write_variable_opt(&mut w, "forward_busy_destination", &user.forward_busy_destination);
        if user.forward_no_answer_enabled {
            write_variable(&mut w, "forward_no_answer_enabled", "true");
        }
        write_variable_opt(
            &mut w,
            "forward_no_answer_destination",
            &user.forward_no_answer_destination,
        );
        if user.forward_user_not_registered_enabled {
            write_variable(&mut w, "forward_user_not_registered_enabled", "true");
        }
        write_variable_opt(
            &mut w,
            "forward_user_not_registered_destination",
            &user.forward_user_not_registered_destination,
        );
        if user.follow_me_enabled {
            write_variable(&mut w, "follow_me_enabled", "true");
        }
        if user.do_not_disturb {
            write_variable(&mut w, "do_not_disturb", "true");
        }
        if let Some(l) = non_empty(&user.extension_language) {
            write_variable(&mut w, "default_language", l);
        }
        if let Some(d) = non_empty(&user.extension_dialect) {
            write_variable(&mut w, "default_dialect", d);
        }
        if let Some(v) = non_empty(&user.extension_voice) {
            write_variable(&mut w, "default_voice", v);
        }
        write_variable(&mut w, "record_stereo", "true");
        write_variable(&mut w, "transfer_fallback_extension", "operator");
        write_variable(&mut w, "export_vars", "domain_name,domain_uuid");
        for s in &user.settings {
            if s.setting_type == "variable" && !s.name.is_empty() {
                write_variable(&mut w, &s.name, &s.value);
            }
        }
        w.write_event(Event::End(BytesEnd::new("variables")))
            .expect("close variables");

        w.write_event(Event::End(BytesEnd::new("user")))
            .expect("close user");
    }
    w.write_event(Event::End(BytesEnd::new("users")))
        .expect("close users");
    w.write_event(Event::End(BytesEnd::new("group")))
        .expect("close group");
    w.write_event(Event::End(BytesEnd::new("groups")))
        .expect("close groups");
    w.write_event(Event::End(BytesEnd::new("domain")))
        .expect("close domain");
    w.write_event(Event::End(BytesEnd::new("section")))
        .expect("close section");
    end_document(&mut w);
    finish(w)
}

/// Pure-XML dialplan skeleton. Later tasks back this with SeaORM
/// queries; the signature stays. Includes the `local_extension` bridge
/// (mirrors `build_dialplan_xml` local-extension handling).
pub fn build_dialplan_xml(domain: &str) -> String {
    let mut w = Writer::new(Cursor::new(Vec::with_capacity(2048)));
    w.write_event(decl()).expect("write decl");
    decl_nl(&mut w);
    document(&mut w);

    let mut section = BytesStart::new("section");
    section.push_attribute(("name", "dialplan"));
    w.write_event(Event::Start(section)).expect("write section");

    let mut context = BytesStart::new("context");
    context.push_attribute(("name", "default"));
    w.write_event(Event::Start(context)).expect("write context");

    let mut ext = BytesStart::new("extension");
    ext.push_attribute(("name", "local_extension"));
    w.write_event(Event::Start(ext)).expect("write extension");

    let mut cond = BytesStart::new("condition");
    cond.push_attribute(("field", "destination_number"));
    cond.push_attribute(("expression", "^(\\d+)$"));
    w.write_event(Event::Start(cond)).expect("write condition");

    let mut action = BytesStart::new("action");
    action.push_attribute(("application", "bridge"));
    action.push_attribute((
        "data",
        format!("sofia/internal/${{destination_number}}@{domain}").as_str(),
    ));
    w.write_event(Event::Empty(action)).expect("write bridge");

    w.write_event(Event::End(BytesEnd::new("condition")))
        .expect("close condition");
    w.write_event(Event::End(BytesEnd::new("extension")))
        .expect("close extension");

    w.write_event(Event::End(BytesEnd::new("context")))
        .expect("close context");
    w.write_event(Event::End(BytesEnd::new("section")))
        .expect("close section");
    end_document(&mut w);
    finish(w)
}

/// Configuration section from DB sip profiles with 60s-cache-friendly
/// pure render. Empty input renders the `internal` fallback profile
/// (mirrors `LAN_IP` fallback in `sofia.rs`); `domain_or_ip` notes the
/// IP-fallback path used when the domain lookup misses.
pub fn build_configuration_xml(name: &str, domain_or_ip: &str) -> String {
    if name.contains("sofia") {
        super::sofia::build_sofia_xml(&[])
    } else {
        // IP fallback note: render an empty configuration shell naming
        // the requested section so FS retries with the resolved IP.
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<configuration name=\"{name}\" description=\"fallback for {domain_or_ip}\"/>"
        )
    }
}
