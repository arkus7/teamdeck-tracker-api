use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use chrono::{DateTime as ChronoDateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

/// DateTime RFC3339
#[derive(Debug, Serialize, Deserialize)]
pub struct DateTime(pub ChronoDateTime<Utc>);

#[Scalar]
impl ScalarType for DateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(DateTime(ChronoDateTime::from(
                ChronoDateTime::parse_from_rfc3339(value.as_str())?,
            )))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}

impl Clone for DateTime {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

/// Date in YYYY-MM-DD format
#[derive(Debug, Serialize, Deserialize, Copy)]
pub struct Date(pub NaiveDate);

pub const DATE_FORMAT: &str = "%Y-%m-%d";

#[Scalar]
impl ScalarType for Date {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(Date(NaiveDate::parse_from_str(value, DATE_FORMAT)?))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.format(DATE_FORMAT).to_string())
    }
}

impl Clone for Date {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

/// Time in HH:MM format
#[derive(Debug, Serialize, Deserialize)]
pub struct Time(pub NaiveTime);

const TIME_FORMAT: &str = "%H:%M";

#[Scalar]
impl ScalarType for Time {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(Time(NaiveTime::parse_from_str(value, TIME_FORMAT)?))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.format(TIME_FORMAT).to_string())
    }
}

impl Clone for Time {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
