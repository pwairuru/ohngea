use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, Event},
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
    let mut w = Writer::new(Cursor::new(Vec::new()));
    w.write_event(decl()).expect("write decl");
    document(&mut w);

    let mut section = BytesStart::new("section");
    section.push_attribute(("name", "directory"));
    w.write_event(Event::Start(section)).expect("write section");

    let mut dom = BytesStart::new("domain");
    dom.push_attribute(("name", domain));
    w.write_event(Event::Start(dom)).expect("write domain");

    let mut u = BytesStart::new("user");
    u.push_attribute(("id", user));
    w.write_event(Event::Start(u)).expect("write user");

    let params = BytesStart::new("params");
    w.write_event(Event::Start(params)).expect("write params");
    let mut p = BytesStart::new("param");
    p.push_attribute(("name", "password"));
    p.push_attribute(("value", password));
    w.write_event(Event::Empty(p)).expect("write password");
    w.write_event(Event::End(BytesEnd::new("params")))
        .expect("close params");

    w.write_event(Event::End(BytesEnd::new("user")))
        .expect("close user");
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
    let mut w = Writer::new(Cursor::new(Vec::new()));
    w.write_event(decl()).expect("write decl");
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
            r#"<?xml version="1.0" encoding="UTF-8"?><configuration name="{name}" description="fallback for {domain_or_ip}"/>"#
        )
    }
}
