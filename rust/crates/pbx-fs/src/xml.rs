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
    w.write_event(Event::Empty(u)).expect("write user");

    w.write_event(Event::End(BytesEnd::new("domain")))
        .expect("close domain");
    w.write_event(Event::End(BytesEnd::new("section")))
        .expect("close section");
    end_document(&mut w);
    finish(w)
}

/// Pure-XML dialplan skeleton. Later tasks back this with SeaORM
/// queries; the signature stays.
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
    ext.push_attribute(("name", domain));
    w.write_event(Event::Empty(ext)).expect("write extension");

    w.write_event(Event::End(BytesEnd::new("context")))
        .expect("close context");
    w.write_event(Event::End(BytesEnd::new("section")))
        .expect("close section");
    end_document(&mut w);
    finish(w)
}
