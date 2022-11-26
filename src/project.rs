use async_graphql::{Context, Object, Result, SimpleObject};
use serde::{Deserialize, Serialize};
use teamdeck::{
    api::{projects::Projects, AsyncQuery},
    AsyncTeamdeck,
};

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
pub struct ProjectModel {
    id: u64,
    name: String,
    color: String,
    archived: bool,
}

#[derive(Default, Debug)]
pub struct ProjectQuery;

#[Object]
impl ProjectQuery {
    #[tracing::instrument(name = "Fetching all projects", skip(ctx))]
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<ProjectModel>> {
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = Projects::builder().build().unwrap();

        let projects = endpoint.query_async(client).await?;
        Ok(projects)
    }
}
