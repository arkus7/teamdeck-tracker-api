use crate::teamdeck::api::TeamdeckApiClient;
use async_graphql::connection::{query, Connection, Edge, EmptyFields};
use async_graphql::{Context, Object, Result, SimpleObject};
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
        let resource = client.get_resource_by_id(resource_id).await?;
        Ok(resource)
    }
}
