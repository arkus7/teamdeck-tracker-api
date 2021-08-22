mod timer;
mod teamdeck;
mod resource;

use crate::timer::{TimerQuery};
use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};
use crate::resource::ResourceQuery;

pub type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(TimerQuery, ResourceQuery);

pub fn create_schema() -> ApiSchema {
    Schema::new(QueryRoot::default(), EmptyMutation, EmptySubscription)
}
