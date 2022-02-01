use crate::auth::guard::AccessTokenAuthGuard;
use crate::auth::token::ResourceId;
use crate::project::Project;
use crate::resource::Resource;
use crate::scalars::{Date, Time};
use crate::teamdeck::api::{CreateTimeEntryBody, TeamdeckApiClient};
use crate::time_entry_tag::TimeEntryTag;
use async_graphql::{ComplexObject, Context, InputObject, Object, Result, ResultExt, SimpleObject};
use chrono::Duration;
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

#[derive(Default, Debug)]
pub struct TimeEntryQuery;

#[Object]
impl TimeEntryQuery {
    #[tracing::instrument(name = "Fetching all time entries for resource", skip(ctx))]
    #[graphql(guard = "AccessTokenAuthGuard::default()")]
    async fn time_entries(&self, ctx: &Context<'_>, date: Option<Date>) -> Result<Vec<TimeEntry>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource_id = *ctx.data_unchecked::<ResourceId>();
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
    pub project_id: u64,
    pub weekend_booking: Option<bool>,
    pub holidays_booking: Option<bool>,
    pub vacations_booking: Option<bool>,
    pub description: Option<String>,
    pub minutes: Option<u64>,
    pub date: Option<Date>,
    pub tag_ids: Option<Vec<u64>>,
}

#[Object]
impl TimeEntryMutation {
    #[tracing::instrument(name = "Create time entry for authorized user", skip(ctx))]
    #[graphql(guard = "AccessTokenAuthGuard::default()")]
    async fn create_time_entry(
        &self,
        ctx: &Context<'_>,
        time_entry: CreateTimeEntryInput,
    ) -> Result<TimeEntry> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource_id = *ctx.data_unchecked::<ResourceId>();

        let request_body = CreateTimeEntryBody::from_graphql_input(&time_entry, resource_id);
        let created_entry = client.add_time_entry(request_body).await.extend()?;

        if let Some(_tags) = time_entry.tag_ids {
            // TODO: Update created entry with tags
            // client.update_time_entry_tags(created_entry.id, tags).await.extend()?;
        }

        Ok(created_entry)
    }
}
