use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Shift {
    pub id: i32,
    pub start_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub hours_worked: Option<BigDecimal>,
    pub odometer_start: i32,
    pub odometer_end: Option<i32>,
    pub miles_driven: Option<i32>,
    pub earnings: BigDecimal,
    pub tips: BigDecimal,
    pub gas_cost: BigDecimal,
    pub day_total: BigDecimal,
    pub hourly_pay: Option<BigDecimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartShiftRequest {
    pub odometer_start: i32,
}

#[derive(Debug, Deserialize)]
pub struct EndShiftRequest {
    pub odometer_end: i32,
    pub earnings: Option<BigDecimal>,
    pub tips: Option<BigDecimal>,
    pub gas_cost: Option<BigDecimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftRequest {
    pub odometer_start: Option<i32>,
    pub odometer_end: Option<i32>,
    pub earnings: Option<BigDecimal>,
    pub tips: Option<BigDecimal>,
    pub gas_cost: Option<BigDecimal>,
    pub notes: Option<String>,
}
