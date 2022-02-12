use async_graphql::{ErrorExtensions, FieldError};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error, Deserialize)]
pub enum TeamdeckApiError {
    #[error("Could not find {} resource (ID: {})", .resource_type, .resource_id)]
    NotFound {
        resource_type: String,
        resource_id: u64,
    },

    #[error("ServerError")]
    ServerError(String),
}

impl From<reqwest::Error> for TeamdeckApiError {
    fn from(error: reqwest::Error) -> Self {
        if let Some(status) = error.status() {
            match status.as_u16() {
                404 => TeamdeckApiError::NotFound {
                    resource_type: "unknown".to_string(),
                    resource_id: 0,
                },
                _ => TeamdeckApiError::ServerError(status.as_str().to_string()),
            }
        } else {
            TeamdeckApiError::ServerError(error.to_string())
        }
    }
}

impl ErrorExtensions for TeamdeckApiError {
    fn extend(&self) -> FieldError {
        self.extend_with(|err, e| match err {
            TeamdeckApiError::NotFound {
                resource_type,
                resource_id,
            } => {
                e.set("code", "NOT_FOUND");
                e.set("resource_type", resource_type.as_str());
                e.set("resource_id", *resource_id);
            },
            TeamdeckApiError::ServerError(reason) => e.set("reason", reason.to_string()),
        })
    }
}
