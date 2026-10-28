use serde::{Deserialize, Serialize};

// ---- Claims (parity with app/auth/service.py::create_token) ----
// Python payload: {sub=user_uuid, domain_id, username, permissions (plural),
// exp, iat}. `perms` alias accepts legacy test tokens on decode; serialize
// always emits `permissions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub domain_id: String,
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default, alias = "perms")]
    pub permissions: Vec<String>,
    pub exp: usize,
    #[serde(default = "default_iat")]
    pub iat: usize,
}

fn default_username() -> String {
    "admin".to_string()
}

fn default_iat() -> usize {
    1700000000
}

fn jwt_secret() -> Vec<u8> {
    if let Ok(s) = std::env::var("JWT_SECRET") {
        let t = s.trim().to_owned();
        if !t.is_empty() {
            // Release must never run on the well-known test value: the
            // binary refuses to boot without a real secret (see
            // pbx-api::main), this is defense-in-depth at the decode site.
            #[cfg(not(debug_assertions))]
            if t == "test-secret" {
                panic!("FATAL: JWT_SECRET=test-secret is forbidden in release builds");
            }
            return t.into_bytes();
        }
    }
    // Debug/test fallback keeps `cargo test` and local boots working
    // without env. Release builds panic above instead of defaulting.
    #[cfg(debug_assertions)]
    {
        "test-secret".to_string().into_bytes()
    }
    #[cfg(not(debug_assertions))]
    {
        panic!("FATAL: JWT_SECRET must be set to a strong random value in release builds");
    }
}

fn now_secs() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(1700000000)
}

pub fn encode_claims(sub: &str, domain_id: &str, username: &str, permissions: &[String]) -> String {
    let now = now_secs();
    let claims = Claims {
        sub: sub.to_owned(),
        domain_id: domain_id.to_owned(),
        username: username.to_owned(),
        permissions: permissions.to_vec(),
        exp: now + 3600,
        iat: now,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(&jwt_secret()),
    )
    .unwrap()
}

pub fn encode_test_token() -> String {
    encode_claims(
        "00000000-0000-0000-0000-000000000001",
        "default",
        "admin",
        &["admin".to_string()],
    )
}

pub fn decode_test_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&jwt_secret()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(data.claims)
}

// ---- bcrypt (parity with app/auth/service.py hash_password/verify_password) ----
pub fn hash_password(password: &str) -> String {
    // Never persist plaintext: on bcrypt failure store an unverifiable random
    // marker (verify_password always returns false for it).
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .unwrap_or_else(|_| format!("unverifiable:{}:{}", now_secs(), uuid_fallback()))
}

fn uuid_fallback() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // No uuid dep in pbx-core: hash time + thread id for uniqueness. Only used
    // for the failure marker, never as an identifier.
    let mut h = DefaultHasher::new();
    (now_secs(), std::thread::current().id()).hash(&mut h);
    format!("{:016x}", h.finish())
}

pub fn verify_password(plain: &str, hashed: &str) -> bool {
    bcrypt::verify(plain, hashed).unwrap_or(false)
}
