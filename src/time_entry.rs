use crate::teamdeck::api::TeamdeckApiClient;
use async_graphql::{Context, Object, Result, SimpleObject, ComplexObject, ErrorExtensions, ResultExt};
use serde::{Deserialize, Serialize};
use crate::scalars::{DateTime, Date};
use crate::project::Project;

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
#[graphql(complex)]
pub struct TimeEntry {
    id: u64,
    resource_id: u64,
    project_id: u64,
    minutes: Option<u64>,
    weekend_booking: bool,
    holidays_booking: bool,
    vacations_booking: bool,
    description: Option<String>,
    external_id: Option<String>,
    start_date: Date,
    end_date: Date,
    creator_resource_id: Option<u64>,
    editor_resource_id: Option<u64>,
    tags: Vec<TimeEntryTag>,
}

#[ComplexObject]
impl TimeEntry {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
pub struct TimeEntryTag {
    id: u64,
    name: String,
    icon: String,
    color: String,
}

#[derive(Default, Debug)]
pub struct TimeEntryQuery;

#[Object]
impl TimeEntryQuery {
    #[tracing::instrument(name = "Fetching all time entries for resource", skip(ctx))]
    async fn time_entries(&self, ctx: &Context<'_>, resource_id: u64) -> Result<Vec<TimeEntry>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let time_entries = client.get_time_entries(resource_id, None).await.extend()?;
        Ok(time_entries)
    }
}
