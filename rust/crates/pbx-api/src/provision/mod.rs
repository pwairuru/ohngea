//! Provision templates (31 vendors) — minijinja port of templates/provision/*.
//! File per vendor for ordering. All templates embedded via include_str!.
//! Render mirrors app/devices/provision.py::render_provision (collapsed matrix,
//! model overrides, poly/polycom aliases).

pub mod aastra;
pub mod acrobits;
pub mod algo;
pub mod atcom;
pub mod avaya;
pub mod cisco;
pub mod digium;
pub mod escene;
pub mod fanvil;
pub mod flyingvoice;
pub mod grandstream;
pub mod groundwire;
pub mod htek;
pub mod linksys;
pub mod linphone;
pub mod mitel;
pub mod obihai;
pub mod panasonic;
pub mod poly;
pub mod polycom;
pub mod sangoma;
pub mod sipnetic;
pub mod snom;
pub mod spectralink;
pub mod swissvoice;
pub mod telekonnectors;
pub mod two_n;
pub mod vtech;
pub mod yealink;
pub mod yeastar;
pub mod zoiper;

/// Vendor candidates — mirrors `app/devices/provision.py::_vendor_candidates`.
fn vendor_candidates(vendor: &str) -> Vec<String> {
    let v = vendor.to_lowercase();
    let t = v.trim();
    if t == "poly" || t == "polycom" {
        return vec!["poly".to_owned(), "polycom".to_owned()];
    }
    vec![t.to_owned()]
}

fn base_template(vendor: &str) -> Option<&'static str> {
    match vendor {
        "2n" => Some(two_n::BASE),
        "aastra" => Some(aastra::BASE),
        "acrobits" => Some(acrobits::BASE),
        "algo" => Some(algo::BASE),
        "atcom" => Some(atcom::BASE),
        "avaya" => Some(avaya::BASE),
        "cisco" => Some(cisco::BASE),
        "digium" => Some(digium::BASE),
        "escene" => Some(escene::BASE),
        "fanvil" => Some(fanvil::BASE),
        "flyingvoice" => Some(flyingvoice::BASE),
        "grandstream" => Some(grandstream::BASE),
        "groundwire" => Some(groundwire::BASE),
        "htek" => Some(htek::BASE),
        "linksys" => Some(linksys::BASE),
        "linphone" => Some(linphone::BASE),
        "mitel" => Some(mitel::BASE),
        "obihai" => Some(obihai::BASE),
        "panasonic" => Some(panasonic::BASE),
        "poly" => Some(poly::BASE),
        "polycom" => Some(polycom::BASE),
        "sangoma" => Some(sangoma::BASE),
        "sipnetic" => Some(sipnetic::BASE),
        "snom" => Some(snom::BASE),
        "spectralink" => Some(spectralink::BASE),
        "swissvoice" => Some(swissvoice::BASE),
        "telekonnectors" => Some(telekonnectors::BASE),
        "vtech" => Some(vtech::BASE),
        "yealink" => Some(yealink::BASE),
        "yeastar" => Some(yeastar::BASE),
        "zoiper" => Some(zoiper::BASE),
        _ => None,
    }
}

/// Model override — mirrors collapsed-matrix `{vendor}/{model}.j2`
/// (case-insensitive, e.g. T46S/t46s, SPA122/spa122).
fn model_template(vendor: &str, model: &str) -> Option<&'static str> {
    let m = model.to_lowercase();
    match (vendor, m.as_str()) {
        ("cisco", "spa122") => Some(cisco::SPA122),
        ("grandstream", "gxp2170") => Some(grandstream::GXP2170),
        ("poly", "vvx250") | ("polycom", "vvx250") => Some(poly::VVX250),
        ("poly", "vvx450") | ("polycom", "vvx450") => Some(poly::VVX450),
        ("yealink", "t29g") => Some(yealink::T29G),
        ("yealink", "t46s") => Some(yealink::T46S),
        _ => None,
    }
}

fn render_str(
    tmpl: &str,
    vendor: &str,
    mac: &str,
    model: &str,
    extension: &str,
    domain: &str,
    password: &str,
) -> String {
    let env = minijinja::Environment::new();
    // `default("")` / `default("base")` filters used by base templates.
    let ctx = minijinja::context! {
        vendor => vendor,
        mac => mac,
        model => model,
        extension => extension,
        domain => domain,
        password => password,
        server_address => domain,
        user_id => extension,
    };
    env.render_str(tmpl, ctx).unwrap_or_else(|_| {
        format!("# Provision for {vendor}\nmac={mac}\nextension={extension}\ndomain={domain}\n")
    })
}

/// Minijinja render — mirrors `render_provision` in
/// `app/devices/provision.py` (collapsed matrix + aliases).
/// Returns `None` when vendor is unknown (caller falls back to legacy/generic).
pub fn render(
    vendor_raw: &str,
    mac: &str,
    model: &str,
    extension: &str,
    domain: &str,
    password: &str,
) -> Option<String> {
    let vendors = vendor_candidates(vendor_raw);
    let model = model.trim();
    // 1. model overrides (each candidate vendor, case-insensitive).
    if !model.is_empty() {
        for v in &vendors {
            if let Some(t) = model_template(v.as_str(), model) {
                return Some(render_str(
                    t,
                    v.as_str(),
                    mac,
                    model,
                    extension,
                    domain,
                    password,
                ));
            }
        }
    }
    // 2. base templates (candidate order → poly wins for polycom, parity).
    for v in &vendors {
        if let Some(t) = base_template(v.as_str()) {
            return Some(render_str(
                t,
                v.as_str(),
                mac,
                model,
                extension,
                domain,
                password,
            ));
        }
    }
    None
}
