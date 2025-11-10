use surrealdb::{Surreal, engine::local::Db};

use crate::{
    error::{AppError, Result},
    models::{MaintenanceItem, Shift},
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

pub async fn query_maitenance_items(db: &Surreal<Db>, query: &str) -> Result<Vec<MaintenanceItem>> {
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

pub async fn get_maitenance_item_by_id(db: &Surreal<Db>, id: &str) -> Result<MaintenanceItem> {
    let maintenance_item: Option<MaintenanceItem> = db.select(("maintenance", id)).await?;
    maintenance_item.ok_or(AppError::MaintenanceItemNotFound)
}
