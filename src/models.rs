use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use surrealdb::sql::Thing;
fn serialize_thing_as_string<S>(thing: &Thing, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&thing.id.to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shift {
    #[serde(serialize_with = "serialize_thing_as_string")]
    pub id: Thing,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub hours_worked: Option<Decimal>,
    pub odometer_start: i32,
    pub odometer_end: Option<i32>,
    pub miles_driven: Option<i32>,
    pub earnings: Decimal,
    pub tips: Decimal,
    pub gas_cost: Decimal,
    pub day_total: Decimal,
    pub hourly_pay: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ShiftRecord {
    pub start_time: surrealdb::sql::Datetime,
    pub end_time: Option<surrealdb::sql::Datetime>,
    pub hours_worked: Option<Decimal>,
    pub odometer_start: i32,
    pub odometer_end: Option<i32>,
    pub miles_driven: Option<i32>,
    pub earnings: Decimal,
    pub tips: Decimal,
    pub gas_cost: Decimal,
    pub day_total: Decimal,
    pub hourly_pay: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct ShiftUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<surrealdb::sql::Datetime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odometer_start: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odometer_end: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miles_driven: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours_worked: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tips: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_cost: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_total: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hourly_pay: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartShiftRequest {
    pub odometer_start: i32,
}

#[derive(Debug, Deserialize)]
pub struct EndShiftRequest {
    pub odometer_end: i32,
    pub earnings: Option<Decimal>,
    pub tips: Option<Decimal>,
    pub gas_cost: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftRequest {
    pub odometer_start: Option<i32>,
    pub odometer_end: Option<i32>,
    pub earnings: Option<Decimal>,
    pub tips: Option<Decimal>,
    pub gas_cost: Option<Decimal>,
    pub notes: Option<String>,
}
