use async_graphql::{Object, Result, SimpleObject};
use crate::teamdeck::api::TeamdeckApiClient;
use serde::{Deserialize, Serialize};
use async_graphql::connection::{Connection, EmptyFields, query, Edge};

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
pub struct Resource {
    id: u64,
    name: String,
    active: bool,
    avatar: Option<String>,
    email: String,
    role: String,
}

#[derive(Default)]
pub struct ResourceQuery(TeamdeckApiClient);

#[Object]
impl ResourceQuery {
    async fn resource(&self, resource_id: u64) -> Result<Resource> {
        let resource = self.0.get_resource_by_id(resource_id).await?;
        Ok(resource)
    }
}