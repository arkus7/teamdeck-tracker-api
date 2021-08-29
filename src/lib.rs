mod project;
mod resource;
mod scalars;
mod teamdeck;
mod timer;

use crate::project::ProjectQuery;
use crate::resource::ResourceQuery;
use crate::teamdeck::api::TeamdeckApiClient;
use crate::timer::{TimerMutation, TimerQuery, Timers};
use async_graphql::{EmptySubscription, MergedObject, Schema};
use async_graphql::extensions::ApolloTracing;

pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(TimerQuery, ResourceQuery, ProjectQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(TimerMutation);

pub fn create_schema() -> ApiSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(TeamdeckApiClient::default())
    .data(Timers::default())
        .extension(ApolloTracing)
    .finish()
}
