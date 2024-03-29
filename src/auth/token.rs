use std::{
    ops::Deref,
    time::{Duration, SystemTime},
};

use async_graphql::SimpleObject;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceId(pub u64);

impl Deref for ResourceId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ResourceId> for u64 {
    fn from(value: ResourceId) -> Self {
        value.0
    }
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("error while encoding token")]
    EncodingError { source: jsonwebtoken::errors::Error },
    #[error("error while decoding token")]
    DecodingError { source: jsonwebtoken::errors::Error },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    iat: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    exp: Option<u64>,
    resource_id: ResourceId,
}

trait Token {
    fn secret() -> String;

    fn encode_claims(claims: &Claims) -> Result<String, TokenError> {
        let headers = Header::default();
        let encoding_key = EncodingKey::from_secret(Self::secret().as_bytes());

        encode(&headers, &claims, &encoding_key)
            .map_err(|e| TokenError::EncodingError { source: e })
    }

    fn decode_claims(token_str: &str) -> Result<Claims, TokenError> {
        let token = token_str.to_string();
        let secret = Self::secret();
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::default();

        let token_data = decode::<Claims>(&token, &decoding_key, &validation)
            .map_err(|e| TokenError::DecodingError { source: e })?;

        Ok(token_data.claims)
    }

    fn expiration_time() -> Option<Duration> {
        None
    }
}

#[derive(Debug)]
pub struct AccessToken(Claims);
impl Token for AccessToken {
    fn secret() -> String {
        std::env::var("JWT_ACCESS_TOKEN_SECRET").unwrap()
    }

    fn expiration_time() -> Option<Duration> {
        Some(Duration::from_secs(60 * 60 * 24 * 7))
    }
}

impl AccessToken {
    fn encode(&self) -> Result<String, TokenError> {
        Self::encode_claims(&self.0)
    }

    pub fn verify(token_str: &str) -> Result<AccessToken, TokenError> {
        let claims = Self::decode_claims(token_str)?;

        Ok(Self(claims))
    }

    pub fn resource_id(&self) -> ResourceId {
        self.0.resource_id
    }
}

struct RefreshToken(Claims);
impl Token for RefreshToken {
    fn secret() -> String {
        std::env::var("JWT_REFRESH_TOKEN_SECRET").unwrap()
    }
}

impl RefreshToken {
    fn encode(&self) -> Result<String, TokenError> {
        Self::encode_claims(&self.0)
    }
}

#[derive(SimpleObject, Debug, Serialize)]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

impl TokenResponse {
    pub fn with_user_data(email: &str, resource_id: ResourceId) -> Result<Self, TokenError> {
        let issued_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let expires_in = AccessToken::expiration_time().unwrap_or_default();
        let access_token_claims = Claims {
            sub: email.to_string(),
            iat: issued_at.as_secs(),
            exp: Some((issued_at + expires_in).as_secs()),
            resource_id,
        };
        let refresh_token_claims = Claims {
            exp: None,
            ..access_token_claims.clone()
        };

        let access_token = AccessToken(access_token_claims).encode()?;
        let refresh_token = RefreshToken(refresh_token_claims).encode()?;

        Ok(TokenResponse {
            access_token,
            refresh_token,
            expires_in: expires_in.as_secs(),
        })
    }
}
