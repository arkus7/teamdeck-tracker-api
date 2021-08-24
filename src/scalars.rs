use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use chrono::{DateTime, Utc};

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

impl Clone for Date {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
