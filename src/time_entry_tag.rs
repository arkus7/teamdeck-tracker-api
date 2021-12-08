use crate::teamdeck::api::TeamdeckApiClient;
use async_graphql::{Context, Object, Result, ResultExt, SimpleObject};
use serde::{Deserialize, Serialize, Deserializer, de::Unexpected};

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
pub struct TimeEntryTag {
    id: u64,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    #[serde(deserialize_with = "bool_from_int")]
    archived: bool,
}

#[derive(Default, Debug)]
pub struct TimeEntryTagQuery;

#[Object]
impl TimeEntryTagQuery {
    #[tracing::instrument(name = "Fetching time entry tag by id", skip(ctx))]
    async fn time_entry_tag(&self, ctx: &Context<'_>, tag_id: u64) -> Result<Option<TimeEntryTag>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let tag: Option<TimeEntryTag> = client
        .get_time_entry_tag(tag_id)
        .await
        .extend()?;
        Ok(tag)
    }

    #[tracing::instrument(name = "Fetching all time entry tags", skip(ctx))]
    async fn time_entry_tags(&self, ctx: &Context<'_>) -> Result<Vec<TimeEntryTag>> {
        let client = ctx.data_unchecked::<TeamdeckApiClient>();
        let tags: Vec<TimeEntryTag> = client.get_time_entry_tags().await.extend()?;
        Ok(tags)
    }
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}
