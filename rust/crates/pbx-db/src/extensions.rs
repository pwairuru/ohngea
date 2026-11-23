//! Extension helpers — PHP ground truth (`fusionpbx/app/extensions/`):
//! - `extension_edit.php` requires the extension to be non-empty only (no
//!   digit/length regex server-side); the old `^\d{2,7}$` rule came from the
//!   Python intermediate and wrongly rejected PHP-legal values.
//! - blank `password` auto-generates (PHP `generate_password`, default
//!   length 20 via `extension.password_length` setting).
//! - blank `user_context` defaults to the domain name.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn rng_next(state: &mut u64) -> u64 {
    // xorshift64* — std-only PRNG (no rand dep); good enough for SIP passwords.
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    *state = state.wrapping_mul(0x2545F4914F6CDD1D);
    *state
}

fn rng_seed() -> u64 {
    // PHP ground truth uses `random_int` (CSPRNG). Seed from the OS entropy
    // pool when available; fall back to the old time/pid mix otherwise.
    // Counter + pid folded in so concurrent calls never share a stream.
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut entropy = 0u64;
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let mut buf = [0u8; 8];
        if f.read_exact(&mut buf).is_ok() {
            entropy = u64::from_ne_bytes(buf);
        }
    }
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15);
    let pid = std::process::id() as u64;
    entropy
        .wrapping_add(t.wrapping_mul(0xBF58476D1CE4E5B9))
        .wrapping_add(c.wrapping_mul(0x94D049BB133111EB))
        .wrapping_add(pid)
        | 1
}

fn pick(state: &mut u64, set: &[u8]) -> char {
    set[(rng_next(state) % set.len() as u64) as usize] as char
}

/// Validate an extension number. PHP ground truth
/// (`extension_edit.php`): `if (empty($extension))` is the only
/// server-side rule — alphanumeric values are legal. The width cap (32)
/// mirrors the `extensions.extension VARCHAR(32)` column, not PHP.
pub fn validate_extension(v: &str) -> Result<(), String> {
    let t = v.trim();
    if t.is_empty() {
        return Err("extension required".to_owned());
    }
    if t.len() > 32 {
        return Err("extension must be at most 32 characters".to_owned());
    }
    Ok(())
}

/// Generate a password of `length` chars. Mirrors
/// `app/extensions/service.py::generate_password` (default 20): at least
/// one lower, one upper, one digit, one symbol when `length >= 4`.
#[must_use]
pub fn generate_password(length: usize) -> String {
    let mut state = rng_seed();
    let mut pwd: Vec<char> = Vec::with_capacity(length);
    if length >= 4 {
        pwd.push(pick(&mut state, LOWER));
        pwd.push(pick(&mut state, UPPER));
        pwd.push(pick(&mut state, DIGITS));
        pwd.push(pick(&mut state, SYMBOLS));
        let all: Vec<u8> = [LOWER, UPPER, DIGITS, SYMBOLS].concat();
        for _ in 4..length {
            pwd.push(pick(&mut state, &all));
        }
        // Fisher-Yates shuffle with the same PRNG.
        for i in (1..pwd.len()).rev() {
            let j = (rng_next(&mut state) % (i as u64 + 1)) as usize;
            pwd.swap(i, j);
        }
    } else {
        let all: Vec<u8> = [LOWER, UPPER, DIGITS, SYMBOLS].concat();
        for _ in 0..length {
            pwd.push(pick(&mut state, &all));
        }
    }
    pwd.into_iter().collect()
}

/// Return the stored password, auto-generating 20 chars when blank.
/// Mirrors `service.py::create_extension` (`if not dump.get("password")`).
#[must_use]
pub fn ensure_password(password: Option<&str>) -> String {
    match password {
        Some(p) if !p.is_empty() => p.to_owned(),
        _ => generate_password(20),
    }
}

/// Default `user_context` to the domain name when blank. Mirrors
/// `service.py::create_extension` (`if not dump.get("user_context")`).
#[must_use]
pub fn default_user_context(user_context: Option<&str>, domain_name: &str) -> String {
    match user_context {
        Some(u) if !u.is_empty() => u.to_owned(),
        _ => domain_name.to_owned(),
    }
}
