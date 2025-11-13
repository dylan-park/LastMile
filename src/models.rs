use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use surrealdb::sql::Thing;

fn serialize_thing_as_string<S>(thing: &Thing, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&thing.id.to_string())
}

// Custom deserializer to handle both int and decimal from database
fn deserialize_flexible_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexibleDecimal {
        Int(i64),
        Decimal(Decimal),
    }

    match FlexibleDecimal::deserialize(deserializer)? {
        FlexibleDecimal::Int(i) => Ok(Decimal::from(i)),
        FlexibleDecimal::Decimal(d) => Ok(d),
    }
}

fn deserialize_optional_flexible_decimal<'de, D>(
    deserializer: D,
) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexibleDecimal {
        Int(i64),
        Decimal(Decimal),
    }

    let opt: Option<FlexibleDecimal> = Option::deserialize(deserializer)?;
    match opt {
        Some(FlexibleDecimal::Int(i)) => Ok(Some(Decimal::from(i))),
        Some(FlexibleDecimal::Decimal(d)) => Ok(Some(d)),
        None => Ok(None),
    }
}

fn deserialize_optional_field<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    Ok(Some(Option::<String>::deserialize(deserializer)?))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shift {
    #[serde(serialize_with = "serialize_thing_as_string")]
    pub id: Thing,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_optional_flexible_decimal")]
    pub hours_worked: Option<Decimal>,
    pub odometer_start: i32,
    pub odometer_end: Option<i32>,
    pub miles_driven: Option<i32>,
    #[serde(deserialize_with = "deserialize_flexible_decimal")]
    pub earnings: Decimal,
    #[serde(deserialize_with = "deserialize_flexible_decimal")]
    pub tips: Decimal,
    #[serde(deserialize_with = "deserialize_flexible_decimal")]
    pub gas_cost: Decimal,
    #[serde(deserialize_with = "deserialize_flexible_decimal")]
    pub day_total: Decimal,
    #[serde(default, deserialize_with = "deserialize_optional_flexible_decimal")]
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
    // Don't skip serializing notes - we want to allow explicitly setting it to None
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
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaintenanceItem {
    #[serde(serialize_with = "serialize_thing_as_string")]
    pub id: Thing,
    pub name: String,
    pub mileage_interval: i32,
    pub last_service_mileage: i32,
    pub enabled: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MaintenanceItemRecord {
    pub name: String,
    pub mileage_interval: i32,
    pub last_service_mileage: i32,
    pub enabled: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct MaintenanceItemUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mileage_interval: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_service_mileage: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    // Don't skip serializing notes - we want to allow explicitly setting it to None
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMaintenanceItemRequest {
    pub name: String,
    pub mileage_interval: i32,
    pub last_service_mileage: Option<i32>,
    pub enabled: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMaintenanceItemRequest {
    pub name: Option<String>,
    pub mileage_interval: Option<i32>,
    pub last_service_mileage: Option<i32>,
    pub enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequiredMaintenanceResponse {
    pub required_maintenance_items: Vec<MaintenanceItem>,
}
