use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
}

impl Claims {
    pub fn from_token(secret: &str, token: &str) -> Result<Self> {
        Ok(jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?
        .claims)
    }

    pub fn into_token(&self, secret: &str) -> Result<String> {
        Ok(jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?)
    }
}
