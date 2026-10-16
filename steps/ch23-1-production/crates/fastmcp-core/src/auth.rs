//! HS256 verification, a JWKS `kid` lookup, and protected-resource metadata.
//!
//! Errors say the token was rejected. They do not include the secret.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The symmetric algorithm this chapter verifies.
pub const AUTH_ALGORITHM: &str = "HS256";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    aud: String,
    exp: usize,
}

/// Sign a token for tests and the bench. The secret stays with the caller.
pub fn sign(secret: &[u8], audience: &str, subject: &str, exp: usize) -> Result<String, String> {
    let claims = Claims {
        sub: subject.to_string(),
        aud: audience.to_string(),
        exp,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| "token rejected".to_string())
}

/// Verify an HS256 token. The audience must match. A bad signature is rejected.
pub fn verify(secret: &[u8], token: &str, audience: &str) -> Result<String, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[audience]);
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|_| "token rejected".to_string())?;
    Ok(data.claims.sub)
}

/// The first `kid` in a JWKS document. No network fetch.
pub fn jwks_kid(document: &Value) -> Option<String> {
    document
        .pointer("/keys/0/kid")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// RFC 9728 fields this chapter advertises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceMetadata {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub scopes_supported: Vec<String>,
}

impl ResourceMetadata {
    pub fn new(
        resource: impl Into<String>,
        authorization_servers: Vec<String>,
        scopes_supported: Vec<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            authorization_servers,
            scopes_supported,
        }
    }
}
