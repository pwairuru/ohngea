use pbx_api::banner::{render_banner, resolve_port, BannerInfo};

#[test]
fn banner_contains_branding_and_port() {
    let info = BannerInfo {
        version: "0.1.0".to_string(),
        bind_addr: "0.0.0.0".to_string(),
        port: 8000,
        database_kind: "sqlite (dev)".to_string(),
        build_mode: "debug".to_string(),
    };
    let out = render_banner(&info);
    assert!(out.contains("Ohngea PBX"), "banner:\n{out}");
    assert!(out.contains("0.1.0"), "banner:\n{out}");
    assert!(out.contains("8000"), "banner:\n{out}");
    assert!(out.contains("/health"), "banner:\n{out}");
}

#[test]
fn resolve_port_defaults_to_8000() {
    // Ensure env does not leak into this assertion.
    std::env::remove_var("PORT");
    std::env::remove_var("API_PORT");
    assert_eq!(resolve_port(), 8000);
}
