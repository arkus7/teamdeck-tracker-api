use crate::teamdeck::api::TeamdeckApiClient;
use async_graphql::{Context, Object, Result, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
pub struct Project {
    id: u64,
    name: String,
    color: String,
    archived: bool,
}

#[derive(Default)]
pub struct ProjectQuery;

#[Object]
impl ProjectQuery {
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let projects = client.get_projects().await?;
        Ok(projects)
    }
}
