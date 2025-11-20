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
    pub remaining_mileage: i32,
    pub enabled: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MaintenanceItemRecord {
    pub name: String,
    pub mileage_interval: i32,
    pub last_service_mileage: i32,
    pub remaining_mileage: i32,
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
    pub remaining_mileage: Option<i32>,
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

#[cfg(test)]

mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::json;

    #[test]
    fn test_deserialize_flexible_decimal_from_int() {
        let json_data = json!({
            "earnings": 100,
            "tips": 20,
            "gas_cost": 15,
            "day_total": 105
        });

        #[derive(serde::Deserialize)]
        struct TestData {
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            earnings: rust_decimal::Decimal,
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            tips: rust_decimal::Decimal,
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            gas_cost: rust_decimal::Decimal,
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            day_total: rust_decimal::Decimal,
        }

        let data: TestData = serde_json::from_value(json_data).unwrap();
        assert_eq!(data.earnings, dec!(100));
        assert_eq!(data.tips, dec!(20));
        assert_eq!(data.gas_cost, dec!(15));
        assert_eq!(data.day_total, dec!(105));
    }

    #[test]
    fn test_deserialize_flexible_decimal_from_decimal() {
        let json_data = json!({
            "earnings": 100.50,
            "tips": 20.25,
            "gas_cost": 15.75,
            "day_total": 105.00
        });

        #[derive(serde::Deserialize)]
        struct TestData {
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            earnings: rust_decimal::Decimal,
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            tips: rust_decimal::Decimal,
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            gas_cost: rust_decimal::Decimal,
            #[serde(deserialize_with = "deserialize_flexible_decimal")]
            day_total: rust_decimal::Decimal,
        }

        let data: TestData = serde_json::from_value(json_data).unwrap();
        assert_eq!(data.earnings, dec!(100.50));
        assert_eq!(data.tips, dec!(20.25));
        assert_eq!(data.gas_cost, dec!(15.75));
        assert_eq!(data.day_total, dec!(105.00));
    }

    #[test]
    fn test_deserialize_optional_flexible_decimal_some_int() {
        let json_data = json!({
            "hourly_pay": 15
        });

        #[derive(serde::Deserialize)]
        struct TestData {
            #[serde(default, deserialize_with = "deserialize_optional_flexible_decimal")]
            hourly_pay: Option<rust_decimal::Decimal>,
        }

        let data: TestData = serde_json::from_value(json_data).unwrap();
        assert_eq!(data.hourly_pay, Some(dec!(15)));
    }

    #[test]
    fn test_deserialize_optional_flexible_decimal_some_decimal() {
        let json_data = json!({
            "hourly_pay": 15.50
        });

        #[derive(serde::Deserialize)]
        struct TestData {
            #[serde(default, deserialize_with = "deserialize_optional_flexible_decimal")]
            hourly_pay: Option<rust_decimal::Decimal>,
        }

        let data: TestData = serde_json::from_value(json_data).unwrap();
        assert_eq!(data.hourly_pay, Some(dec!(15.50)));
    }

    #[test]
    fn test_deserialize_optional_flexible_decimal_none() {
        let json_data = json!({});

        #[derive(serde::Deserialize)]
        struct TestData {
            #[serde(default, deserialize_with = "deserialize_optional_flexible_decimal")]
            hourly_pay: Option<rust_decimal::Decimal>,
        }

        let data: TestData = serde_json::from_value(json_data).unwrap();
        assert_eq!(data.hourly_pay, None);
    }

    #[test]
    fn test_start_shift_request_deserialization() {
        let json_data = json!({
            "odometer_start": 12345
        });

        let request: StartShiftRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(request.odometer_start, 12345);
    }

    #[test]
    fn test_end_shift_request_deserialization_all_fields() {
        let json_data = json!({
            "odometer_end": 12445,
            "earnings": 100.50,
            "tips": 20.25,
            "gas_cost": 15.00,
            "notes": "Good shift"
        });

        let request: EndShiftRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(request.odometer_end, 12445);
        assert_eq!(request.earnings, Some(dec!(100.50)));
        assert_eq!(request.tips, Some(dec!(20.25)));
        assert_eq!(request.gas_cost, Some(dec!(15.00)));
        assert_eq!(request.notes, Some("Good shift".to_string()));
    }

    #[test]
    fn test_end_shift_request_deserialization_minimal() {
        let json_data = json!({
            "odometer_end": 12445
        });

        let request: EndShiftRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(request.odometer_end, 12445);
        assert_eq!(request.earnings, None);
        assert_eq!(request.tips, None);
        assert_eq!(request.gas_cost, None);
        assert_eq!(request.notes, None);
    }

    #[test]
    fn test_update_shift_request_deserialization() {
        let json_data = json!({
            "odometer_start": 12345,
            "odometer_end": 12445,
            "earnings": 100.00,
            "tips": 20.00,
            "gas_cost": 15.00,
            "notes": "Updated notes"
        });

        let request: UpdateShiftRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(request.odometer_start, Some(12345));
        assert_eq!(request.odometer_end, Some(12445));
        assert_eq!(request.earnings, Some(dec!(100.00)));
        assert_eq!(request.tips, Some(dec!(20.00)));
        assert_eq!(request.gas_cost, Some(dec!(15.00)));
        assert_eq!(request.notes, Some(Some("Updated notes".to_string())));
    }

    #[test]
    fn test_update_shift_request_clear_notes() {
        let json_data = json!({
            "notes": null
        });

        let request: UpdateShiftRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(request.notes, Some(None));
    }

    #[test]
    fn test_create_maintenance_item_request_deserialization() {
        let json_data = json!({
            "name": "Oil Change",
            "mileage_interval": 3000,
            "last_service_mileage": 10000,
            "enabled": true,
            "notes": "Full synthetic"
        });

        let request: CreateMaintenanceItemRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(request.name, "Oil Change");
        assert_eq!(request.mileage_interval, 3000);
        assert_eq!(request.last_service_mileage, Some(10000));
        assert_eq!(request.enabled, true);
        assert_eq!(request.notes, Some("Full synthetic".to_string()));
    }

    #[test]
    fn test_create_maintenance_item_request_minimal() {
        let json_data = json!({
            "name": "Tire Rotation",
            "mileage_interval": 5000,
            "enabled": true
        });

        let request: CreateMaintenanceItemRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(request.name, "Tire Rotation");
        assert_eq!(request.mileage_interval, 5000);
        assert_eq!(request.last_service_mileage, None);
        assert_eq!(request.enabled, true);
        assert_eq!(request.notes, None);
    }
}
