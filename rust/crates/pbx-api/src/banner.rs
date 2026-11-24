/// Boot banner + branding for `pbx-api` (no logging deps, plain stdout).
///
/// Printed once from `main.rs` before serving so silent boot is gone.
/// Later enterprise logging (tracing subscriber) will build on this,
/// not replace it: banner stays on stdout, request logs go via tracing.
pub struct BannerInfo {
    pub version: String,
    pub bind_addr: String,
    pub port: u16,
    pub database_kind: String,
    pub build_mode: String,
}

/// Resolve listen port: `PORT` wins, then `API_PORT` (compose host-map
/// compat), then `8000`. Invalid values fall back to `8000`.
pub fn resolve_port() -> u16 {
    for key in ["PORT", "API_PORT"] {
        if let Ok(raw) = std::env::var(key) {
            if let Ok(p) = raw.trim().parse::<u16>() {
                if p != 0 {
                    return p;
                }
            }
        }
    }
    8000
}

/// Short database label for the banner. Never prints credentials.
pub fn database_kind() -> String {
    match std::env::var("DATABASE_URL") {
        Ok(url) if url.starts_with("postgres") => "postgres".to_string(),
        Ok(url) if url.starts_with("sqlite") => "sqlite".to_string(),
        Ok(_) => "custom".to_string(),
        Err(_) => "sqlite (dev)".to_string(),
    }
}

pub fn build_mode() -> String {
    if cfg!(debug_assertions) {
        "debug".to_string()
    } else {
        "release".to_string()
    }
}

pub fn render_banner(info: &BannerInfo) -> String {
    format!(
        r#"
   ____  _                            ____  ______  __
  / __ \/ /_  ____  ____ ____  ____ _ / __ )/_  __ \/ /
 / / / / __ \/ __ \/ __ `/ _ \/ __ `// __  / / /_/ / /
/ /_/ / / / / / / / /_/ /  __/ /_/ // /_/ / / _, _/_/
/\____/_/ /_/_/ /_/\__, /\___/\__,_/_____/ /_/ |_(_)
             \____/
  Ohngea PBX API  v{version}  ({mode})
  ─────────────────────────────────────
  Listening : http://{addr}:{port}
  Health    : http://{addr}:{port}/health
  API root  : http://{addr}:{port}/api/v1
  OpenAPI   : http://{addr}:{port}/api/v1/openapi.json
  Database  : {db}
"#,
        version = info.version,
        mode = info.build_mode,
        addr = info.bind_addr,
        port = info.port,
        db = info.database_kind,
    )
}
