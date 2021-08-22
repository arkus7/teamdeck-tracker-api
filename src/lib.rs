mod project;
mod resource;
mod teamdeck;
mod timer;

use crate::project::ProjectQuery;
use crate::resource::ResourceQuery;
use crate::teamdeck::api::TeamdeckApiClient;
use crate::timer::TimerQuery;
use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};

pub type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(TimerQuery, ResourceQuery, ProjectQuery);

pub fn create_schema() -> ApiSchema {
    Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(TeamdeckApiClient::default())
        .finish()
}
