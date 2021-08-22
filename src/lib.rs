use async_graphql::{EmptyMutation, EmptySubscription, Schema, Object, Context};

pub type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello_world(&self) -> &str {
        "Hello, World"
    }
}