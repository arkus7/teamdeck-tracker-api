use reqwest;
use crate::resource::{Resource};

pub struct TeamdeckApiClient {
    api_key: String
}

impl Default for TeamdeckApiClient {
    fn default() -> Self {
        TeamdeckApiClient {
            api_key: env!("TEAMDECK_API_KEY").to_string()
        }
    }
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
}