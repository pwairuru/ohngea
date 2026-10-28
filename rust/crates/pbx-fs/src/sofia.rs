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

fn start_profile(writer: &mut Writer<Cursor<Vec<u8>>>, name: &str) {
    let mut prof = BytesStart::new("profile");
    prof.push_attribute(("name", name));
    writer
        .write_event(Event::Start(prof))
        .expect("write profile");
    let settings = BytesStart::new("settings");
    writer
        .write_event(Event::Start(settings))
        .expect("write settings");
}

fn end_profile(writer: &mut Writer<Cursor<Vec<u8>>>) {
    writer
        .write_event(Event::End(BytesEnd::new("settings")))
        .expect("close settings");
    writer
        .write_event(Event::End(BytesEnd::new("profile")))
        .expect("close profile");
}

/// Build `sofia.conf.xml`. Verbatim logic of `_build_sofia_xml`.
#[must_use]
pub fn build_sofia_xml(profiles: &[SofiaProfileInput]) -> String {
    let mut w = Writer::new(Cursor::new(Vec::new()));
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .expect("write decl");

    let mut root = BytesStart::new("configuration");
    root.push_attribute(("name", "sofia.conf"));
    root.push_attribute(("description", "sofia Endpoint"));
    w.write_event(Event::Start(root)).expect("write root");
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
            let sip_ip = p.sip_ip.clone().unwrap_or_else(|| "0.0.0.0".to_owned());
            let lan = lan_ip();
            let ext_sip_ip = p.ext_sip_ip.clone().unwrap_or_else(|| lan.clone());
            let ext_rtp_ip = p.ext_rtp_ip.clone().unwrap_or_else(|| ext_sip_ip.clone());
            let rtp_ip = p.rtp_ip.clone().unwrap_or_else(|| "0.0.0.0".to_owned());
            let mut realm = p.realm.clone().unwrap_or_else(|| p.name.clone());
            if realm.is_empty() {
                realm = "test.example.com".to_owned();
            }
            let mut context = p.context.clone().unwrap_or_else(|| realm.clone());
            if context.is_empty() {
                context = realm.clone();
            }
            start_profile(&mut w, name);
            param(&mut w, "sip-ip", &sip_ip);
            param(&mut w, "ext-sip-ip", &ext_sip_ip);
            param(&mut w, "ext-rtp-ip", &ext_rtp_ip);
            param(&mut w, "rtp-ip", &rtp_ip);
            param(&mut w, "sip-port", &sip_port);
            param(&mut w, "dialplan", "XML");
            param(&mut w, "context", &context);
            param(&mut w, "realm", &realm);
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
            end_profile(&mut w);
        }
    }

    w.write_event(Event::End(BytesEnd::new("profiles")))
        .expect("close profiles");
    w.write_event(Event::End(BytesEnd::new("configuration")))
        .expect("close root");
    String::from_utf8(w.into_inner().into_inner()).expect("xml is utf-8")
}
