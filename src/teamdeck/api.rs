use crate::scalars::Date;
use crate::teamdeck::error::TeamdeckApiError;
use crate::time_entry::{CreateTimeEntryInput, TimeEntryModel};
use chrono::{NaiveDate, Utc};
use reqwest;
use reqwest::IntoUrl;
use serde::Serialize;
use std::fmt::Debug;

const API_KEY_ENV_VARIABLE: &str = "TEAMDECK_API_KEY";
const API_KEY_HEADER_NAME: &str = "X-Api-Key";

pub struct TeamdeckApiClient {
    api_key: String,
}

impl Default for TeamdeckApiClient {
    fn default() -> Self {
        TeamdeckApiClient::from_env()
    }
}

#[derive(Debug)]
pub struct PaginationInfo {
    pub total_count: u64,
    pub pages_count: u64,
    pub current_page: u64,
    pub items_per_page: u64,
}

#[derive(Debug)]
pub struct Page<S: Serialize> {
    pub items: Vec<S>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct CreateTimeEntryBody {
    pub resource_id: u64,
    pub project_id: u64,
    pub minutes: u64,
    pub weekend_booking: Option<bool>,
    pub holidays_booking: Option<bool>,
    pub vacations_booking: Option<bool>,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub creator_resource_id: u64,
    pub editor_resource_id: u64,
    pub tags: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTimeEntryBody {
    pub project_id: u64,
    pub minutes: u64,
    pub weekend_booking: Option<bool>,
    pub holidays_booking: Option<bool>,
    pub vacations_booking: Option<bool>,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub editor_resource_id: u64,
    pub tags: Option<Vec<u64>>,
}

impl CreateTimeEntryBody {
    pub fn from_graphql_input(input: &CreateTimeEntryInput, resource_id: u64) -> Self {
        let date = input
            .date
            .unwrap_or_else(|| Date(Utc::now().date_naive()))
            .0;
        CreateTimeEntryBody {
            resource_id,
            project_id: input.project_id,
            minutes: input.minutes.unwrap_or(1),
            weekend_booking: input.weekend_booking,
            holidays_booking: input.holidays_booking,
            vacations_booking: input.vacations_booking,
            description: input.description.clone(),
            start_date: date,
            end_date: date,
            creator_resource_id: resource_id,
            editor_resource_id: resource_id,
            tags: input.tag_ids.clone(),
        }
    }
}

impl TeamdeckApiClient {
    fn from_env() -> Self {
        Self {
            api_key: std::env::var(API_KEY_ENV_VARIABLE)
                .unwrap_or_else(|_| panic!("Missing {} env variable", API_KEY_ENV_VARIABLE)),
        }
    }

    #[tracing::instrument(name = "Update time entry by ID", skip(self), err)]
    pub async fn update_time_entry(
        &self,
        time_entry_id: u64,
        body: &UpdateTimeEntryBody,
    ) -> Result<TimeEntryModel, TeamdeckApiError> {
        let updated_entry = self
            .put(format!(
                "https://api.teamdeck.io/v1/time-entries/{}",
                time_entry_id
            ))
            .json(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(updated_entry)
    }

    #[tracing::instrument(name = "Update time entry tags", skip(self), err)]
    pub async fn update_time_entry_tags(
        &self,
        time_entry_id: u64,
        tag_ids: Vec<u64>,
    ) -> Result<Vec<u64>, TeamdeckApiError> {
        let tags = self
            .put(format!(
                "https://api.teamdeck.io/v1/time-entries/{time_entry_id}/tags"
            ))
            .json(&tag_ids)
            .send()
            .await?
            .json()
            .await?;

        Ok(tags)
    }

    #[tracing::instrument(name = "Create new time entry via Teamdeck API", skip(self), err)]
    pub async fn add_time_entry(
        &self,
        body: CreateTimeEntryBody,
    ) -> Result<TimeEntryModel, TeamdeckApiError> {
        let response = self
            .post("https://api.teamdeck.io/v1/time-entries")
            .json(&body)
            .send()
            .await?;

        let response_body = response.text().await?;
        dbg!(&response_body);
        let time_entry = serde_json::from_str(&response_body)
            .map_err(|e| TeamdeckApiError::ServerError(e.to_string()))?;
        Ok(time_entry)
    }

    fn put<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .put(url)
            .header(API_KEY_HEADER_NAME, &self.api_key)
    }

    fn post<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .post(url)
            .header(API_KEY_HEADER_NAME, &self.api_key)
    }
}
