mod timer;

use async_graphql::{EmptyMutation, EmptySubscription, Schema, Object, Context, Result, MergedObject};
use crate::timer::{Timer, TimerQuery};

pub type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(MergedObject, Default)]
struct QueryRoot(TimerQuery);

pub fn create_schema() -> ApiSchema {
    Schema::new(QueryRoot::default(), EmptyMutation, EmptySubscription)
}