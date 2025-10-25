use crate::error::{AppError, Result};
use crate::models::Shift;
use sqlx::{MySql, Pool, Transaction};

pub async fn has_active_shift(tx: &mut Transaction<'_, MySql>) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM shifts WHERE end_time IS NULL FOR UPDATE",
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(count > 0)
}

pub async fn get_shift_by_id(db: &Pool<MySql>, id: i32) -> Result<Shift> {
    sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::ShiftNotFound)
}
