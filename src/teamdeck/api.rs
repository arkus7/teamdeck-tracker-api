use crate::scalars::Date;
use crate::teamdeck::error::TeamdeckApiError;
use crate::time_entry::{CreateTimeEntryInput, TimeEntryModel};
use crate::time_entry_tag::TimeEntryTag;
use chrono::{NaiveDate, Utc};
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::IntoUrl;
use reqwest::{self};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

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

#[derive(Debug, Clone, Copy)]
enum PaginationHeader {
    TotalCount,
    PagesCount,
    CurrentPage,
    ItemsPerPage,
}

impl PaginationHeader {
    fn as_str(&self) -> &'static str {
        match self {
            PaginationHeader::TotalCount => "x-pagination-total-count",
            PaginationHeader::PagesCount => "x-pagination-page-count",
            PaginationHeader::CurrentPage => "x-pagination-current-page",
            PaginationHeader::ItemsPerPage => "x-pagination-per-page",
        }
    }
}

impl Display for PaginationHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<PaginationHeader> for HeaderName {
    fn from(header: PaginationHeader) -> Self {
        HeaderName::from_static(header.as_str())
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

    #[tracing::instrument(
        name = "Fetching time entry tags page from Teamdeck API",
        skip(self),
        err
    )]
    pub async fn get_time_entry_tags_page(
        &self,
        page: Option<u64>,
    ) -> Result<Page<TimeEntryTag>, TeamdeckApiError> {
        let mut params = HashMap::new();
        params.insert("page", page.unwrap_or(1).to_string());

        let response = self
            .get("https://api.teamdeck.io/v1/time-entry-tags")
            .query(&params)
            .send()
            .await?;

        let headers = response.headers();
        let pagination = TeamdeckApiClient::read_pagination_info(headers)?;

        let time_entries = response.json().await?;

        Ok(Page {
            items: time_entries,
            pagination,
        })
    }

    #[tracing::instrument(
        name = "Fetching all time entry tags from Teamdeck API",
        skip(self),
        err
    )]
    pub async fn get_time_entry_tags(&self) -> Result<Vec<TimeEntryTag>, TeamdeckApiError> {
        self.traverse_all_pages(|page| self.get_time_entry_tags_page(page))
            .await
    }

    #[tracing::instrument(
        name = "Fetching time entry tag by ID from Teamdeck API",
        skip(self),
        err
    )]
    pub async fn get_time_entry_tag(
        &self,
        tag_id: u64,
    ) -> Result<Option<TimeEntryTag>, TeamdeckApiError> {
        let tag = self
            .get(format!("https://api.teamdeck.io/v1/time-entry-tags/{}", tag_id).as_str())
            .send()
            .await?
            .json()
            .await?;

        Ok(Some(tag))
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

    fn get<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .get(url)
            .header(API_KEY_HEADER_NAME, &self.api_key)
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

    fn read_pagination_info(headers: &HeaderMap) -> Result<PaginationInfo, TeamdeckApiError> {
        let pages_count = TeamdeckApiClient::get_pagination_header_value(
            headers,
            PaginationHeader::PagesCount.into(),
        )?;
        let total_count = TeamdeckApiClient::get_pagination_header_value(
            headers,
            PaginationHeader::TotalCount.into(),
        )?;
        let current_page = TeamdeckApiClient::get_pagination_header_value(
            headers,
            PaginationHeader::CurrentPage.into(),
        )?;
        let items_per_page = TeamdeckApiClient::get_pagination_header_value(
            headers,
            PaginationHeader::ItemsPerPage.into(),
        )?;

        Ok(PaginationInfo {
            total_count,
            pages_count,
            current_page,
            items_per_page,
        })
    }

    fn get_pagination_header_value(
        headers: &HeaderMap,
        header: HeaderName,
    ) -> Result<u64, TeamdeckApiError> {
        let header_value = headers.get(&header).ok_or_else(|| {
            TeamdeckApiError::ServerError(format!("Missing {} header value in response", &header))
        });

        let string_val = header_value?
            .to_str()
            .map_err(|e| TeamdeckApiError::ServerError(e.to_string()))?;
        string_val
            .parse::<u64>()
            .map_err(|e| TeamdeckApiError::ServerError(e.to_string()))
    }

    #[tracing::instrument(
        name = "Traverse all pages",
        skip(self, f),
        level = "debug"
        err
    )]
    async fn traverse_all_pages<F, ResultFuture, PageItem>(
        &self,
        f: F,
    ) -> Result<Vec<PageItem>, TeamdeckApiError>
    where
        F: Copy + FnOnce(Option<u64>) -> ResultFuture,
        ResultFuture: Future<Output = Result<Page<PageItem>, TeamdeckApiError>>,
        PageItem: Serialize + Debug,
    {
        let mut items: Vec<PageItem> = vec![];
        let mut current_page = 0;
        let mut total_pages: u64 = 1;

        while current_page != total_pages && total_pages != 0 {
            current_page += 1;
            let page = f(Some(current_page)).await?;
            items.extend(page.items);
            total_pages = page.pagination.pages_count;
        }

        Ok(items)
    }
}
