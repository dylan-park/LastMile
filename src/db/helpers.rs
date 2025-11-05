use surrealdb::{Surreal, engine::local::Db};

use crate::{
    error::{AppError, Result},
    models::Shift,
};

pub async fn has_active_shift(db: &Surreal<Db>) -> Result<bool> {
    let query = "SELECT * FROM shifts WHERE end_time = NONE LIMIT 1";
    let mut result = db.query(query).await.map_err(Box::new)?;
    let shifts: Vec<Shift> = result.take(0).map_err(Box::new)?;
    Ok(!shifts.is_empty())
}

pub async fn get_shift_by_id(db: &Surreal<Db>, id: &str) -> Result<Shift> {
    let shift: Option<Shift> = db.select(("shifts", id)).await.map_err(Box::new)?;
    shift.ok_or(AppError::ShiftNotFound)
}
