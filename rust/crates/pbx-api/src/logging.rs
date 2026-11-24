//! Structured enterprise logging for `pbx-api`.
//!
//! Single owner of process-wide [`tracing`] subscriber setup. Controlled by
//! four env vars (see `rust/README.md` runbook):
//!
//! - `LOG_LEVEL` (`trace|debug|info|warn|error`, default `info`) — base level
//!   applied to our crates; noisy deps (`sea_orm`, `hyper`, `tower_http`)
//!   stay at `warn` unless the level is `warn`/`error`.
//! - `RUST_LOG` — full filter directive; wins over `LOG_LEVEL` when set.
//! - `LOG_FORMAT` (`json|pretty`; default `json` in release, `pretty` in
//!   debug builds). File and stdout sinks share the format.
//! - `LOG_FILE` — full file path (e.g. `/var/log/ohngea/pbx-api.log`); when
//!   set, a daily-rotating file appender is added alongside stdout.
//!
//! Bad env values never panic and never block boot: a stderr warning is
//! emitted and a safe fallback applies. Writers are non-blocking; the
//! returned [`LogGuards`] must be held to end of `main` so no log lines are
//! dropped. Tests use [`init_for_tests`], which is idempotent within the
//! test process.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::layer::SubscriberExt;
#[cfg(feature = "otel")]
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
#[cfg(feature = "otel")]
use tracing_subscriber::Layer;
use tracing_subscriber::{fmt, EnvFilter};

/// Full filter directive used when no valid level override is given.
pub fn default_filter() -> String {
    "pbx_api=info,pbx_fs=info,pbx_db=info,pbx_core=info,sea_orm=warn,hyper=warn,tower_http=warn"
        .to_string()
}

/// Map a bare `LOG_LEVEL` value to a full filter directive.
///
/// Unknown values (e.g. `verbose`) emit a stderr warning and fall back to
/// [`default_filter`]. `None`/empty means "unset" and falls back silently.
pub fn resolve_filter(level: Option<&str>) -> String {
    let Some(raw) = level.map(str::trim).filter(|s| !s.is_empty()) else {
        return default_filter();
    };
    let ours = raw.to_ascii_lowercase();
    if !matches!(ours.as_str(), "trace" | "debug" | "info" | "warn" | "error") {
        eprintln!("logging: invalid LOG_LEVEL={raw:?}, falling back to default filter");
        return default_filter();
    }
    // Noisy deps stay at `warn` unless the caller asked for `warn`/`error`
    // (quieter than `warn`), so debug/trace never floods third-party logs.
    let deps = match ours.as_str() {
        "warn" | "error" => ours.clone(),
        _ => "warn".to_string(),
    };
    format!(
        "pbx_api={ours},pbx_fs={ours},pbx_db={ours},pbx_core={ours},\
         sea_orm={deps},hyper={deps},tower_http={deps}"
    )
}

/// Output encoding for both sinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Json,
    Pretty,
}

impl LogFormat {
    fn default_for_build() -> Self {
        if cfg!(debug_assertions) {
            Self::Pretty
        } else {
            Self::Json
        }
    }
}

/// Map a `LOG_FORMAT` value to [`LogFormat`]. Unknown values emit a stderr
/// warning and fall back to the build default (`pretty` in debug, `json` in
/// release). `None`/empty means "unset" and falls back silently.
pub fn resolve_format(raw: Option<&str>) -> LogFormat {
    let Some(value) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return LogFormat::default_for_build();
    };
    match value.to_ascii_lowercase().as_str() {
        "json" => LogFormat::Json,
        "pretty" => LogFormat::Pretty,
        _ => {
            eprintln!("logging: invalid LOG_FORMAT={value:?}, falling back to build default");
            LogFormat::default_for_build()
        }
    }
}

/// Split a `LOG_FILE` full path into `(dir, file_stem)` for daily rotation,
/// e.g. `/var/log/ohngea/pbx-api.log` → (`/var/log/ohngea`, `pbx-api.log`).
/// Total function: missing parents become `.`, missing file names become
/// `pbx-api.log`. Never panics.
pub fn split_log_file(path: &str) -> (PathBuf, String) {
    let p = Path::new(path);
    let stem = p
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "pbx-api.log".to_string());
    let dir = p
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    (dir, stem)
}

const SECRET_KEYS: &[&str] = &["password", "passwd", "secret", "token"];
const ESL_MAX_CHARS: usize = 200;

fn is_secret_key(token: &str) -> bool {
    let key = token
        .trim()
        .trim_matches(['=', ':', ',', ';', '"', '\''])
        .to_ascii_lowercase();
    SECRET_KEYS.iter().any(|k| key.contains(k))
}

/// Quote char that opens a multi-word value, unless it also closes in the
/// same token (e.g. `password="hunter 2"` spans two whitespace tokens).
fn opens_quote(value: &str) -> Option<char> {
    let c = value.chars().next()?;
    if (c == '"' || c == '\'') && !(value.len() > c.len_utf8() && value.ends_with(c)) {
        Some(c)
    } else {
        None
    }
}

/// Redact an ESL command string for log fields: masks
/// `password|passwd|secret|token` values (both `key=value`/`key:value` and
/// bare `key value` forms, including quoted multi-word values), then caps
/// the result at 200 chars. Never emits the secret itself.
pub fn redact_esl(cmd: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut tokens = cmd.split_whitespace();
    while let Some(tok) = tokens.next() {
        if let Some((key, value)) = tok.split_once('=').or_else(|| tok.split_once(':')) {
            if is_secret_key(key) {
                if let Some(q) = opens_quote(value) {
                    for rest in tokens.by_ref() {
                        if rest.contains(q) {
                            break;
                        }
                    }
                }
                parts.push(format!("{key}=***"));
                continue;
            }
        } else if is_secret_key(tok) {
            parts.push(tok.to_string());
            if let Some(value) = tokens.next() {
                if let Some(q) = opens_quote(value) {
                    for rest in tokens.by_ref() {
                        if rest.contains(q) {
                            break;
                        }
                    }
                }
                parts.push("***".to_string());
            }
            continue;
        }
        parts.push(tok.to_string());
    }
    let joined = parts.join(" ");
    if joined.chars().count() > ESL_MAX_CHARS {
        joined.chars().take(ESL_MAX_CHARS).collect()
    } else {
        joined
    }
}

/// Holds the non-blocking writer guards; dropping them flushes and shuts
/// down the background logging threads. Keep alive to end of `main`.
pub struct LogGuards {
    _stdout: WorkerGuard,
    _file: Option<WorkerGuard>,
}

fn filter_from_env() -> EnvFilter {
    if let Some(directives) = std::env::var("RUST_LOG")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        match EnvFilter::try_new(&directives) {
            Ok(filter) => return filter,
            Err(e) => eprintln!(
                "logging: invalid RUST_LOG={directives:?} ({e}), falling back to LOG_LEVEL/default"
            ),
        }
    }
    let directives = resolve_filter(std::env::var("LOG_LEVEL").ok().as_deref());
    EnvFilter::try_new(&directives).unwrap_or_else(|_| EnvFilter::new("info"))
}

/// Install the process-wide subscriber. Call first in `main` and hold the
/// returned guards to end of `main`. Never panics: bad env values warn on
/// stderr and fall back; an already-set global subscriber is left as-is.
pub fn init() -> LogGuards {
    let format = resolve_format(std::env::var("LOG_FORMAT").ok().as_deref());
    let filter = filter_from_env();
    let (stdout_writer, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());

    // Build the file writer first so its guard is hoisted out of the
    // format match below (json/pretty layers are different types per arm).
    let mut file_guard: Option<WorkerGuard> = None;
    let file_writer: Option<NonBlocking> = match std::env::var("LOG_FILE")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(path) => {
            let (dir, stem) = split_log_file(&path);
            match std::fs::create_dir_all(&dir) {
                Ok(()) => {
                    let appender = tracing_appender::rolling::daily(&dir, &stem);
                    let (writer, guard) = tracing_appender::non_blocking(appender);
                    file_guard = Some(guard);
                    Some(writer)
                }
                Err(e) => {
                    eprintln!(
                        "logging: cannot create log dir {} ({e}); continuing with stdout only",
                        dir.display()
                    );
                    None
                }
            }
        }
        None => None,
    };

    // Box the whole subscriber: the json/pretty stacks are different types.
    let subscriber: Box<dyn tracing::Subscriber + Send + Sync> = match format {
        LogFormat::Json => {
            let sub = tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().json().with_writer(stdout_writer))
                .with(file_writer.map(|w| fmt::layer().json().with_writer(w)));
            #[cfg(feature = "otel")]
            let sub = sub.with(otel_layer());
            Box::new(sub)
        }
        LogFormat::Pretty => {
            let sub = tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().pretty().with_writer(stdout_writer))
                .with(file_writer.map(|w| fmt::layer().pretty().with_writer(w)));
            #[cfg(feature = "otel")]
            let sub = sub.with(otel_layer());
            Box::new(sub)
        }
    };
    // `try_init` keeps a pre-existing global (e.g. set by a test harness)
    // instead of panicking.
    let _ = subscriber.try_init();

    LogGuards {
        _stdout: stdout_guard,
        _file: file_guard,
    }
}

static TEST_INIT: OnceLock<()> = OnceLock::new();

/// Install a sink (no-output) subscriber for tests. Idempotent within the
/// process: parallel tests calling this never double-init or panic.
pub fn init_for_tests() {
    TEST_INIT.get_or_init(|| {
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new(default_filter()))
            .with(fmt::layer().with_writer(std::io::sink));
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

/// OpenTelemetry layer hook (only compiled with `--features otel`, off by
/// default at zero cost). No exporter is wired in this work (spec non-goal):
/// spans flow into an SDK provider with no processors. Add an OTLP
/// `BatchSpanProcessor` here once the collector endpoint is configured.
#[cfg(feature = "otel")]
fn otel_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber + for<'span> LookupSpan<'span>,
{
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::trace::SdkTracerProvider;

    static PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
    let provider = PROVIDER.get_or_init(|| SdkTracerProvider::builder().build());
    tracing_opentelemetry::layer().with_tracer(provider.tracer("pbx-api"))
}
