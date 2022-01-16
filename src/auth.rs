use async_graphql::{Context, Object, Result, SimpleObject};
use serde::Serialize;

use crate::teamdeck::{api::TeamdeckApiClient, error::TeamdeckApiError};

mod google {
    use serde::Deserialize;
    use thiserror::Error;

    const USER_INFO_EMAIL_SCOPE: &str = "https://www.googleapis.com/auth/userinfo.email";
    const OAUTH2_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
    const RESPONSE_TYPE_CODE: &str = "code";
    const ACCESS_TYPE_OFFLINE: &str = "offline";
    const EXPECTED_DOMAIN: &str = "moodup.team";

    #[derive(Error, Debug)]
    pub enum GoogleAuthError {
        #[error("decoding error")]
        TokenDecodeError { source: jsonwebtoken::errors::Error },
        #[error("received token from Google did not include `id_token` field")]
        IdTokenMissing,
        #[error("email `{0}` is not verified")]
        EmailNotVerified(String),
        #[error("invalid domain (expected {expected:?}, found {found:?})")]
        InvalidDomain { expected: String, found: String },
    }

    #[derive(Deserialize, Debug)]
    pub struct GoogleTokenResponse {
        access_token: String,
        refresh_token: Option<String>,
        expires_in: u64,
        id_token: Option<String>,
        scope: String,
        token_type: String,
    }

    #[derive(Deserialize)]
    struct GoogleTokenClaims {
        sub: String,
        email: String,
        email_verified: bool,
        #[serde(rename(deserialize = "hd"))]
        domain: String,
    }

    impl GoogleTokenResponse {
        pub fn email(&self) -> Result<String, GoogleAuthError> {
            let id_token = match &self.id_token {
                Some(token) => token,
                None => return Err(GoogleAuthError::IdTokenMissing),
            };

            let token_data = jsonwebtoken::dangerous_insecure_decode::<GoogleTokenClaims>(id_token)
                .map_err(|e| GoogleAuthError::TokenDecodeError { source: e })?;
            let claims = token_data.claims;

            if !claims.email_verified {
                return Err(GoogleAuthError::EmailNotVerified(claims.email));
            }

            if claims.domain != EXPECTED_DOMAIN {
                return Err(GoogleAuthError::InvalidDomain {
                    expected: EXPECTED_DOMAIN.to_string(),
                    found: claims.domain,
                });
            }

            Ok(claims.email)
        }
    }

    pub struct GoogleOAuth2;

    impl GoogleOAuth2 {
        // NOTE: Done this way in order to not being required to store
        // Google credentials on the clients. They simply ask for the URL
        // where they should redirect the user
        pub fn get_login_url() -> String {
            let base_url = OAUTH2_URL;
            let client_id = std::env::var("GOOGLE_OAUTH2_CLIENT_ID").unwrap();
            // TODO: Add redirect_uri to env variables
            let redirect_uri = "http://localhost:8000/google/redirect";
            let scope = USER_INFO_EMAIL_SCOPE;
            let response_type = RESPONSE_TYPE_CODE;
            // TODO: Do we need refresh tokens at all?
            let access_type = ACCESS_TYPE_OFFLINE;

            format!(
                "{}?client_id={}&redirect_uri={}&scope={}&response_type={}&access_type={}",
                base_url, client_id, redirect_uri, scope, response_type, access_type
            )
        }

        pub async fn exchange_code_for_token(
            code: String,
        ) -> Result<GoogleTokenResponse, Box<dyn std::error::Error>> {
            // FIXME: Don't use Box<dyn Error>, replace with something better
            // TODO: Refactor params, maybe introduce struct?
            let params = [
                (
                    "client_id",
                    std::env::var("GOOGLE_OAUTH2_CLIENT_ID").unwrap(),
                ),
                (
                    "client_secret",
                    std::env::var("GOOGLE_OAUTH2_CLIENT_SECRET").unwrap(),
                ),
                ("code", code),
                ("grant_type", "authorization_code".to_string()),
                (
                    "redirect_uri",
                    "http://localhost:8000/google/redirect".to_string(),
                ),
            ];

            let response = reqwest::Client::new()
                .post("https://oauth2.googleapis.com/token")
                .form(&params)
                .send()
                .await?
                .json()
                .await?;

            Ok(response)
        }
    }
}

mod token {
    use std::time::{SystemTime, Duration};

    use async_graphql::SimpleObject;
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct Claims {
        sub: String,
        iat: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        exp: Option<u64>,
        resource_id: u64,
    }

    trait Token {
        fn secret() -> String;

        fn encode_claims(claims: &Claims) -> Result<String, TokenError> {
            let headers = Header::default();
            let encoding_key = EncodingKey::from_secret(Self::secret().as_bytes());

            encode(&headers, &claims, &encoding_key)
                .map_err(|e| TokenError::EncodingError { source: e })
        }

        fn verify(token_str: &str) -> Result<Claims, TokenError> {
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

    struct AccessToken(Claims);
    impl Token for AccessToken {
        fn secret() -> String {
            std::env::var("JWT_ACCESS_TOKEN_SECRET").unwrap()
        }

        fn expiration_time() -> Option<Duration> {
            Some(Duration::from_secs(60 * 60 * 24))
        }
    }

    impl AccessToken {
        fn encode(&self) -> Result<String, TokenError> {
            Self::encode_claims(&self.0)
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

    #[derive(Debug, Error)]
    pub enum TokenError {
        #[error("unknown error occured during token creation")]
        Unknown,
        #[error("error while encoding token")]
        EncodingError { source: jsonwebtoken::errors::Error },
        #[error("error while decoding token")]
        DecodingError { source: jsonwebtoken::errors::Error },
    }

    impl TokenResponse {
        pub fn with_user_data(email: &str, resource_id: u64) -> Result<Self, TokenError> {
            let issued_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
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
                expires_in,
            })
        }
    }
}

#[derive(Default, Debug)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    #[tracing::instrument(name = "Fetch url for authorization")]
    async fn google_auth_url(&self) -> Result<String> {
        Ok(google::GoogleOAuth2::get_login_url())
    }
}

#[derive(Default, Debug)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    async fn login_with_google(
        &self,
        ctx: &Context<'_>,
        code: String,
    ) -> Result<token::TokenResponse> {
        let google_token = google::GoogleOAuth2::exchange_code_for_token(code).await?;
        let email = google_token.email()?;

        let teamdeck_api = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource = teamdeck_api.get_resource_by_email(&email).await?;

        if let Some(resource) = resource {
            let token = token::TokenResponse::with_user_data(&email, resource.id)?;
            Ok(token)
        } else {
            Err(async_graphql::Error::new(format!(
                "No Teamdeck account found with `{}` email",
                email
            )))
        }
    }
}
