use async_graphql::{Context, Object, Result, SimpleObject};
use serde::{de::Unexpected, Deserialize, Deserializer, Serialize};
use teamdeck::{
    api::{
        paged,
        time_entries::{TimeEntryTag, TimeEntryTags},
        AsyncQuery, Pagination,
    },
    AsyncTeamdeck,
};

#[derive(Serialize, Deserialize, SimpleObject, Debug)]
pub struct TimeEntryTagModel {
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
    async fn time_entry_tag(
        &self,
        ctx: &Context<'_>,
        tag_id: u64,
    ) -> Result<Option<TimeEntryTagModel>> {
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = TimeEntryTag::builder().id(tag_id as usize).build()?;

        let tag = endpoint.query_async(client).await?;
        Ok(tag)
    }

    #[tracing::instrument(name = "Fetching all time entry tags", skip(ctx))]
    async fn time_entry_tags(&self, ctx: &Context<'_>) -> Result<Vec<TimeEntryTagModel>> {
        let client = ctx.data_unchecked::<AsyncTeamdeck>();
        let endpoint = TimeEntryTags::builder().build()?;

        let tags = paged(endpoint, Pagination::All).query_async(client).await?;
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
