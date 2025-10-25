use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use bigdecimal::BigDecimal;
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, warn};

use crate::calculations;
use crate::db::helpers::{get_shift_by_id, has_active_shift};
use crate::error::{AppError, Result};
use crate::models::{EndShiftRequest, Shift, StartShiftRequest, UpdateShiftRequest};
use crate::state::AppState;
use crate::validation;

pub async fn get_all_shifts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Shift>>> {
    info!("Fetching all shifts");
    let shifts = sqlx::query_as::<_, Shift>("SELECT * FROM shifts ORDER BY start_time DESC")
        .fetch_all(&state.db)
        .await?;

    info!("Retrieved {} shifts", shifts.len());
    Ok(Json(shifts))
}

pub async fn get_active_shift(State(state): State<Arc<AppState>>) -> Result<Json<Option<Shift>>> {
    info!("Checking for active shift");
    let shift = sqlx::query_as::<_, Shift>(
        "SELECT * FROM shifts WHERE end_time IS NULL ORDER BY start_time DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;

    if shift.is_some() {
        info!("Active shift found");
    } else {
        info!("No active shift");
    }

    Ok(Json(shift))
}

pub async fn start_shift(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<StartShiftRequest>,
) -> Result<Json<Shift>> {
    info!(
        "Starting new shift with odometer: {}",
        payload.odometer_start
    );

    // Use transaction to prevent race condition
    let mut tx = state.db.begin().await?;

    // Check for active shift with row lock
    if has_active_shift(&mut tx).await? {
        warn!("Attempted to start shift while one is already active");
        return Err(AppError::ActiveShiftExists);
    }

    let now = Utc::now();

    let result = sqlx::query(
        r#"
        INSERT INTO shifts (start_time, odometer_start, earnings, tips, gas_cost, day_total)
        VALUES (?, ?, 0.00, 0.00, 0.00, 0.00)
        "#,
    )
    .bind(now)
    .bind(payload.odometer_start)
    .execute(&mut *tx)
    .await?;

    let shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(result.last_insert_id() as i32)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    info!("Shift started successfully: id={}", shift.id);
    Ok(Json(shift))
}

pub async fn end_shift(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<EndShiftRequest>,
) -> Result<Json<Shift>> {
    info!("Ending shift: id={}", id);

    let shift = get_shift_by_id(&state.db, id).await?;

    // Validate inputs
    validation::validate_odometer(shift.odometer_start, payload.odometer_end)?;

    let earnings = payload.earnings.unwrap_or_else(|| BigDecimal::from(0));
    let tips = payload.tips.unwrap_or_else(|| BigDecimal::from(0));
    let gas_cost = payload.gas_cost.unwrap_or_else(|| BigDecimal::from(0));

    validation::validate_monetary_values(&earnings, &tips, &gas_cost)?;

    let notes = validation::sanitize_notes(payload.notes);

    // Calculate derived fields
    let now = Utc::now().naive_utc();
    let miles_driven = calculations::calculate_miles(shift.odometer_start, payload.odometer_end);
    let hours_worked = calculations::calculate_hours(shift.start_time, now);
    let day_total = calculations::calculate_day_total(&earnings, &tips, &gas_cost);
    let hourly_pay = calculations::calculate_hourly_pay(&day_total, &hours_worked);

    // Use transaction for consistency
    let mut tx = state.db.begin().await?;

    sqlx::query(
        r#"
        UPDATE shifts
        SET end_time = ?,
            odometer_end = ?,
            miles_driven = ?,
            hours_worked = ?,
            earnings = ?,
            tips = ?,
            gas_cost = ?,
            day_total = ?,
            hourly_pay = ?,
            notes = ?
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(payload.odometer_end)
    .bind(miles_driven)
    .bind(&hours_worked)
    .bind(&earnings)
    .bind(&tips)
    .bind(&gas_cost)
    .bind(&day_total)
    .bind(&hourly_pay)
    .bind(&notes)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    let updated_shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    info!(
        "Shift ended successfully: id={}, hours={}, miles={}",
        id, hours_worked, miles_driven
    );
    Ok(Json(updated_shift))
}

pub async fn update_shift(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateShiftRequest>,
) -> Result<Json<Shift>> {
    info!("Updating shift: id={}", id);

    let shift = get_shift_by_id(&state.db, id).await?;

    // Merge updates with existing values
    let odometer_start = payload.odometer_start.unwrap_or(shift.odometer_start);
    let odometer_end = payload.odometer_end.or(shift.odometer_end);
    let earnings = payload.earnings.unwrap_or(shift.earnings);
    let tips = payload.tips.unwrap_or(shift.tips);
    let gas_cost = payload.gas_cost.unwrap_or(shift.gas_cost);
    let notes = if payload.notes.is_some() {
        validation::sanitize_notes(payload.notes)
    } else {
        shift.notes
    };

    // Validate monetary values
    validation::validate_monetary_values(&earnings, &tips, &gas_cost)?;

    // Validate odometer if both values exist
    if let Some(end) = odometer_end {
        validation::validate_odometer(odometer_start, end)?;
    }

    // Recalculate derived fields
    let miles_driven = odometer_end.map(|end| calculations::calculate_miles(odometer_start, end));

    let hours_worked = if let Some(end_time) = shift.end_time {
        Some(calculations::calculate_hours(shift.start_time, end_time))
    } else {
        None
    };

    let day_total = calculations::calculate_day_total(&earnings, &tips, &gas_cost);

    let hourly_pay = hours_worked
        .as_ref()
        .and_then(|hw| calculations::calculate_hourly_pay(&day_total, hw));

    // Use transaction for consistency
    let mut tx = state.db.begin().await?;

    sqlx::query(
        r#"
        UPDATE shifts
        SET odometer_start = ?,
            odometer_end = ?,
            miles_driven = ?,
            hours_worked = ?,
            earnings = ?,
            tips = ?,
            gas_cost = ?,
            day_total = ?,
            hourly_pay = ?,
            notes = ?
        WHERE id = ?
        "#,
    )
    .bind(odometer_start)
    .bind(odometer_end)
    .bind(miles_driven)
    .bind(&hours_worked)
    .bind(&earnings)
    .bind(&tips)
    .bind(&gas_cost)
    .bind(&day_total)
    .bind(&hourly_pay)
    .bind(&notes)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    let updated_shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    info!("Shift updated successfully: id={}", id);
    Ok(Json(updated_shift))
}

pub async fn export_csv(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    info!("Exporting shifts to CSV");
    let shifts = sqlx::query_as::<_, Shift>("SELECT * FROM shifts ORDER BY start_time ASC")
        .fetch_all(&state.db)
        .await?;

    let mut csv = String::from(
        "ID,Start Time,End Time,Hours Worked,Odometer Start,Odometer End,Miles Driven,Earnings,Tips,Gas Cost,Day Total,Hourly Pay,Notes\n",
    );

    for shift in &shifts {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            shift.id,
            shift.start_time.format("%Y-%m-%d %H:%M:%S"),
            shift
                .end_time
                .map_or(String::new(), |t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            shift
                .hours_worked
                .as_ref()
                .map_or(String::new(), |h| h.to_string()),
            shift.odometer_start,
            shift.odometer_end.map_or(String::new(), |o| o.to_string()),
            shift.miles_driven.map_or(String::new(), |m| m.to_string()),
            shift.earnings,
            shift.tips,
            shift.gas_cost,
            shift.day_total,
            shift
                .hourly_pay
                .as_ref()
                .map_or(String::new(), |hp| hp.to_string()),
            shift.notes.as_deref().unwrap_or("").replace(',', ";")
        ));
    }

    info!("Exported {} shifts", shifts.len());

    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "text/csv"),
            (
                "Content-Disposition",
                "attachment; filename=\"uber_eats_shifts.csv\"",
            ),
        ],
        csv,
    ))
}
