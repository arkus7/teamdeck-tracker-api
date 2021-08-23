use crate::teamdeck::api::TeamdeckApiClient;
use async_graphql::connection::{query, Connection, Edge, EmptyFields};
use async_graphql::{Context, InputObject, Object, Result, ResultExt, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
pub struct Resource {
    id: u64,
    name: String,
    active: bool,
    avatar: Option<String>,
    email: Option<String>,
    role: Option<String>,
}

#[derive(Default)]
pub struct ResourceQuery;

#[Object]
impl ResourceQuery {
    async fn resource(&self, ctx: &Context<'_>, resource_id: u64) -> Result<Resource> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource = client.get_resource_by_id(resource_id).await.extend()?;
        Ok(resource)
    }

    async fn resources(&self, ctx: &Context<'_>) -> Result<Vec<Resource>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resources = client.get_resources().await.extend()?;
        Ok(resources)
    }
}
