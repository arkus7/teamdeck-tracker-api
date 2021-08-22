use thiserror::Error;

#[derive(Debug, Error)]
pub enum TeamdeckApiError {
    #[error("Could not find resource")]
    NotFound,

    #[error("ServerError")]
    ServerError(String),

    #[error("No Extensions")]
    ErrorWithoutExtensions,
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
