use crate::calculations::calculate_remaining_mileage;
use crate::db::setup_database;
use crate::models::{MaintenanceItem, MaintenanceItemRecord, Shift, ShiftRecord};

use chrono::{Duration, Utc};
use rand::RngExt;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use surrealdb::{Surreal, engine::local::Db};

// Constants
const STARTING_ODOMETER: i32 = 163000;
const SEED_DAYS: i64 = 90;
const SHIFT_PROBABILITY: f64 = 0.7;

// Helper struct for random shift data
struct ShiftData {
    start_hour: u32,
    start_minute: u32,
    hours_worked_float: f64,
    miles_driven: i32,
    earnings_float: f64,
    tips_float: f64,
    gas_per_mile: f64,
}

impl ShiftData {
    /// Generate random shift data
    fn random() -> Option<Self> {
        let mut rng = rand::rng();

        if !rng.random_bool(SHIFT_PROBABILITY) {
            return None;
        }

        Some(Self {
            start_hour: rng.random_range(7..15),
            start_minute: rng.random_range(0..60),
            hours_worked_float: rng.random_range(7.0..8.75),
            miles_driven: rng.random_range(80..161),
            earnings_float: rng.random_range(30.0..60.0),
            tips_float: rng.random_range(35.0..85.0),
            gas_per_mile: rng.random_range(0.08..0.15),
        })
    }

    /// Convert to ShiftRecord and return new odometer reading
    fn to_shift_record(
        &self,
        current_date: chrono::DateTime<Utc>,
        current_odometer: i32,
    ) -> (ShiftRecord, i32) {
        let start_time = current_date
            .date_naive()
            .and_hms_opt(self.start_hour, self.start_minute, 0)
            .expect("Invalid time values")
            .and_local_timezone(Utc)
            .unwrap();

        let hours_worked = Decimal::from_f64(self.hours_worked_float)
            .expect("Invalid hours_worked float")
            .round_dp(2);

        let duration_seconds = (self.hours_worked_float * 3600.0) as i64;
        let end_time = start_time + Duration::seconds(duration_seconds);

        let odometer_start = current_odometer;
        let odometer_end = odometer_start + self.miles_driven;
        let gas_cost_float = self.miles_driven as f64 * self.gas_per_mile;

        let earnings = Decimal::from_f64(self.earnings_float)
            .expect("Invalid earnings float")
            .round_dp(2);
        let tips = Decimal::from_f64(self.tips_float)
            .expect("Invalid tips float")
            .round_dp(2);
        let gas_cost = Decimal::from_f64(gas_cost_float)
            .expect("Invalid gas_cost float")
            .round_dp(2);

        let day_total = (earnings + tips - gas_cost).round_dp(2);
        let hourly_pay = (day_total / hours_worked).round_dp(2);

        let record = ShiftRecord {
            start_time: start_time.into(),
            end_time: Some(end_time.into()),
            hours_worked: Some(hours_worked),
            odometer_start,
            odometer_end: Some(odometer_end),
            miles_driven: Some(self.miles_driven),
            earnings,
            tips,
            gas_cost,
            day_total,
            hourly_pay: Some(hourly_pay),
            notes: None,
        };

        (record, odometer_end)
    }
}

// Maintenance item configuration
struct MaintenanceConfig {
    name: &'static str,
    interval: i32,
    max_random_offset: i32,
    notes: Option<&'static str>,
}

impl MaintenanceConfig {
    fn create_record(
        &self,
        current_mileage: i32,
        last_service_offset: i32,
    ) -> MaintenanceItemRecord {
        let last_service_mileage = current_mileage - last_service_offset;

        MaintenanceItemRecord {
            name: self.name.to_string(),
            mileage_interval: self.interval,
            last_service_mileage,
            remaining_mileage: calculate_remaining_mileage(
                current_mileage,
                last_service_mileage,
                self.interval,
            ),
            enabled: true,
            notes: self.notes.map(|s| s.to_string()),
        }
    }
}

pub async fn seed_demo_data(db: &Surreal<Db>) -> surrealdb::Result<()> {
    setup_database(db).await;

    let mut current_odometer = STARTING_ODOMETER;

    // Seed shifts
    current_odometer = seed_shifts(db, current_odometer).await?;

    // Seed maintenance items
    seed_maintenance_items(db, current_odometer).await?;

    Ok(())
}

async fn seed_shifts(db: &Surreal<Db>, starting_odometer: i32) -> surrealdb::Result<i32> {
    let mut current_odometer = starting_odometer;
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(SEED_DAYS);
    let mut current_date = start_date;

    while current_date <= end_date {
        if let Some(shift_data) = ShiftData::random() {
            let (record, new_odometer) = shift_data.to_shift_record(current_date, current_odometer);

            let _: Option<Shift> = db.create("shifts").content(record).await?;
            current_odometer = new_odometer;
        }

        current_date += Duration::days(1);
    }

    Ok(current_odometer)
}

async fn seed_maintenance_items(db: &Surreal<Db>, current_mileage: i32) -> surrealdb::Result<()> {
    let configs = [
        MaintenanceConfig {
            name: "Oil Change",
            interval: 3000,
            max_random_offset: 2900,
            notes: Some("Synthetic"),
        },
        MaintenanceConfig {
            name: "Tire Rotation",
            interval: 5000,
            max_random_offset: 4900,
            notes: None,
        },
        MaintenanceConfig {
            name: "Brake Inspection",
            interval: 10000,
            max_random_offset: 9900,
            notes: None,
        },
    ];

    // Generate all random offsets at once
    let offsets: Vec<i32> = {
        let mut rng = rand::rng();
        configs
            .iter()
            .map(|config| rng.random_range(100..config.max_random_offset))
            .collect()
    };

    // Create all maintenance items
    for (config, offset) in configs.iter().zip(offsets.iter()) {
        let record = config.create_record(current_mileage, *offset);
        let _: Option<MaintenanceItem> = db.create("maintenance").content(record).await?;
    }

    Ok(())
}
