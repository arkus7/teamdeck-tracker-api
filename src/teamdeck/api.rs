use crate::project::Project;
use crate::resource::Resource;
use crate::teamdeck::error::TeamdeckApiError;
use crate::time_entry::TimeEntry;
use chrono::{DateTime, Utc, NaiveDate};
use reqwest;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::IntoUrl;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::collections::HashMap;
use crate::scalars::DATE_FORMAT;

const API_KEY_ENV_VARIABLE: &str = "TEAMDECK_API_KEY";
const API_KEY_HEADER_NAME: &str = "X-Api-Key";

pub struct TeamdeckApiClient {
    api_key: String,
}

impl TeamdeckApiClient {
    fn from_env() -> Self {
        Self {
            api_key: std::env::var(API_KEY_ENV_VARIABLE)
                .expect(format!("Missing {} env variable", API_KEY_ENV_VARIABLE).as_str()),
        }
    }
}

impl Default for TeamdeckApiClient {
    fn default() -> Self {
        TeamdeckApiClient::from_env()
    }
}

enum PaginationHeader {
    TotalCount,
    PagesCount,
    CurrentPage,
    ItemsPerPage,
}

impl Display for PaginationHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Into<HeaderName> for PaginationHeader {
    fn into(self) -> HeaderName {
        match self {
            PaginationHeader::TotalCount => HeaderName::from_static("x-pagination-total-count"),
            PaginationHeader::CurrentPage => HeaderName::from_static("x-pagination-current-page"),
            PaginationHeader::ItemsPerPage => HeaderName::from_static("x-pagination-per-page"),
            PaginationHeader::PagesCount => HeaderName::from_static("x-pagination-page-count"),
        }
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

impl TeamdeckApiClient {
    #[tracing::instrument(name = "Fetching resource by ID from Teamdeck API", skip(self), err)]
    pub async fn get_resource_by_id(&self, resource_id: u64) -> Result<Option<Resource>, TeamdeckApiError> {
        let resource = self
            .get(format!("https://api.teamdeck.io/v1/resources/{}", resource_id).as_str())
            .send()
            .await?
            .json()
            .await?;

        Ok(resource)
    }

    #[tracing::instrument(name = "Fetching resource page from Teamdeck API", skip(self), err)]
    pub async fn get_resources_page(
        &self,
        page: Option<u64>,
    ) -> Result<Page<Resource>, TeamdeckApiError> {
        let response = self
            .get("https://api.teamdeck.io/v1/resources")
            .query(&[("page", page.unwrap_or(1))])
            .send()
            .await?;
        tracing::debug!("Response: {:?}", response);
        let headers = response.headers().clone();
        let pagination = TeamdeckApiClient::read_pagination_info(&headers)?;
        let resources = response.json().await?;

        Ok(Page {
            items: resources,
            pagination,
        })
    }

    #[tracing::instrument(name = "Fetching all resources from Teamdeck API", skip(self), err)]
    pub async fn get_resources(&self) -> Result<Vec<Resource>, TeamdeckApiError> {
        self.traverse_all_pages(|page| self.get_resources_page(page))
            .await
    }

    #[tracing::instrument(name = "Fetching projects page from Teamdeck API", skip(self), err)]
    pub async fn get_projects_page(
        &self,
        page: Option<u64>,
    ) -> Result<Page<Project>, TeamdeckApiError> {
        let response = self
            .get("https://api.teamdeck.io/v1/projects")
            .query(&[("page", page.unwrap_or(1))])
            .send()
            .await?;

        let headers = response.headers().clone();
        let pagination = TeamdeckApiClient::read_pagination_info(&headers)?;

        let projects = response.json().await?;

        Ok(Page {
            items: projects,
            pagination,
        })
    }

    #[tracing::instrument(name = "Fetching all projects from Teamdeck API", skip(self), err)]
    pub async fn get_projects(&self) -> Result<Vec<Project>, TeamdeckApiError> {
        self.traverse_all_pages(|page| self.get_projects_page(page))
            .await
    }

    #[tracing::instrument(
        name = "Fetching project by ID from Teamdeck API",
        skip(self),
        err
    )]
    pub async fn get_project_by_id(&self, project_id: u64) -> Result<Option<Project>, TeamdeckApiError> {
        let project = self.get(format!("https://api.teamdeck.io/v1/projects/{}", project_id).as_str())
            .send()
            .await?
            .json()
            .await?;
        Ok(Some(project))
    }

    #[tracing::instrument(
        name = "Fetching all time entries page from Teamdeck API",
        skip(self),
        err
    )]
    pub async fn get_time_entries(
        &self,
        resource_id: u64,
        date: Option<DateTime<Utc>>,
    ) -> Result<Vec<TimeEntry>, TeamdeckApiError> {
        self.traverse_all_pages(|page| self.get_time_entries_page(resource_id, date, page))
            .await
    }

    #[tracing::instrument(name = "Fetching time entries page from Teamdeck API", skip(self), err)]
    pub async fn get_time_entries_page(
        &self,
        resource_id: u64,
        date: Option<DateTime<Utc>>,
        page: Option<u64>,
    ) -> Result<Page<TimeEntry>, TeamdeckApiError> {
        let response = self
            .get("https://api.teamdeck.io/v1/time-entries")
            .query(&[
                ("resource_id", resource_id.to_string()),
                ("page", page.unwrap_or(1).to_string()),
                ("expand", "tags".to_string()),
            ])
            .send()
            .await?;

        let headers = response.headers();
        let pagination = TeamdeckApiClient::read_pagination_info(&headers)?;

        let time_entries = response.json().await?;

        Ok(Page {
            items: time_entries,
            pagination,
        })
    }

    fn get<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .get(url)
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
        let header_value = headers
            .get(&header)
            .ok_or(TeamdeckApiError::ServerError(format!(
                "Missing {} header value in response",
                &header
            )));

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

        while current_page != total_pages {
            current_page = current_page + 1;
            let page = f(Some(current_page)).await?;
            items.extend(page.items);
            total_pages = page.pagination.pages_count;
        }

        Ok(items)
    }
}
