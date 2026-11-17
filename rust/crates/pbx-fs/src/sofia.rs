//! Generate `sofia.conf.xml` from DB sip profiles + fallback.
//!
//! Verbatim port of `app/freeswitch/sofia_config.py::_build_sofia_xml`
//! (145 lines): empty input renders the `internal` + `internal-test`
//! fallback for `LAN_IP` (default `192.168.100.82`); otherwise one
//! `<profile>` per row with the same param mapping and the same
//! `hold-music` / `record-path` conditionals.

use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, Event},
    Writer,
};
use std::io::Cursor;

/// One sip_profiles row's fields used by the XML builder.
/// Mirrors the `getattr(p, ...)` lookups in `_build_sofia_xml`.
#[derive(Clone, Debug, Default)]
pub struct SofiaProfileInput {
    pub name: String,
    pub sip_port: i32,
    pub sip_ip: Option<String>,
    pub ext_sip_ip: Option<String>,
    pub ext_rtp_ip: Option<String>,
    pub rtp_ip: Option<String>,
    pub realm: Option<String>,
    pub context: Option<String>,
    pub challenge_realm: Option<String>,
    pub apply_inbound_acl: Option<String>,
    pub local_network_acl: Option<String>,
    pub hold_music: Option<String>,
    pub record_path: Option<String>,
    /// Enabled `v_sip_profile_settings` rows (Lua: settings params).
    /// Rows duplicating a fixed param above are skipped.
    pub settings: Vec<(String, String)>,
    /// `v_sip_profile_domains` rows `(name, alias, parse)`.
    pub domains: Vec<(String, String, String)>,
    /// Enabled gateways of this profile (Lua: embedded `<gateway>`).
    pub gateways: Vec<super::config::GatewayInput>,
}

fn lan_ip() -> String {
    std::env::var("LAN_IP").unwrap_or_else(|_| "192.168.100.82".to_owned())
}

fn param(writer: &mut Writer<Cursor<Vec<u8>>>, name: &str, value: &str) {
    let mut e = BytesStart::new("param");
    e.push_attribute(("name", name));
    e.push_attribute(("value", value));
    writer.write_event(Event::Empty(e)).expect("write param");
}

fn start_settings(writer: &mut Writer<Cursor<Vec<u8>>>) {
    let settings = BytesStart::new("settings");
    writer
        .write_event(Event::Start(settings))
        .expect("write settings");
}

fn start_profile(writer: &mut Writer<Cursor<Vec<u8>>>, name: &str) {
    let mut prof = BytesStart::new("profile");
    prof.push_attribute(("name", name));
    writer
        .write_event(Event::Start(prof))
        .expect("write profile");
    let aliases = BytesStart::new("aliases");
    writer
        .write_event(Event::Start(aliases))
        .expect("write aliases");
    writer
        .write_event(Event::End(BytesEnd::new("aliases")))
        .expect("close aliases");
    start_settings(writer);
}

fn end_profile(writer: &mut Writer<Cursor<Vec<u8>>>) {
    writer
        .write_event(Event::End(BytesEnd::new("settings")))
        .expect("close settings");
    writer
        .write_event(Event::End(BytesEnd::new("profile")))
        .expect("close profile");
}

/// Embed one profile's gateways + domains between `<aliases/>` and
/// `<settings>`: the Lua `sofia.conf.lua` order is
/// `aliases → gateways → domains → settings`, but `start_profile` opens
/// `<settings>` eagerly, so this closes it, writes the blocks, and reopens
/// it. Fixed params rendered afterwards still land inside `<settings>`.
fn interleave_gateways_domains(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    gateways: &[super::config::GatewayInput],
    domains: &[(String, String, String)],
) {
    use std::io::Write;
    writer
        .write_event(Event::End(BytesEnd::new("settings")))
        .expect("pause settings");
    let gateways_el = BytesStart::new("gateways");
    writer
        .write_event(Event::Start(gateways_el))
        .expect("write gateways");
    for g in gateways {
        // Raw (not `Event::Text`, which would escape the tags): the element
        // is already well-formed XML from `config::gateway_element`.
        // Indent matches the surrounding Lua-rendered layout.
        let el = super::config::gateway_element(g);
        for line in el.lines() {
            writer
                .get_mut()
                .write_all(format!("                    {line}\n").as_bytes())
                .expect("write gateway");
        }
    }
    writer
        .write_event(Event::End(BytesEnd::new("gateways")))
        .expect("close gateways");
    let domains_el = BytesStart::new("domains");
    writer
        .write_event(Event::Start(domains_el))
        .expect("write domains");
    for (name, alias, parse) in domains {
        let mut d = BytesStart::new("domain");
        d.push_attribute(("name", name.as_str()));
        d.push_attribute(("alias", alias.as_str()));
        d.push_attribute(("parse", parse.as_str()));
        writer.write_event(Event::Empty(d)).expect("write domain");
    }
    writer
        .write_event(Event::End(BytesEnd::new("domains")))
        .expect("close domains");
    start_settings(writer);
}

/// Build `sofia.conf.xml`. Verbatim logic of `_build_sofia_xml`.
#[must_use]
pub fn build_sofia_xml(profiles: &[SofiaProfileInput]) -> String {
    build_sofia_xml_with_globals(profiles, &[])
}

/// Full `sofia.conf.xml` (Lua `sofia.conf.lua` parity): enabled
/// `v_sofia_global_settings` first, then one `<profile>` per enabled,
/// hostname-scoped `v_sip_profiles` row with its gateways, domains and
/// extra settings. Empty input renders the `internal` + `internal-test`
/// fallback (mirrors `LAN_IP` fallback); `domain_or_ip` callers keep the
/// old `build_sofia_xml` path.
#[must_use]
pub fn build_sofia_xml_with_globals(
    profiles: &[SofiaProfileInput],
    globals: &[(String, String)],
) -> String {
    let mut w = Writer::new(Cursor::new(Vec::with_capacity(4096)));
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .expect("write decl");
    // switch_xml rejects `?>` immediately followed by `<` (see xml.rs decl_nl).
    w.write_event(Event::Text(quick_xml::events::BytesText::new("\n")))
        .expect("write decl newline");

    let mut root = BytesStart::new("configuration");
    root.push_attribute(("name", "sofia.conf"));
    root.push_attribute(("description", "sofia Endpoint"));
    w.write_event(Event::Start(root)).expect("write root");
    let globals_el = BytesStart::new("global_settings");
    w.write_event(Event::Start(globals_el))
        .expect("write global_settings");
    for (name, value) in globals {
        param(&mut w, name, value);
    }
    w.write_event(Event::End(BytesEnd::new("global_settings")))
        .expect("close global_settings");
    let profiles_el = BytesStart::new("profiles");
    w.write_event(Event::Start(profiles_el))
        .expect("write profiles");

    if profiles.is_empty() {
        // fallback default as before
        let lan = lan_ip();
        let fallback = [
            ("internal", "5060", "test.example.com", "test.example.com"),
            (
                "internal-test",
                "5061",
                "test.example.com",
                "test.example.com",
            ),
        ];
        for (name, port, ctx, realm) in fallback {
            start_profile(&mut w, name);
            param(&mut w, "sip-ip", "0.0.0.0");
            param(&mut w, "ext-sip-ip", &lan);
            param(&mut w, "ext-rtp-ip", &lan);
            param(&mut w, "rtp-ip", "0.0.0.0");
            param(&mut w, "sip-port", port);
            param(&mut w, "dialplan", "XML");
            param(&mut w, "context", ctx);
            param(&mut w, "realm", realm);
            param(&mut w, "challenge-realm", "auto_from");
            param(&mut w, "accept-blind-reg", "false");
            param(&mut w, "inbound-late-negotiation", "true");
            param(&mut w, "rtp-timer-name", "soft");
            param(&mut w, "enable-100rel", "false");
            param(&mut w, "disable-register", "false");
            param(&mut w, "apply-inbound-acl", "e2e-all");
            param(&mut w, "local-network-acl", "localnet.auto");
            end_profile(&mut w);
        }
    } else {
        // Single `LAN_IP` env lookup for the whole render (was: one syscall
        // per profile inside the loop).
        let lan = lan_ip();
        for p in profiles {
            let name = if p.name.is_empty() {
                "internal"
            } else {
                &p.name
            };
            let sip_port = if p.sip_port == 0 {
                "5060".to_owned()
            } else {
                p.sip_port.to_string()
            };
            // Borrow profile fields: the old code cloned ~6 `Option<String>`
            // per profile just to borrow them in `param()` below.
            let sip_ip = p.sip_ip.as_deref().unwrap_or("0.0.0.0");
            let ext_sip_ip = p.ext_sip_ip.as_deref().unwrap_or(lan.as_str());
            let ext_rtp_ip = p.ext_rtp_ip.as_deref().unwrap_or(ext_sip_ip);
            let rtp_ip = p.rtp_ip.as_deref().unwrap_or("0.0.0.0");
            let realm = match p.realm.as_deref().filter(|s| !s.is_empty()) {
                Some(r) => r,
                None if !p.name.is_empty() => p.name.as_str(),
                None => "test.example.com",
            };
            let context = match p.context.as_deref().filter(|s| !s.is_empty()) {
                Some(c) => c,
                None => realm,
            };
            start_profile(&mut w, name);
            if !p.gateways.is_empty() || !p.domains.is_empty() {
                interleave_gateways_domains(&mut w, &p.gateways, &p.domains);
            }
            param(&mut w, "sip-ip", sip_ip);
            param(&mut w, "ext-sip-ip", ext_sip_ip);
            param(&mut w, "ext-rtp-ip", ext_rtp_ip);
            param(&mut w, "rtp-ip", rtp_ip);
            param(&mut w, "sip-port", &sip_port);
            param(&mut w, "dialplan", "XML");
            param(&mut w, "context", context);
            param(&mut w, "realm", realm);
            param(
                &mut w,
                "challenge-realm",
                p.challenge_realm.as_deref().unwrap_or("auto_from"),
            );
            param(&mut w, "accept-blind-reg", "false");
            param(&mut w, "inbound-late-negotiation", "true");
            param(&mut w, "rtp-timer-name", "soft");
            param(&mut w, "enable-100rel", "false");
            param(&mut w, "disable-register", "false");
            param(
                &mut w,
                "apply-inbound-acl",
                p.apply_inbound_acl.as_deref().unwrap_or("e2e-all"),
            );
            param(
                &mut w,
                "local-network-acl",
                p.local_network_acl.as_deref().unwrap_or("localnet.auto"),
            );
            if let Some(hm) = &p.hold_music {
                param(&mut w, "hold-music", hm);
            }
            if let Some(rp) = &p.record_path {
                param(&mut w, "record-path", rp);
            }
            // Extra `v_sip_profile_settings` rows (Lua parity); skip names
            // already rendered from the fixed struct fields above.
            const FIXED: &[&str] = &[
                "sip-ip",
                "ext-sip-ip",
                "ext-rtp-ip",
                "rtp-ip",
                "sip-port",
                "dialplan",
                "context",
                "realm",
                "challenge-realm",
                "accept-blind-reg",
                "inbound-late-negotiation",
                "rtp-timer-name",
                "enable-100rel",
                "disable-register",
                "apply-inbound-acl",
                "local-network-acl",
                "hold-music",
                "record-path",
            ];
            for (name, value) in &p.settings {
                if FIXED.iter().any(|f| f.eq_ignore_ascii_case(name)) {
                    continue;
                }
                param(&mut w, name, value);
            }
            end_profile(&mut w);
        }
    }

    w.write_event(Event::End(BytesEnd::new("profiles")))
        .expect("close profiles");
    w.write_event(Event::End(BytesEnd::new("configuration")))
        .expect("close root");
    String::from_utf8(w.into_inner().into_inner()).expect("xml is utf-8")
}
