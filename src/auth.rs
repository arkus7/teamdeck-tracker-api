mod google;
mod token;

use async_graphql::{Context, Object, Result};

use crate::teamdeck::api::TeamdeckApiClient;

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
