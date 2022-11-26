mod google;
pub mod guard;
pub mod token;

use async_graphql::{Context, Object, Result};
use teamdeck::{
    api::{resources::Resources, AsyncQuery},
    AsyncTeamdeck,
};

use crate::resource::ResourceModel;

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
    async fn exchange_authorization_code_for_token(
        &self,
        ctx: &Context<'_>,
        authorization_code: String,
    ) -> Result<token::TokenResponse> {
        let google_token =
            google::GoogleOAuth2::exchange_code_for_token(authorization_code).await?;
        let email = google_token.email()?;

        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = Resources::builder().email(&email).build().unwrap();

        let resources: Vec<ResourceModel> = endpoint.query_async(client).await?;
        let resource = resources.first();

        if let Some(resource) = resource {
            let token =
                token::TokenResponse::with_user_data(&email, token::ResourceId(resource.id))?;
            Ok(token)
        } else {
            Err(async_graphql::Error::new(format!(
                "No Teamdeck account found with `{}` email",
                email
            )))
        }
    }
}
