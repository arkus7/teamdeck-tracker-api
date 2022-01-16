use async_graphql::{ErrorExtensions, FieldError};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error, Deserialize)]
pub enum TeamdeckApiError {
    #[error("Could not find resource")]
    NotFound,

    #[error("ServerError")]
    ServerError(String),
}

impl From<reqwest::Error> for TeamdeckApiError {
    fn from(error: reqwest::Error) -> Self {
        if let Some(status) = error.status() {
            match status.as_u16() {
                404 => TeamdeckApiError::NotFound,
                _ => TeamdeckApiError::ServerError(status.as_str().to_string()),
            }
        } else {
            TeamdeckApiError::ServerError(error.to_string())
        }
    }
}

impl ErrorExtensions for TeamdeckApiError {
    fn extend(self) -> FieldError {
        self.extend_with(|err, e| match err {
            TeamdeckApiError::NotFound => e.set("code", "NOT_FOUND"),
            TeamdeckApiError::ServerError(reason) => e.set("reason", reason.to_string()),
        })
    }
}
