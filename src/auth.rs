use async_graphql::{Context, Object, Result, SimpleObject};
use serde::Serialize;

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
        pub fn get_login_url() -> String {
            let base_url = OAUTH2_URL;
            let client_id = std::env::var("GOOGLE_OAUTH2_CLIENT_ID").unwrap();
            let redirect_uri = "http://localhost:8000/google/redirect";
            let scope = USER_INFO_EMAIL_SCOPE;
            let response_type = RESPONSE_TYPE_CODE;
            let access_type = ACCESS_TYPE_OFFLINE;

            format!(
                "{}?client_id={}&redirect_uri={}&scope={}&response_type={}&access_type={}",
                base_url, client_id, redirect_uri, scope, response_type, access_type
            )
        }

        pub async fn exchange_code_for_token(
            code: String,
        ) -> Result<GoogleTokenResponse, Box<dyn std::error::Error>> {
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

#[derive(SimpleObject, Debug, Serialize)]
struct Token {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[Object]
impl AuthMutation {
    async fn login_with_google(&self, ctx: &Context<'_>, code: String) -> Result<Token> {
        let google_token = google::GoogleOAuth2::exchange_code_for_token(code).await?;
        dbg!(&google_token);
        let email = google_token.email()?;
        let token = Token {
            access_token: "".to_string(),
            refresh_token: "".to_string(),
            expires_in: 0,
        };
        Ok(token)
    }
}
