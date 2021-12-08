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
