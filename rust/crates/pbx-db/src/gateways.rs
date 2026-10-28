//! Gateway column defaults — mirrors `app/gateways/models.py`:
//! `expire_seconds=800`, `retry_seconds=30`, `context="public"`,
//! `profile="external"`, `gateway_register=true`.

/// Default `expire_seconds` (Python: `Field(default=800)`).
#[must_use]
pub const fn default_expire_seconds() -> i32 {
    800
}

/// Default `retry_seconds` (Python: `Field(default=30)`).
#[must_use]
pub const fn default_retry_seconds() -> i32 {
    30
}

/// Default `context` (Python: `Field(default="public")`).
#[must_use]
pub fn default_context() -> &'static str {
    "public"
}

/// Default `profile` (Python: `Field(default="external")`).
#[must_use]
pub fn default_profile() -> &'static str {
    "external"
}
