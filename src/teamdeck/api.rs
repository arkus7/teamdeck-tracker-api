use crate::project::Project;
use crate::resource::Resource;
use crate::teamdeck::error::TeamdeckApiError;
use reqwest;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::IntoUrl;
use serde::{Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

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
    pub async fn get_resource_by_id(&self, id: u64) -> Result<Resource, TeamdeckApiError> {
        let response = self
            .get(format!("https://api.teamdeck.io/v1/resources/{}", id).as_str())
            .send()
            .await?;
        let resource = response.json().await?;

        Ok(resource)
    }

    pub async fn get_resources_page(
        &self,
        page: Option<u64>,
    ) -> Result<Page<Resource>, TeamdeckApiError> {
        let response = self
            .get("https://api.teamdeck.io/v1/resources")
            .query(&[("page", page.unwrap_or(1))])
            .send()
            .await?;
        let headers = response.headers().clone();
        let pagination = TeamdeckApiClient::read_pagination_info(&headers)?;
        let resources = response.json().await?;

        Ok(Page {
            items: resources,
            pagination,
        })
    }

    pub async fn get_resources(&self) -> Result<Vec<Resource>, TeamdeckApiError> {
        self.traverse_all_pages(|page| self.get_resources_page(page))
            .await
    }

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

    pub async fn get_projects(&self) -> Result<Vec<Project>, TeamdeckApiError> {
        self.traverse_all_pages(|page| self.get_projects_page(page))
            .await
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
            total_pages = page.pagination.pages_count
        }

        Ok(items)
    }
}
