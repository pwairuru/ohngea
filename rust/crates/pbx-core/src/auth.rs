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
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-secret".to_string())
        .into_bytes()
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
    bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap_or_else(|_| format!("stub:{password}"))
}

pub fn verify_password(plain: &str, hashed: &str) -> bool {
    bcrypt::verify(plain, hashed).unwrap_or(false)
}
