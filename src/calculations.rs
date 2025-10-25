use bigdecimal::BigDecimal;
use chrono::{Duration, NaiveDateTime};

pub fn calculate_miles(odometer_start: i32, odometer_end: i32) -> i32 {
    odometer_end - odometer_start
}

pub fn calculate_hours(start_time: NaiveDateTime, end_time: NaiveDateTime) -> BigDecimal {
    let duration: Duration = end_time.signed_duration_since(start_time);
    BigDecimal::from(duration.num_seconds()) / BigDecimal::from(3600)
}

pub fn calculate_day_total(
    earnings: &BigDecimal,
    tips: &BigDecimal,
    gas_cost: &BigDecimal,
) -> BigDecimal {
    earnings + tips - gas_cost
}

pub fn calculate_hourly_pay(
    day_total: &BigDecimal,
    hours_worked: &BigDecimal,
) -> Option<BigDecimal> {
    if hours_worked > &BigDecimal::from(0) {
        Some(day_total / hours_worked)
    } else {
        None
    }
}
