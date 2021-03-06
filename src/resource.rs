use crate::{
    auth::{guard::AccessTokenAuthGuard, token::ResourceId},
    teamdeck::api::TeamdeckApiClient,
};
use async_graphql::{Context, Object, Result, ResultExt, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, SimpleObject, Debug, Clone)]
pub struct Resource {
    pub id: u64,
    name: String,
    active: bool,
    avatar: Option<String>,
    email: Option<String>,
    role: Option<String>,
}

#[derive(Default, Debug)]
pub struct ResourceQuery;

#[Object]
impl ResourceQuery {
    #[tracing::instrument(name = "Fetching resource by id", skip(ctx))]
    async fn resource(&self, ctx: &Context<'_>, resource_id: u64) -> Result<Option<Resource>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource = client.get_resource_by_id(resource_id).await.extend()?;
        Ok(resource)
    }

    #[tracing::instrument(name = "Fetching all resources", skip(ctx))]
    async fn resources(&self, ctx: &Context<'_>) -> Result<Vec<Resource>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resources = client.get_resources().await.extend()?;
        Ok(resources)
    }

    #[tracing::instrument(name = "Fetching authorized user", skip(ctx))]
    #[graphql(guard = "AccessTokenAuthGuard::default()")]
    async fn me(&self, ctx: &Context<'_>) -> Result<Option<Resource>> {
        let resource_id = *ctx.data_unchecked::<ResourceId>();
        let client = ctx.data_unchecked::<TeamdeckApiClient>();

        let resource = client.get_resource_by_id(resource_id).await.extend()?;

        Ok(resource)
    }
}
