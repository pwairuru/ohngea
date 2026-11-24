//! TOTP second factor — parity with PHP
//! `core/authentication/resources/classes/plugins/totp.php`.
//!
//! PHP stores `user_totp_secret` (base32) on `v_users`, shows an
//! `otpauth://totp/...` QR on first login, then verifies codes with
//! `google_authenticator::checkCode` (RFC 6238, SHA1, 30s step).
//! This module is std-only (no new deps): SHA1 + HMAC-SHA1 + base32.

use std::time::{SystemTime, UNIX_EPOCH};

pub const TOTP_DIGITS: u32 = 6;
pub const TOTP_PERIOD: u64 = 30;
const VERIFY_WINDOW: i64 = 1;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---- SHA1 (FIPS 180-4) ----

fn sha1(msg: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    let mut data = msg.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let tmp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = tmp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn hmac_sha1(key: &[u8], msg: &[u8]) -> [u8; 20] {
    let mut k = key.to_vec();
    if k.len() > 64 {
        k = sha1(&k).to_vec();
    }
    while k.len() < 64 {
        k.push(0);
    }
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5C).collect();
    let mut inner = ipad;
    inner.extend_from_slice(msg);
    let inner_hash = sha1(&inner);
    let mut outer = opad;
    outer.extend_from_slice(&inner_hash);
    sha1(&outer)
}

// ---- base32 (RFC 4648, uppercase, no padding on encode) ----

const B32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'2'..=b'7' => Some(c - b'2' + 26),
        b'a'..=b'z' => Some(c - b'a'),
        _ => None,
    }
}

pub(crate) fn base32_encode(raw: &[u8]) -> String {
    let mut out = String::new();
    let mut bits: u32 = 0;
    let mut nbits = 0;
    for &b in raw {
        bits = (bits << 8) | b as u32;
        nbits += 8;
        while nbits >= 5 {
            nbits -= 5;
            out.push(B32_ALPHABET[((bits >> nbits) & 31) as usize] as char);
        }
    }
    if nbits > 0 {
        out.push(B32_ALPHABET[((bits << (5 - nbits)) & 31) as usize] as char);
    }
    out
}

pub(crate) fn base32_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut bits: u32 = 0;
    let mut nbits = 0;
    for c in s.bytes().filter(|&b| b != b'=' && b != b' ' && b != b'-') {
        let v = base32_val(c).ok_or_else(|| "invalid base32".to_string())?;
        bits = (bits << 5) | v as u32;
        nbits += 5;
        if nbits >= 8 {
            nbits -= 8;
            out.push(((bits >> nbits) & 0xFF) as u8);
        }
    }
    if out.is_empty() {
        return Err("empty secret".to_string());
    }
    Ok(out)
}

// ---- base64url (for reset tokens) ----

const B64U_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub(crate) fn base64url_encode(raw: &[u8]) -> String {
    let mut out = String::new();
    let mut bits: u32 = 0;
    let mut nbits = 0;
    for &b in raw {
        bits = (bits << 8) | b as u32;
        nbits += 8;
        while nbits >= 6 {
            nbits -= 6;
            out.push(B64U_ALPHABET[((bits >> nbits) & 63) as usize] as char);
        }
    }
    if nbits > 0 {
        out.push(B64U_ALPHABET[((bits << (6 - nbits)) & 63) as usize] as char);
    }
    out
}

pub(crate) fn base64url_decode(s: &str) -> Result<Vec<u8>, String> {
    let val = |c: u8| -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'-' => Some(62),
            b'_' => Some(63),
            _ => None,
        }
    };
    let mut out = Vec::new();
    let mut bits: u32 = 0;
    let mut nbits = 0;
    for c in s.bytes() {
        let v = val(c).ok_or_else(|| "invalid base64url".to_string())?;
        bits = (bits << 6) | v as u32;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            out.push(((bits >> nbits) & 0xFF) as u8);
        }
    }
    Ok(out)
}

// ---- TOTP ----

fn hotp(key: &[u8], counter: u64) -> String {
    let mac = hmac_sha1(key, &counter.to_be_bytes());
    let offset = (mac[19] & 0x0F) as usize;
    let code = (((mac[offset] & 0x7F) as u32) << 24)
        | ((mac[offset + 1] as u32) << 16)
        | ((mac[offset + 2] as u32) << 8)
        | (mac[offset + 3] as u32);
    format!("{:06}", code % 10u32.pow(TOTP_DIGITS))
}

/// TOTP code for `secret` (base32) at `unix_secs`. Mirrors PHP
/// `google_authenticator::getCode` (SHA1, 30s step, 6 digits).
pub fn totp_at(secret: &str, unix_secs: u64) -> Result<String, String> {
    let key = base32_decode(secret)?;
    Ok(hotp(&key, unix_secs / TOTP_PERIOD))
}

/// TOTP code for the current time step.
pub fn current_code(secret: &str) -> Result<String, String> {
    totp_at(secret, now_secs())
}

/// Verify with ±1 step window (PHP clock-skew parity). Constant-time compare.
pub fn verify_code(secret: &str, code: &str, unix_secs: u64) -> bool {
    let code = code.trim();
    if code.len() != TOTP_DIGITS as usize || !code.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let Ok(key) = base32_decode(secret) else {
        return false;
    };
    let step = (unix_secs / TOTP_PERIOD) as i64;
    for s in (step - VERIFY_WINDOW)..=(step + VERIFY_WINDOW) {
        if s < 0 {
            continue;
        }
        let cand = hotp(&key, s as u64);
        let a = cand.as_bytes();
        let b = code.as_bytes();
        let mut diff = 0u8;
        for i in 0..a.len() {
            diff |= a[i] ^ b[i];
        }
        if diff == 0 {
            return true;
        }
    }
    false
}

/// Verify against current time.
pub fn verify_current(secret: &str, code: &str) -> bool {
    verify_code(secret, code, now_secs())
}

/// New random 160-bit secret, base32 (PHP `generate_password(20)` parity).
/// Std-only: `/dev/urandom` with a time-seeded hash fallback.
pub fn generate_secret() -> String {
    let mut raw = [0u8; 20];
    let mut filled = false;
    // NOTE: never `fs::read("/dev/urandom")` — it has no EOF so read-to-end
    // grows the Vec unbounded until OOM. Read exactly 20 bytes instead.
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read as _;
        if f.read_exact(&mut raw).is_ok() {
            filled = true;
        }
    }
    if !filled {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        for (i, chunk) in raw.chunks_mut(8).enumerate() {
            let mut h = DefaultHasher::new();
            (
                now_secs(),
                std::process::id(),
                CTR.fetch_add(1, Ordering::Relaxed),
                i,
            )
                .hash(&mut h);
            chunk.copy_from_slice(&h.finish().to_be_bytes()[..chunk.len()]);
        }
    }
    base32_encode(&raw)
}

/// `otpauth://` URL for QR enrolment (PHP totp.php shape:
/// `otpauth://totp/{user}?secret=..&issuer=..`).
pub fn otpauth_url(username: &str, issuer: &str, secret: &str) -> String {
    format!("otpauth://totp/{username}?secret={secret}&issuer={issuer}")
}

// ---- password-reset tokens ----
//
// PHP ground truth `fusionpbx/resources/login.php`: the reset link carries
// an encrypted `username|domain_uuid|password_hash|timestamp` payload, the
// link expires after 60 minutes, and it is single-use because confirm
// re-checks the stored password hash (a used link's hash no longer matches).
//
// Rust port: same shape as a signed JWT instead of symmetric encryption.
// `marker` = hex(SHA1(password_hash)): confirm recomputes it from the live
// row, so a consumed link (password changed → new hash) fails closed.
// Expiry rides on the standard `exp` claim; no new table, no new dep
// (jsonwebtoken is already used for access tokens).

/// Purpose tag separating reset tokens from access/mfa tokens.
pub const RESET_PURPOSE: &str = "password-reset";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResetClaims {
    pub sub: String,
    pub domain_id: String,
    pub marker: String,
    pub purpose: String,
    pub exp: usize,
    pub iat: usize,
}

/// Marker binding a reset token to the current password hash (single-use:
/// any password change rotates the hash and invalidates outstanding links).
#[must_use]
pub fn reset_marker(password_hash: &str) -> String {
    sha1(password_hash.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn reset_secret() -> Vec<u8> {
    // Same secret as access tokens (single JWT_SECRET ops surface).
    // Tests set JWT_SECRET implicitly via the debug fallback.
    if let Ok(s) = std::env::var("JWT_SECRET") {
        let t = s.trim().to_owned();
        if !t.is_empty() {
            return t.into_bytes();
        }
    }
    "test-secret".to_string().into_bytes()
}

fn reset_now() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(1_700_000_000)
}

/// Signed reset token. `ttl_secs` is 3600 in production (PHP 60-minute
/// parity); tests pass negative values for already-expired tokens.
pub fn encode_reset_token(sub: &str, domain_id: &str, marker: &str, ttl_secs: i64) -> String {
    let now = reset_now() as i64;
    let claims = ResetClaims {
        sub: sub.to_owned(),
        domain_id: domain_id.to_owned(),
        marker: marker.to_owned(),
        purpose: RESET_PURPOSE.to_owned(),
        exp: (now + ttl_secs).max(0) as usize,
        iat: now.max(0) as usize,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(&reset_secret()),
    )
    .unwrap()
}

/// Verify signature + expiry + purpose tag. Marker-vs-DB comparison is the
/// caller's job (it needs the live user row).
pub fn decode_reset_token(token: &str) -> Result<ResetClaims, String> {
    let data = jsonwebtoken::decode::<ResetClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&reset_secret()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| e.to_string())?;
    if data.claims.purpose != RESET_PURPOSE {
        return Err("wrong token purpose".to_string());
    }
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 Appendix B, SHA1 secret "12345678901234567890".
    const RFC_SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    #[test]
    fn rfc6238_t59_is_287082() {
        assert_eq!(totp_at(RFC_SECRET, 59).unwrap(), "287082");
    }

    #[test]
    fn rfc6238_t1111111109_is_081804() {
        assert_eq!(totp_at(RFC_SECRET, 1_111_111_109).unwrap(), "081804");
    }

    #[test]
    fn rfc6238_t1234567890_is_005924() {
        assert_eq!(totp_at(RFC_SECRET, 1_234_567_890).unwrap(), "005924");
    }

    #[test]
    fn verify_accepts_window_rejects_wrong() {
        assert!(verify_code(RFC_SECRET, "287082", 59));
        assert!(verify_code(RFC_SECRET, "287082", 59 + 29)); // same step
        assert!(verify_code(RFC_SECRET, "287082", 89)); // +1 step window
        assert!(!verify_code(RFC_SECRET, "287082", 120)); // +2 steps out
        assert!(!verify_code(RFC_SECRET, "000000", 59));
        assert!(!verify_code(RFC_SECRET, "abcdef", 59));
        assert!(!verify_code(RFC_SECRET, "", 59));
    }

    #[test]
    fn invalid_secret_rejected() {
        assert!(totp_at("!!!!", 59).is_err());
        assert!(!verify_code("!!!!", "123456", 59));
    }

    #[test]
    fn generated_secret_roundtrips_20_bytes() {
        let s = generate_secret();
        assert_eq!(base32_decode(&s).unwrap().len(), 20);
        assert!(current_code(&s).is_ok());
        let code = current_code(&s).unwrap();
        assert!(verify_current(&s, &code));
    }

    #[test]
    fn otpauth_url_shape_matches_php() {
        let u = otpauth_url("alice", "example.com", "ABCDEF");
        assert!(u.starts_with("otpauth://totp/alice?"));
        assert!(u.contains("secret=ABCDEF"));
        assert!(u.contains("issuer=example.com"));
    }

    #[test]
    fn sha1_known_vector() {
        let d = sha1(b"abc");
        assert_eq!(
            d.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }
}
