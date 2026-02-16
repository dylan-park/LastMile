use crate::calculations::calculate_remaining_mileage;
use crate::db::setup_database;
use crate::models::{MaintenanceItem, MaintenanceItemRecord, Shift, ShiftRecord};

use chrono::{Duration, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use surrealdb::{Surreal, engine::local::Db};

pub async fn seed_demo_data(db: &Surreal<Db>) -> surrealdb::Result<()> {
    setup_database(db).await;

    // Generate Shifts
    let mut rng = StdRng::from_entropy();
    let mut current_odometer = 163000;

    // Generate dates (last 3 months)
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(90);
    let mut current_date = start_date;

    while current_date <= end_date {
        // 5 random days per week logic approximated: 70% chance of shift per day
        if rng.gen_bool(0.7) {
            // Random start time (07:00 - 15:00)
            let start_hour = rng.gen_range(7..15);
            let start_minute = rng.gen_range(0..60);
            let start_time = current_date
                .date_naive()
                .and_hms_opt(start_hour, start_minute, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap();

            // Random duration (7.0 - 8.75 hours)
            let hours_worked_float = rng.gen_range(7.0..8.75);
            let hours_worked = Decimal::from_f64(hours_worked_float).unwrap().round_dp(2);

            let duration_seconds = (hours_worked_float * 3600.0) as i64;
            let end_time = start_time + Duration::seconds(duration_seconds);

            // Miles driven (80 - 160)
            let miles_driven = rng.gen_range(80..161);
            let odometer_start = current_odometer;
            let odometer_end = odometer_start + miles_driven;

            // Financials
            let earnings_float = rng.gen_range(30.0..60.0);
            let tips_float = rng.gen_range(35.0..85.0);
            let gas_per_mile = rng.gen_range(0.08..0.15);
            let gas_cost_float = miles_driven as f64 * gas_per_mile;

            let earnings = Decimal::from_f64(earnings_float).unwrap().round_dp(2);
            let tips = Decimal::from_f64(tips_float).unwrap().round_dp(2);
            let gas_cost = Decimal::from_f64(gas_cost_float).unwrap().round_dp(2);
            let day_total = (earnings + tips - gas_cost).round_dp(2);
            let hourly_pay = (day_total / hours_worked).round_dp(2);

            let record = ShiftRecord {
                start_time: start_time.into(),
                end_time: Some(end_time.into()),
                hours_worked: Some(hours_worked),
                odometer_start,
                odometer_end: Some(odometer_end),
                miles_driven: Some(miles_driven),
                earnings,
                tips,
                gas_cost,
                day_total,
                hourly_pay: Some(hourly_pay),
                notes: None,
            };

            let _: Option<Shift> = db.create("shifts").content(record).await?;

            current_odometer = odometer_end;
        }
        current_date += Duration::days(1);
    }

    // Generate Maintenance Items
    let current_mileage = current_odometer;

    // Oil Change (every 3000 miles)
    let oil_last = current_mileage - rng.gen_range(100..2900);
    let oil_interval = 3000;
    let _: Option<MaintenanceItem> = db
        .create("maintenance")
        .content(MaintenanceItemRecord {
            name: "Oil Change".to_string(),
            mileage_interval: oil_interval,
            last_service_mileage: oil_last,
            remaining_mileage: calculate_remaining_mileage(current_mileage, oil_last, oil_interval),
            enabled: true,
            notes: Some("Synthetic".to_string()),
        })
        .await?;

    // Tire Rotation (every 5000 miles)
    let tire_last = current_mileage - rng.gen_range(100..4900);
    let tire_interval = 5000;
    let _: Option<MaintenanceItem> = db
        .create("maintenance")
        .content(MaintenanceItemRecord {
            name: "Tire Rotation".to_string(),
            mileage_interval: tire_interval,
            last_service_mileage: tire_last,
            remaining_mileage: calculate_remaining_mileage(
                current_mileage,
                tire_last,
                tire_interval,
            ),
            enabled: true,
            notes: None,
        })
        .await?;

    // Brake Inspection (every 10000 miles)
    let brake_last = current_mileage - rng.gen_range(100..9900);
    let brake_interval = 10000;
    let _: Option<MaintenanceItem> = db
        .create("maintenance")
        .content(MaintenanceItemRecord {
            name: "Brake Inspection".to_string(),
            mileage_interval: brake_interval,
            last_service_mileage: brake_last,
            remaining_mileage: calculate_remaining_mileage(
                current_mileage,
                brake_last,
                brake_interval,
            ),
            enabled: true,
            notes: None,
        })
        .await?;

    Ok(())
}
