mod timer;

use crate::timer::{Timer, TimerQuery};
use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};

pub type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(TimerQuery);

pub fn create_schema() -> ApiSchema {
    Schema::new(QueryRoot::default(), EmptyMutation, EmptySubscription)
}
