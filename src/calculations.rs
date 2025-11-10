use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

pub fn calculate_miles(odometer_start: i32, odometer_end: i32) -> i32 {
    odometer_end - odometer_start
}

pub fn calculate_hours(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Decimal {
    let duration = end_time.signed_duration_since(start_time);
    let seconds = Decimal::from(duration.num_seconds());
    let hours = seconds / Decimal::from(3600);
    hours.round_dp(2)
}

pub fn calculate_day_total(earnings: &Decimal, tips: &Decimal, gas_cost: &Decimal) -> Decimal {
    (earnings + tips - gas_cost).round_dp(2)
}

pub fn calculate_hourly_pay(day_total: &Decimal, hours_worked: &Decimal) -> Option<Decimal> {
    if hours_worked > &Decimal::ZERO {
        Some((day_total / hours_worked).round_dp(2))
    } else {
        None
    }
}

pub fn calculate_is_maintenance_required(
    latest_mileage: i32,
    last_service_mileage: i32,
    mileage_interval: i32,
) -> bool {
    latest_mileage >= (last_service_mileage + mileage_interval)
}

// Helper function to ensure decimal values from user input are properly normalized with 2 decimal places
pub fn normalize_decimal(value: Decimal) -> Decimal {
    value.round_dp(2)
}
