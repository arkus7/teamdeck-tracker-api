use async_graphql::{async_trait::async_trait, Guard};
use thiserror::Error;

use super::token::{AccessToken, ResourceId};

#[derive(Debug)]
pub struct AccessTokenAuthGuard;

impl AccessTokenAuthGuard {
    pub fn new() -> Self {
        AccessTokenAuthGuard
    }
}

impl Default for AccessTokenAuthGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Unauthorized, missing, invalid or expired access token")]
    InvalidAccessToken,
}

#[async_trait]
impl Guard for AccessTokenAuthGuard {
    #[tracing::instrument(name = "Checking access token with guard", skip(ctx))]
    async fn check(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<()> {
        if ctx.data_opt::<AccessToken>().is_some() && ctx.data_opt::<ResourceId>().is_some() {
            Ok(())
        } else {
            Err(AuthError::InvalidAccessToken.into())
        }
    }
}
