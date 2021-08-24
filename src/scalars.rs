use async_graphql::{InputValueError, InputValueResult, ScalarType, Value, Scalar};
use chrono::{DateTime, NaiveDateTime, Utc};

/// DateTime RFC3339
pub struct Date(pub DateTime<Utc>);

#[Scalar]
impl ScalarType for Date {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(Date(DateTime::from(DateTime::parse_from_rfc3339(
                value.as_str(),
            )?)))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}
