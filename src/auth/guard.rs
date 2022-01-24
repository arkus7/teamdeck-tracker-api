use async_graphql::{async_trait::async_trait, guard::Guard};

use super::token::AccessToken;

pub struct AuthGuard;

impl AuthGuard {
    pub fn new() -> Self {
        AuthGuard
    }
}

#[async_trait]
impl Guard for AuthGuard {
    async fn check(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<()> {
        if ctx.data_opt::<AccessToken>().is_some() {
            Ok(())
        } else {
            Err("Unauthorized".into())
        }
    }
}
