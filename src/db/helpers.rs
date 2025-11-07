use surrealdb::{Surreal, engine::local::Db};

use crate::{
    error::{AppError, Result},
    models::Shift,
};

// Helper to execute a query and extract shifts
pub async fn query_shifts(db: &Surreal<Db>, query: &str) -> Result<Vec<Shift>> {
    let mut result = db.query(query).await?;
    let shifts: Vec<Shift> = result.take(0)?;
    Ok(shifts)
}

pub async fn has_active_shift(db: &Surreal<Db>) -> Result<bool> {
    let shifts = query_shifts(db, "SELECT * FROM shifts WHERE end_time = NONE LIMIT 1").await?;
    Ok(!shifts.is_empty())
}

pub async fn get_shift_by_id(db: &Surreal<Db>, id: &str) -> Result<Shift> {
    let shift: Option<Shift> = db.select(("shifts", id)).await?;
    shift.ok_or(AppError::ShiftNotFound)
}
