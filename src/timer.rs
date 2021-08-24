use crate::scalars::Date;
use async_graphql::*;
use chrono::{Utc};
use std::sync::{Mutex, Arc};

#[derive(SimpleObject, Clone)]
pub struct Timer {
    id: u64,
    resource_id: u64,
    started_at: Date,
    ended_at: Option<Date>,
    description: Option<String>,
    project_id: u64,
}

impl Timer {
    pub fn from_input(input: CreateTimerInput) -> Timer {
        Timer {
            id: 0,
            resource_id: input.resource_id,
            started_at: Date(Utc::now()),
            ended_at: None,
            description: input.description,
            project_id: input.project_id,
        }
    }
}

#[derive(Default, Debug)]
pub struct TimerQuery;

#[Object]
impl TimerQuery {
    #[tracing::instrument(
        name = "Find current timer",
        skip(self, ctx),
    )]
    async fn current_timer<'ctx>(&'ctx self, ctx: &Context<'ctx>, resource_id: u64) -> Result<Option<Timer>> {
        let timers = ctx.data_unchecked::<Timers>();
        Ok(timers.get_by_resource_id(resource_id).last().cloned())
    }

    #[tracing::instrument(
        name = "Find all timers for resource",
        skip(self, ctx),
    )]
    async fn timers<'ctx>(&'ctx self, ctx: &Context<'ctx>, resource_id: u64) -> Result<Vec<Timer>> {
        let timers = ctx.data_unchecked::<Timers>();
        Ok(timers.get_by_resource_id(resource_id))
    }
}

#[derive(InputObject, Debug)]
pub struct CreateTimerInput {
    resource_id: u64,
    project_id: u64,
    description: Option<String>
}

#[derive(Default)]
pub struct TimerMutation;

#[Object]
impl TimerMutation {
    #[tracing::instrument(
        name = "Starting new timer",
        skip(self, ctx),
    )]
    async fn start_timer(&self, ctx: &Context<'_>, input: CreateTimerInput) -> Result<Timer> {
        let timer = Timer::from_input(input);
        let timers = ctx.data_unchecked::<Timers>();
        timers.add(&timer);
        Ok(timer)
    }
}

pub struct Timers {
    data: Arc<Mutex<Vec<Timer>>>
}

impl Default for Timers {
    fn default() -> Self {
        Timers {
            data: Arc::new(Mutex::new(vec![]))
        }
    }
}

impl Timers {
    fn get_by_resource_id(&self, resource_id: u64) -> Vec<Timer> {
        let timers = self.data.lock().unwrap();
        timers.iter().filter(|t| t.resource_id == resource_id).cloned().collect()
    }

    fn add(&self, timer: &Timer) {
        self.data.lock().unwrap().push(timer.clone())
    }
}
