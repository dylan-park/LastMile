use surrealdb::{Surreal, engine::local::Db};
use tracing::info;

use crate::{
    calculations::calculate_remaining_mileage,
    error::{AppError, Result},
    models::{MaintenanceItem, MaintenanceItemUpdate, Shift},
};

// Helper to execute a query and extract shifts
pub async fn query_shifts(db: &Surreal<Db>, query: &str) -> Result<Vec<Shift>> {
    let mut result = db.query(query).await?;
    let shifts: Vec<Shift> = result.take(0)?;
    Ok(shifts)
}

// Helper to execute a parameterized query with two datetime binds and extract shifts
pub async fn query_shifts_with_date_range(
    db: &Surreal<Db>,
    query: &str,
    start: surrealdb::sql::Datetime,
    end: surrealdb::sql::Datetime,
) -> Result<Vec<Shift>> {
    let mut result = db
        .query(query)
        .bind(("start", start))
        .bind(("end", end))
        .await?;
    let shifts: Vec<Shift> = result.take(0)?;
    Ok(shifts)
}

pub async fn query_maintenance_items(
    db: &Surreal<Db>,
    query: &str,
) -> Result<Vec<MaintenanceItem>> {
    let mut result = db.query(query).await?;
    let maintenance_items: Vec<MaintenanceItem> = result.take(0)?;
    Ok(maintenance_items)
}

pub async fn has_active_shift(db: &Surreal<Db>) -> Result<bool> {
    let shifts = query_shifts(db, "SELECT * FROM shifts WHERE end_time = NONE LIMIT 1").await?;
    Ok(!shifts.is_empty())
}

pub async fn get_shift_by_id(db: &Surreal<Db>, id: &str) -> Result<Shift> {
    let shift: Option<Shift> = db.select(("shifts", id)).await?;
    shift.ok_or(AppError::ShiftNotFound)
}

pub async fn get_maintenance_item_by_id(db: &Surreal<Db>, id: &str) -> Result<MaintenanceItem> {
    let maintenance_item: Option<MaintenanceItem> = db.select(("maintenance", id)).await?;
    maintenance_item.ok_or(AppError::MaintenanceItemNotFound)
}

// Helper function to update all maintenance items' remaining mileage
// Called when shift odometer readings change
pub async fn update_all_maintenance_remaining_mileage(
    db: &Surreal<Db>,
    latest_mileage: i32,
) -> Result<()> {
    info!("Updating remaining mileage for all maintenance items");

    let maintenance_items = query_maintenance_items(db, "SELECT * FROM maintenance").await?;

    for item in maintenance_items {
        let remaining_mileage = calculate_remaining_mileage(
            latest_mileage,
            item.last_service_mileage,
            item.mileage_interval,
        );

        let update = MaintenanceItemUpdate {
            remaining_mileage: Some(remaining_mileage),
            ..Default::default()
        };

        let _: Option<MaintenanceItem> = db
            .update(("maintenance", item.id.id.to_string().as_str()))
            .merge(update)
            .await?;
    }

    info!("Updated remaining mileage for all maintenance items");
    Ok(())
}
