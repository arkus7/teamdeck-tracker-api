use serde::{Deserialize, Serialize};
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

/// Struct representing response from Google OAuth2 API
/// after exchanging authorization code for token.
///
/// Available fields:
///     - `access_token` (String)
///         The token that your application sends to authorize a Google API request.
///     - `expires_in` (u64)
///         The remaining lifetime of the access token in seconds.
///     - `refresh_token` (Option<String>)
///         A token that you can use to obtain a new access token.
///         Refresh tokens are valid until the user revokes access.
///         Again, this field is only present in this response if you set the
///         access_type parameter to `offline` in the initial request to Google's authorization server.
///     - `scope` (String)
///         The scopes of access granted by the access_token expressed as a list of space-delimited, case-sensitive strings.
///     - `token_type` (String)
///         The type of token returned. At this time, this field's value is always set to `Bearer`.
#[derive(Deserialize, Debug)]
pub struct GoogleTokenResponse {
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct GoogleTokenClaims {
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

#[derive(Debug, Serialize)]
struct ExchangeCodeForTokenParams {
    client_id: String,
    client_secret: String,
    code: String,
    grant_type: String,
    redirect_uri: String,
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

        let params = ExchangeCodeForTokenParams {
            client_id: std::env::var("GOOGLE_OAUTH2_CLIENT_ID").unwrap(),
            client_secret: std::env::var("GOOGLE_OAUTH2_CLIENT_SECRET").unwrap(),
            code,
            // TODO: move authorization code to const?
            grant_type: "authorization_code".to_string(),
            // TODO: get redirect url from env variable
            redirect_uri: "http://localhost:8000/google/redirect".to_string(),
        };

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
