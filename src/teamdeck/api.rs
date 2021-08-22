use crate::project::Project;
use crate::resource::Resource;
use crate::teamdeck::error::TeamdeckApiError;
use reqwest;
use reqwest::header::{HeaderMap, HeaderName, ACCEPT};
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::process::Output;

pub struct TeamdeckApiClient {
    api_key: String,
}

impl Default for TeamdeckApiClient {
    fn default() -> Self {
        TeamdeckApiClient {
            api_key: env!("TEAMDECK_API_KEY").to_string(),
        }
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
    pub async fn get_resource_by_id(&self, id: u64) -> reqwest::Result<Resource> {
        reqwest::Client::new()
            .get(format!("https://api.teamdeck.io/v1/resources/{}", id).as_str())
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json()
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
        // TODO: fix to use traverse_all_pages
        // let res: Result<Vec<Project>, TeamdeckApiError> = self.traverse_all_pages(Self::get_projects_page).await;
        let mut items: Vec<Project> = vec![];
        let mut current_page = 0;
        let mut total_pages: u64 = 1;

        while current_page != total_pages {
            current_page = current_page + 1;
            println!("total pages: {:?}, current page: {}", total_pages, current_page);
            let page = self.get_projects_page(Some(current_page)).await?;
            items.extend(page.items);
            total_pages = page.pagination.pages_count
        }

        Ok(items)
    }

    fn get<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .get(url)
            .header("X-API-KEY", &self.api_key)
    }

    fn read_pagination_info(headers: &HeaderMap) -> Result<PaginationInfo, TeamdeckApiError> {
        // let total_count = headers
        //     .get("x-pagination-total-count")
        //     .expect("x-pagination-total-count missing")
        //     .to_str()
        //     .expect("x-pagination-total-count is not string")
        //     .parse::<u64>()
        //     .unwrap_or(0);
        let pages_count = TeamdeckApiClient::get_pagination_header_value(headers, PaginationHeader::PagesCount.into())?;
        let total_count = TeamdeckApiClient::get_pagination_header_value(headers, PaginationHeader::TotalCount.into())?;
        let current_page = TeamdeckApiClient::get_pagination_header_value(headers, PaginationHeader::CurrentPage.into())?;
        let items_per_page = TeamdeckApiClient::get_pagination_header_value(headers, PaginationHeader::ItemsPerPage.into())?;

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

        let string_val = header_value?.to_str().map_err(|e| TeamdeckApiError::ServerError(e.to_string()))?;
        string_val.parse::<u64>().map_err(|e| TeamdeckApiError::ServerError(e.to_string()))
    }

    // TODO: Fix types, not working so far
    async fn traverse_all_pages<F: Copy, ResultFuture, PageItem>(&self, f: F) -> Result<Vec<PageItem>, TeamdeckApiError>
        where
            F: FnOnce(&Self, Option<u64>) -> ResultFuture,
            ResultFuture: Future<Output = Result<Page<PageItem>, TeamdeckApiError>>,
            PageItem: Serialize
    {
        let mut items: Vec<PageItem> = vec![];
        let mut current_page = 0;
        let mut total_pages: u64 = 1;

        while current_page != total_pages {
            current_page = current_page + 1;
            println!("total pages: {:?}, current page: {}", total_pages, current_page);
            let page = f(self, Some(current_page)).await?;
            items.extend(page.items);
            total_pages = page.pagination.pages_count
        }

        Ok(items)
    }
}
