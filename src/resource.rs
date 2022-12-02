use crate::auth::{guard::AccessTokenAuthGuard, token::ResourceId};
use async_graphql::{Context, Object, Result, SimpleObject};
use serde::{Deserialize, Serialize};
use teamdeck::{
    api::{
        paged,
        resources::{Resource, Resources, ResourcesSortBy},
        sort_by::SortBy,
        AsyncQuery, Pagination,
    },
    AsyncTeamdeck,
};

#[derive(Serialize, Deserialize, SimpleObject, Debug, Clone)]
pub struct ResourceModel {
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
    async fn resource(&self, ctx: &Context<'_>, resource_id: u64) -> Result<Option<ResourceModel>> {
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = Resource::builder()
            .id(resource_id as usize)
            .build()
            .unwrap();

        let resource = endpoint.query_async(client).await?;
        Ok(resource)
    }

    #[tracing::instrument(name = "Fetching all resources", skip(ctx))]
    async fn resources(&self, ctx: &Context<'_>) -> Result<Vec<ResourceModel>> {
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = Resources::builder()
            .sort(SortBy::Asc(ResourcesSortBy::Name))
            .build()
            .unwrap();
        let resources = paged(endpoint, Pagination::All).query_async(client).await?;
        Ok(resources)
    }

    #[tracing::instrument(name = "Fetching authorized user", skip(ctx))]
    #[graphql(guard = "AccessTokenAuthGuard::default()")]
    async fn me(&self, ctx: &Context<'_>) -> Result<Option<ResourceModel>> {
        let resource_id = ctx.data_unchecked::<ResourceId>().0;
        let td = ctx.data_unchecked::<AsyncTeamdeck>();

        let endpoint = Resource::builder()
            .id(resource_id as usize)
            .build()
            .unwrap();

        let resource = endpoint.query_async(td).await?;

        Ok(resource)
    }
}
