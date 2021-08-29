use crate::project::Project;
use crate::resource::Resource;
use crate::scalars::{Date, DateTime, Time};
use crate::teamdeck::api::{CreateTimeEntryBody, TeamdeckApiClient};
use crate::teamdeck::error::TeamdeckApiError;
use async_graphql::{ComplexObject, Context, InputObject, Object, Result, ResultExt, SimpleObject};
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
#[graphql(complex)]
pub struct TimeEntry {
    id: u64,
    resource_id: u64,
    project_id: u64,
    minutes: u64,
    weekend_booking: bool,
    holidays_booking: bool,
    vacations_booking: bool,
    description: Option<String>,
    external_id: Option<String>,
    start_date: Date,
    end_date: Date,
    creator_resource_id: Option<u64>,
    editor_resource_id: Option<u64>,
    tags: Option<Vec<TimeEntryTag>>,
}

#[ComplexObject]
impl TimeEntry {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let project = client.get_project_by_id(self.project_id).await.extend();
        Ok(project.unwrap_or(None))
    }

    async fn resource(&self, ctx: &Context<'_>) -> Result<Option<Resource>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource = client.get_resource_by_id(self.resource_id).await.extend();
        Ok(resource.unwrap_or(None))
    }

    async fn formatted_duration(&self) -> Result<String> {
        let duration = Duration::minutes(self.minutes as i64);
        let duration_in_seconds = duration.num_seconds();
        let minutes = (duration_in_seconds / 60) % 60;
        let hours = (duration_in_seconds / 60) / 60;
        Ok(format!("{}:{:02}", hours, minutes))
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
    async fn time_entries(
        &self,
        ctx: &Context<'_>,
        resource_id: u64,
        date: Option<Date>,
    ) -> Result<Vec<TimeEntry>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let time_entries = client
            .get_time_entries(resource_id, date.map(|d| d.0))
            .await
            .extend()?;
        Ok(time_entries)
    }
}

#[derive(Default, Debug)]
pub struct TimeEntryMutation;

#[derive(InputObject, Debug, Serialize, Deserialize)]
pub struct CreateTimeEntryInput {
    pub resource_id: u64,
    pub project_id: u64,
    pub weekend_booking: Option<bool>,
    pub holidays_booking: Option<bool>,
    pub vacations_booking: Option<bool>,
    pub description: Option<String>,
}

#[derive(InputObject, Debug)]
pub struct CreateTimeEntryFromDurationInput {
    entry_input: CreateTimeEntryInput,
    duration: Time,
    date: Option<Date>,
}

#[derive(InputObject, Debug)]
pub struct CreateTimeEntryFromTimeRangeInput {
    entry_input: CreateTimeEntryInput,
    start: Time,
    end: Time,
    date: Option<Date>,
}

#[Object]
impl TimeEntryMutation {
    #[tracing::instrument(name = "Add time entry from duration", skip(ctx))]
    async fn add_time_entry_from_duration(
        &self,
        ctx: &Context<'_>,
        input: CreateTimeEntryFromDurationInput,
    ) -> Result<TimeEntry> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let minutes = input.duration.to_duration().num_minutes().unsigned_abs();
        let request_body =
            CreateTimeEntryBody::from_graphql_input(input.entry_input, input.date, minutes);
        let time_entry = client.add_time_entry(request_body).await.extend()?;
        Ok(time_entry)
    }

    #[tracing::instrument(name = "Add time entry from time range", skip(ctx))]
    async fn add_time_entry_from_time_range(
        &self,
        ctx: &Context<'_>,
        input: CreateTimeEntryFromTimeRangeInput,
    ) -> Result<TimeEntry> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let minutes = input
            .start
            .duration_to(input.end)
            .num_minutes()
            .unsigned_abs();
        let request_body =
            CreateTimeEntryBody::from_graphql_input(input.entry_input, input.date, minutes);
        let time_entry = client.add_time_entry(request_body).await.extend()?;
        Ok(time_entry)
    }
}
