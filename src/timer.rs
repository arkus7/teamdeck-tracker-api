use async_graphql::*;
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct Timer {
    id: u64,
    resource_id: u64,
    started_at: Option<u64>,
    ended_at: Option<u64>,
    description: Option<String>,
    project_id: u64,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            id: 0,
            resource_id: 0,
            started_at: None,
            ended_at: None,
            description: None,
            project_id: 0,
        }
    }
}

#[derive(Default)]
pub struct TimerQuery;
#[Object]
impl TimerQuery {
    async fn current_timer(&self, resource_id: u64) -> Result<Option<Timer>> {
        println!("{}", resource_id);
        Ok(Some(Timer::new()))
    }
}