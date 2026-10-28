use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub domain_id: String,
    pub perms: Vec<String>,
    pub exp: usize,
}

pub fn encode_test_token() -> String {
    let claims = Claims {
        sub: "00000000-0000-0000-0000-000000000001".into(),
        domain_id: "default".into(),
        perms: vec!["admin".into()],
        exp: 2000000000,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"test-secret"),
    )
    .unwrap()
}

pub fn decode_test_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(b"test-secret"),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(data.claims)
}
