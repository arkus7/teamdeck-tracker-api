use crate::auth::guard::AccessTokenAuthGuard;
use crate::auth::token::ResourceId;
use crate::project::ProjectModel;
use crate::resource::ResourceModel;
use crate::scalars::Date;
use crate::teamdeck::api::{CreateTimeEntryBody, TeamdeckApiClient, UpdateTimeEntryBody};
use crate::time_entry_tag::TimeEntryTag;
use async_graphql::{ComplexObject, Context, InputObject, Object, Result, ResultExt, SimpleObject};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use teamdeck::api::projects::Project;
use teamdeck::api::resources::Resource;
use teamdeck::api::time_entries::{TimeEntries, TimeEntry};
use teamdeck::api::AsyncQuery;
use teamdeck::AsyncTeamdeck;
use thiserror::Error;
use tracing::error;

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
#[graphql(complex)]
pub struct TimeEntryModel {
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
impl TimeEntryModel {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<ProjectModel>> {
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = Project::builder()
            .id(self.project_id as usize)
            .build()
            .unwrap();

        let project = endpoint.query_async(client).await?;
        Ok(project)
    }

    async fn resource(&self, ctx: &Context<'_>) -> Result<Option<ResourceModel>> {
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = Resource::builder()
            .id(self.resource_id as usize)
            .build()
            .unwrap();

        let resource = endpoint.query_async(client).await?;
        Ok(resource)
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
    async fn time_entries(
        &self,
        ctx: &Context<'_>,
        date: Option<Date>,
        project_id: Option<u64>,
    ) -> Result<Vec<TimeEntryModel>> {
        let resource_id = *ctx.data_unchecked::<ResourceId>();
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = TimeEntries::builder()
            .resource_id(resource_id.0)
            .build()
            .unwrap();

        let time_entries: Vec<TimeEntryModel> = endpoint.query_async(client).await?;

        let time_entries = match project_id {
            None => time_entries,
            Some(id) => time_entries
                .into_iter()
                .filter(|e| e.project_id == id)
                .collect(),
        };

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
    #[graphql(validator(min_items = 1))]
    pub tag_ids: Vec<u64>,
}

#[derive(InputObject, Debug, Serialize, Deserialize)]
pub struct UpdateTimeEntryInput {
    pub project_id: Option<u64>,
    pub minutes: Option<u64>,
    pub weekend_booking: Option<bool>,
    pub holidays_booking: Option<bool>,
    pub vacations_booking: Option<bool>,
    pub description: Option<String>,
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    #[graphql(validator(min_items = 1))]
    pub tag_ids: Option<Vec<u64>>,
}

#[derive(Debug, Error)]
enum UpdateTimeEntryError {
    #[error("You must be creator of the time entry to update it")]
    NotACreator,
}

#[Object]
impl TimeEntryMutation {
    #[tracing::instrument(name = "Create time entry for authorized user", skip(ctx))]
    #[graphql(guard = "AccessTokenAuthGuard::default()")]
    async fn create_time_entry(
        &self,
        ctx: &Context<'_>,
        time_entry: CreateTimeEntryInput,
    ) -> Result<TimeEntryModel> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource_id = *ctx.data_unchecked::<ResourceId>();

        let request_body = CreateTimeEntryBody::from_graphql_input(&time_entry, resource_id.into());
        let mut created_entry = client.add_time_entry(request_body).await.extend()?;

        let tag_ids = time_entry.tag_ids;
        let _ = client
            .update_time_entry_tags(created_entry.id, tag_ids)
            .await
            .extend()
            .map_err(|e| error!("{:?}", e));

        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = TimeEntry::builder()
            .id(created_entry.id as usize)
            .build()
            .unwrap();

        let created_entry = endpoint.query_async(client).await?;

        Ok(created_entry)
    }

    #[tracing::instrument(name = "Update time entry", skip(ctx))]
    #[graphql(guard = "AccessTokenAuthGuard::default()")]
    async fn update_time_entry(
        &self,
        ctx: &Context<'_>,
        time_entry_id: u64,
        update_data: UpdateTimeEntryInput,
    ) -> Result<TimeEntryModel> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let resource_id = *ctx.data_unchecked::<ResourceId>();
        let resource_id = resource_id.0;

        let td = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = TimeEntry::builder()
            .id(time_entry_id as usize)
            .build()
            .unwrap();

        let time_entry: TimeEntryModel = endpoint.query_async(td).await?;

        if time_entry.resource_id != resource_id {
            Err(UpdateTimeEntryError::NotACreator.into())
        } else {
            let UpdateTimeEntryInput {
                project_id,
                minutes,
                weekend_booking,
                holidays_booking,
                vacations_booking,
                description,
                start_date,
                end_date,
                tag_ids,
            } = update_data;
            let mut updated_entry = client
                .update_time_entry(
                    time_entry_id,
                    &UpdateTimeEntryBody {
                        project_id: project_id.unwrap_or(time_entry.project_id),
                        minutes: minutes.unwrap_or(time_entry.minutes),
                        weekend_booking,
                        holidays_booking,
                        vacations_booking,
                        description,
                        start_date: start_date.map(|d| d.0).unwrap_or(time_entry.start_date.0),
                        end_date: end_date.map(|d| d.0).unwrap_or(time_entry.end_date.0),
                        editor_resource_id: resource_id,
                        tags: tag_ids.clone(),
                    },
                )
                .await
                .extend()?;

            if let Some(tags) = tag_ids {
                if !tags.is_empty() {
                    let _ = client
                        .update_time_entry_tags(time_entry_id, tags)
                        .await
                        .extend()
                        .map_err(|e| error!("{:?}", e));

                    updated_entry = endpoint.query_async(td).await?;
                }
            }

            Ok(updated_entry)
        }
    }
}
