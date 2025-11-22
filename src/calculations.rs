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

pub fn calculate_remaining_mileage(
    latest_mileage: i32,
    last_service_mileage: i32,
    mileage_interval: i32,
) -> i32 {
    // If latest mileage is less than last service mileage, return full interval
    // This handles cases where no shifts exist yet or odometer was reset
    if latest_mileage < last_service_mileage {
        return mileage_interval;
    }

    let remaining = mileage_interval - (latest_mileage - last_service_mileage);
    remaining.max(0) // Clamp to 0, never go negative
}

// Helper function to ensure decimal values from user input are properly normalized with 2 decimal places
pub fn normalize_decimal(value: Decimal) -> Decimal {
    value.round_dp(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_miles() {
        assert_eq!(calculate_miles(100, 150), 50);
        assert_eq!(calculate_miles(0, 100), 100);
        assert_eq!(calculate_miles(1000, 1000), 0);
    }

    #[test]
    fn test_calculate_miles_large_values() {
        assert_eq!(calculate_miles(50000, 50250), 250);
    }

    #[test]
    fn test_calculate_hours() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 12, 30, 0).unwrap();

        let hours = calculate_hours(start, end);
        assert_eq!(hours, dec!(2.5));
    }

    #[test]
    fn test_calculate_hours_one_hour() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap();

        let hours = calculate_hours(start, end);
        assert_eq!(hours, dec!(1.0));
    }

    #[test]
    fn test_calculate_hours_fractional() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 10, 45, 0).unwrap();

        let hours = calculate_hours(start, end);
        assert_eq!(hours, dec!(0.75));
    }

    #[test]
    fn test_calculate_hours_rounding() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 1, 10, 10, 0).unwrap();

        let hours = calculate_hours(start, end);
        assert_eq!(hours, dec!(0.17));
    }

    #[test]
    fn test_calculate_day_total() {
        let earnings = dec!(100.0);
        let tips = dec!(20.0);
        let gas_cost = dec!(15.0);

        let total = calculate_day_total(&earnings, &tips, &gas_cost);
        assert_eq!(total, dec!(105.0));
    }

    #[test]
    fn test_calculate_day_total_zero_gas() {
        let earnings = dec!(100.0);
        let tips = dec!(20.0);
        let gas_cost = dec!(0.0);

        let total = calculate_day_total(&earnings, &tips, &gas_cost);
        assert_eq!(total, dec!(120.0));
    }

    #[test]
    fn test_calculate_day_total_high_gas_cost() {
        let earnings = dec!(100.0);
        let tips = dec!(20.0);
        let gas_cost = dec!(80.0);

        let total = calculate_day_total(&earnings, &tips, &gas_cost);
        assert_eq!(total, dec!(40.0));
    }

    #[test]
    fn test_calculate_day_total_exceeds_earnings() {
        let earnings = dec!(50.0);
        let tips = dec!(10.0);
        let gas_cost = dec!(80.0);

        let total = calculate_day_total(&earnings, &tips, &gas_cost);
        assert_eq!(total, dec!(-20.0));
    }

    #[test]
    fn test_calculate_hourly_pay() {
        let day_total = dec!(105.0);
        let hours_worked = dec!(7.0);

        let hourly = calculate_hourly_pay(&day_total, &hours_worked);
        assert_eq!(hourly, Some(dec!(15.0)));
    }

    #[test]
    fn test_calculate_hourly_pay_with_rounding() {
        let day_total = dec!(100.0);
        let hours_worked = dec!(7.0);

        let hourly = calculate_hourly_pay(&day_total, &hours_worked);
        assert_eq!(hourly, Some(dec!(14.29)));
    }

    #[test]
    fn test_calculate_hourly_pay_zero_hours() {
        let day_total = dec!(100.0);
        let hours_worked = dec!(0.0);

        let hourly = calculate_hourly_pay(&day_total, &hours_worked);
        assert_eq!(hourly, None);
    }

    #[test]
    fn test_calculate_hourly_pay_negative_total() {
        let day_total = dec!(-20.0);
        let hours_worked = dec!(5.0);

        let hourly = calculate_hourly_pay(&day_total, &hours_worked);
        assert_eq!(hourly, Some(dec!(-4.0)));
    }

    #[test]
    fn test_calculate_is_maintenance_required_true() {
        assert!(calculate_is_maintenance_required(5000, 2000, 3000));
        assert!(calculate_is_maintenance_required(5001, 2000, 3000));
        assert!(calculate_is_maintenance_required(10000, 5000, 5000));
    }

    #[test]
    fn test_calculate_is_maintenance_required_false() {
        assert!(!calculate_is_maintenance_required(4999, 2000, 3000));
        assert!(!calculate_is_maintenance_required(2000, 2000, 3000));
        assert!(!calculate_is_maintenance_required(0, 0, 1000));
    }

    #[test]
    fn test_calculate_is_maintenance_required_exact_interval() {
        assert!(calculate_is_maintenance_required(5000, 2000, 3000));
    }

    #[test]
    fn test_calculate_remaining_mileage_normal() {
        // Latest: 5000, Last service: 2000, Interval: 3000
        // Next service due at: 2000 + 3000 = 5000
        // Remaining: 3000 - (5000 - 2000) = 0
        assert_eq!(calculate_remaining_mileage(5000, 2000, 3000), 0);
    }

    #[test]
    fn test_calculate_remaining_mileage_with_miles_left() {
        // Latest: 4500, Last service: 2000, Interval: 3000
        // Next service due at: 2000 + 3000 = 5000
        // Remaining: 3000 - (4500 - 2000) = 500
        assert_eq!(calculate_remaining_mileage(4500, 2000, 3000), 500);
    }

    #[test]
    fn test_calculate_remaining_mileage_overdue() {
        // Latest: 6000, Last service: 2000, Interval: 3000
        // Next service due at: 2000 + 3000 = 5000
        // Remaining: 3000 - (6000 - 2000) = -1000, clamped to 0
        assert_eq!(calculate_remaining_mileage(6000, 2000, 3000), 0);
    }

    #[test]
    fn test_calculate_remaining_mileage_just_serviced() {
        // Latest: 2000, Last service: 2000, Interval: 3000
        // Remaining: 3000 - (2000 - 2000) = 3000
        assert_eq!(calculate_remaining_mileage(2000, 2000, 3000), 3000);
    }

    #[test]
    fn test_calculate_remaining_mileage_zero_latest() {
        // Latest: 0, Last service: 0, Interval: 1000
        // Remaining: 1000 - (0 - 0) = 1000
        assert_eq!(calculate_remaining_mileage(0, 0, 1000), 1000);
    }

    #[test]
    fn test_calculate_remaining_mileage_latest_less_than_last_service() {
        // Latest: 0, Last service: 10000, Interval: 3000
        // Should return full interval since latest < last_service
        assert_eq!(calculate_remaining_mileage(0, 10000, 3000), 3000);

        // Latest: 5000, Last service: 10000, Interval: 5000
        assert_eq!(calculate_remaining_mileage(5000, 10000, 5000), 5000);
    }

    #[test]
    fn test_normalize_decimal() {
        assert_eq!(normalize_decimal(dec!(10.123456)), dec!(10.12));
        assert_eq!(normalize_decimal(dec!(10.126)), dec!(10.13));
        assert_eq!(normalize_decimal(dec!(10.1)), dec!(10.1));
        assert_eq!(normalize_decimal(dec!(10)), dec!(10));
    }

    #[test]
    fn test_normalize_decimal_zero() {
        assert_eq!(normalize_decimal(dec!(0.0)), dec!(0.0));
    }

    #[test]
    fn test_normalize_decimal_negative() {
        assert_eq!(normalize_decimal(dec!(-10.123)), dec!(-10.12));
    }
}
