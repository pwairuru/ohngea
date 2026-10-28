//! Scoped Extism WASM sandbox for the plugin system.
//!
//! Host-side complement to the registry in [`crate::plugins`]: the registry
//! decides *which* plugins may load (manifest + license gate + tenant
//! toggles); this module decides *how* their code runs — inside an
//! [`extism`] sandbox with no ambient authority.
//!
//! Limits (all set on every call):
//! - WASI **off** by default ([`PluginSandbox::new`]); a guest importing
//!   `wasi_snapshot_preview1` fails to link unless explicitly enabled.
//! - HTTP hosts disallowed (`disallow_all_hosts`), no path mounts.
//! - Linear memory capped at [`DEFAULT_MAX_MEMORY_PAGES`] pages.
//! - Wasmtime fuel capped at [`DEFAULT_FUEL_LIMIT`] per call
//!   (`PluginBuilder::with_fuel_limit`) — runaway loops die on fuel
//!   exhaustion even before the epoch-interruption timeout fires.
//! - Wall-clock timeout [`DEFAULT_TIMEOUT_MS`] (`Manifest::with_timeout`).
//!
//! Host functions (allowlist, the *only* imports a guest can see):
//! - `pbx_log(offset)` — records the call in a per-sandbox counter.
//!   Payload logging is a later step (see below).
//! - `db_get(key_id) -> 1/0` — **stub**: answers from the static
//!   [`DB_GET_ALLOWLIST`]. Real table/key lookup via SeaORM is the explicit
//!   next step (PDK + `db_get(table, key)` with per-tenant RLS).
//!
//! Tenant gating is NOT duplicated here: routes are already 403-gated by
//! `plugin_gate` / per-handler checks, so the sandbox is only reached for
//! enabled plugins. [`PluginSandbox::register`] enforces the license gate
//! (mirrors [`crate::plugins::is_license_allowed`]) before any exec.
//!
//! NEXT (not in scope): PDK crate for guest authors, real `db_get` wiring,
//! `CompiledPlugin`/pool caching (currently one compile per `call_plugin`;
//! per-call bytes are a single `Arc` slice copy at the `Manifest` boundary).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use extism::{Manifest, PluginBuilder, UserData, Val, ValType, Wasm};

use crate::plugins::{is_license_allowed, PluginManifest};

/// Default wall-clock timeout per guest call (epoch interruption).
pub const DEFAULT_TIMEOUT_MS: u64 = 1_000;
/// Default wasmtime fuel budget per guest call (incl. instantiation).
pub const DEFAULT_FUEL_LIMIT: u64 = 10_000_000;
/// Default linear-memory cap, in 64 KiB WASM pages (16 = 1 MiB).
pub const DEFAULT_MAX_MEMORY_PAGES: u32 = 16;

/// Stub allowlist for the `db_get` host function: numeric key ids answered
/// with `1`. Everything else answers `0`. Placeholder until real SeaORM
/// wiring lands (see module docs).
pub const DB_GET_ALLOWLIST: &[i64] = &[1, 2, 3];

/// Reference guest compiled from `plugins/example/plugin.wat` (kept in sync
/// by the `guest_wasm_matches_wat` test). Powers the `example`-route hook.
pub const EXAMPLE_GUEST_WASM: &[u8] = include_bytes!("../../../../plugins/example/plugin.wasm");

/// Cap per-plugin WASM bytes: the registry is manifest-gated but must not
/// buffer arbitrarily large uploads in RAM.
pub const MAX_WASM_BYTES: usize = 8 * 1024 * 1024;

/// Sandbox failure modes.
#[derive(Debug)]
pub enum SandboxError {
    /// Manifest license not in the allowed set — rejected before any exec.
    LicenseDenied(String),
    /// No WASM bytes registered under this plugin name.
    UnknownPlugin(String),
    /// WASM payload exceeds [`MAX_WASM_BYTES`] — rejected before storing.
    TooLarge { name: String, bytes: usize },
    /// Guest failed to link/instantiate (e.g. WASI import with WASI off).
    Build(String),
    /// Guest trapped or exceeded fuel/timeout during the call.
    Call(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LicenseDenied(m) => write!(f, "plugin license denied: {m}"),
            Self::UnknownPlugin(n) => write!(f, "no wasm registered for plugin {n}"),
            Self::TooLarge { name, bytes } => {
                write!(f, "wasm for plugin {name} too large: {bytes} bytes")
            }
            Self::Build(e) => write!(f, "guest build failed: {e}"),
            Self::Call(e) => write!(f, "guest call failed: {e}"),
        }
    }
}

impl std::error::Error for SandboxError {}

/// Scoped sandbox: registry of (manifest-gated) guests + execution limits.
///
/// Fresh [`extism::Plugin`] per [`call_plugin`](Self::call_plugin): no guest
/// state leaks between calls. `Clone`d [`UserData`] counter observes
/// `pbx_log` calls.
pub struct PluginSandbox {
    // `Arc` shares bytes between registry and in-flight calls: `call_plugin`
    // clones the `Arc` (pointer copy) and copies bytes exactly once at the
    // `Wasm::data` boundary, instead of cloning a `Vec<u8>` per call.
    registry: HashMap<String, Arc<[u8]>>,
    wasi_enabled: bool,
    timeout_ms: u64,
    fuel_limit: u64,
    max_memory_pages: u32,
    log_calls: UserData<u64>,
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginSandbox {
    /// Locked-down defaults: WASI off, timeout + fuel + memory cap set,
    /// no hosts, no path mounts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            wasi_enabled: false,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            fuel_limit: DEFAULT_FUEL_LIMIT,
            max_memory_pages: DEFAULT_MAX_MEMORY_PAGES,
            log_calls: UserData::new(0),
        }
    }

    /// Override the wall-clock timeout (epoch interruption).
    #[must_use]
    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Explicitly enable WASI for guests that need it (off by default).
    #[must_use]
    pub fn with_wasi(mut self, enabled: bool) -> Self {
        self.wasi_enabled = enabled;
        self
    }

    #[must_use]
    pub fn wasi_enabled(&self) -> bool {
        self.wasi_enabled
    }

    #[must_use]
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    #[must_use]
    pub fn fuel_limit(&self) -> u64 {
        self.fuel_limit
    }

    #[must_use]
    pub fn max_memory_pages(&self) -> u32 {
        self.max_memory_pages
    }

    /// How many times guests called the `pbx_log` host function.
    #[must_use]
    pub fn log_calls(&self) -> u64 {
        self.log_calls
            .get()
            .ok()
            .and_then(|m| m.lock().ok().map(|c| *c))
            .unwrap_or(0)
    }

    /// Manifest gate: only license-allowed plugins may register WASM.
    /// Tenant enablement is enforced at the route layer before any call.
    pub fn register(
        &mut self,
        manifest: &PluginManifest,
        wasm: Vec<u8>,
    ) -> Result<(), SandboxError> {
        if !is_license_allowed(&manifest.license) {
            return Err(SandboxError::LicenseDenied(format!(
                "license {} not allowed for plugin {}",
                manifest.license, manifest.name
            )));
        }
        if wasm.len() > MAX_WASM_BYTES {
            return Err(SandboxError::TooLarge {
                name: manifest.name.clone(),
                bytes: wasm.len(),
            });
        }
        self.registry.insert(manifest.name.clone(), Arc::from(wasm));
        Ok(())
    }

    /// Run `func` of plugin `name` with `input` bytes, return output bytes.
    ///
    /// Manifest: timeout + memory cap + no hosts. Builder: WASI flag, fuel
    /// limit, `pbx_log` + `db_get` host fns (the full import allowlist).
    pub fn call_plugin(
        &self,
        name: &str,
        func: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, SandboxError> {
        let bytes = self
            .registry
            .get(name)
            .ok_or_else(|| SandboxError::UnknownPlugin(name.to_string()))?;
        // Single copy at the owned-`Vec<u8>` boundary `Wasm::data` requires.
        let manifest = Manifest::new([Wasm::data(bytes.to_vec())])
            .with_timeout(Duration::from_millis(self.timeout_ms))
            .with_memory_max(self.max_memory_pages)
            .disallow_all_hosts();
        let mut plugin = PluginBuilder::new(manifest)
            .with_wasi(self.wasi_enabled)
            .with_fuel_limit(self.fuel_limit)
            .with_function(
                "pbx_log",
                [ValType::I64],
                [],
                self.log_calls.clone(),
                |_, _, _, data| {
                    if let Ok(m) = data.get() {
                        if let Ok(mut c) = m.lock() {
                            *c += 1;
                        }
                    }
                    Ok(())
                },
            )
            .with_function(
                "db_get",
                [ValType::I64],
                [ValType::I64],
                UserData::new(()),
                |_, inputs, outputs, _| {
                    let key = inputs.first().and_then(|v| v.i64()).unwrap_or(-1);
                    let allowed = if DB_GET_ALLOWLIST.contains(&key) {
                        1
                    } else {
                        0
                    };
                    if let Some(slot) = outputs.first_mut() {
                        *slot = Val::I64(allowed);
                    }
                    Ok(())
                },
            )
            .build()
            .map_err(|e| SandboxError::Build(e.to_string()))?;
        plugin
            .call(func, input)
            .map_err(|e| SandboxError::Call(e.to_string()))
    }
}
