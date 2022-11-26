pub mod auth;
mod project;
mod resource;
mod scalars;
mod teamdeck;
mod time_entry;
mod time_entry_tag;
mod timer;

use crate::project::ProjectQuery;
use crate::resource::ResourceQuery;
use crate::teamdeck::api::TeamdeckApiClient;
use crate::time_entry::{TimeEntryMutation, TimeEntryQuery};
use crate::timer::{TimerMutation, TimerQuery, Timers};
use ::teamdeck::AsyncTeamdeck;
use async_graphql::extensions::ApolloTracing;
use async_graphql::{EmptySubscription, MergedObject, Schema};
use auth::{AuthMutation, AuthQuery};
use time_entry_tag::TimeEntryTagQuery;

pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    TimerQuery,
    ResourceQuery,
    ProjectQuery,
    TimeEntryQuery,
    TimeEntryTagQuery,
    AuthQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(TimerMutation, TimeEntryMutation, AuthMutation);

pub fn create_schema() -> ApiSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(TeamdeckApiClient::default())
    .data(AsyncTeamdeck::new(
        std::env::var("TEAMDECK_API_KEY").unwrap(),
    ))
    .data(Timers::default())
    .extension(ApolloTracing)
    .finish()
}
