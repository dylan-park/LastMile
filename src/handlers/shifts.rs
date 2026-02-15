use crate::middleware::SessionId;

use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    calculations,
    db::helpers::{
        get_shift_by_id, has_active_shift, query_shifts, query_shifts_with_date_range,
        update_all_maintenance_remaining_mileage,
    },
    error::{AppError, Result},
    models::{
        DateRangeQuery, EndShiftRequest, OptionalDateRangeQuery, Shift, ShiftRecord, ShiftUpdate,
        StartShiftRequest, UpdateShiftRequest,
    },
    state::AppState,
    validation,
};

pub async fn get_all_shifts(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session_id): Extension<SessionId>,
) -> Result<Json<Vec<Shift>>> {
    info!("Fetching all shifts for session {}", session_id.0);
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    let shifts = query_shifts(&db, "SELECT * FROM shifts ORDER BY start_time DESC").await?;

    info!("Retrieved {} shifts", shifts.len());
    Ok(Json(shifts))
}

pub async fn get_shifts_by_range(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session_id): Extension<SessionId>,
    Query(params): Query<DateRangeQuery>,
) -> Result<Json<Vec<Shift>>> {
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;
    info!(
        "Fetching shifts in range: {} to {}",
        params.start, params.end
    );

    // Parse the ISO 8601 datetime strings
    let start_time: DateTime<Utc> = params.start.parse().map_err(|e| {
        warn!("Invalid start date format: {}", e);
        AppError::Database(Box::new(surrealdb::Error::Api(
            surrealdb::error::Api::Query("Invalid start date format".to_string()),
        )))
    })?;

    let end_time: DateTime<Utc> = params.end.parse().map_err(|e| {
        warn!("Invalid end date format: {}", e);
        AppError::Database(Box::new(surrealdb::Error::Api(
            surrealdb::error::Api::Query("Invalid end date format".to_string()),
        )))
    })?;

    // Convert to SurrealDB datetime for query
    let start_surreal: surrealdb::sql::Datetime = start_time.into();
    let end_surreal: surrealdb::sql::Datetime = end_time.into();

    // Query shifts within the date range
    let query = "SELECT * FROM shifts WHERE start_time >= $start AND start_time <= $end ORDER BY start_time DESC";
    let shifts = query_shifts_with_date_range(&db, query, start_surreal, end_surreal).await?;

    info!("Retrieved {} shifts in range", shifts.len());
    Ok(Json(shifts))
}

pub async fn get_active_shift(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session_id): Extension<SessionId>,
) -> Result<Json<Option<Shift>>> {
    info!("Checking for active shift");
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    let shifts = query_shifts(
        &db,
        "SELECT * FROM shifts WHERE end_time = NONE ORDER BY start_time DESC LIMIT 1",
    )
    .await?;
    let shift = shifts.into_iter().next();

    if shift.is_some() {
        info!("Active shift found");
    } else {
        info!("No active shift");
    }

    Ok(Json(shift))
}

pub async fn start_shift(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session_id): Extension<SessionId>,
    Json(payload): Json<StartShiftRequest>,
) -> Result<Json<Shift>> {
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    info!(
        "Starting new shift with odometer: {}",
        payload.odometer_start
    );

    // Check for active shift
    if has_active_shift(&db).await? {
        warn!("Attempted to start shift while one is already active");
        return Err(AppError::ActiveShiftExists);
    }

    let now = Utc::now();

    let record = ShiftRecord {
        start_time: now.into(),
        end_time: None,
        hours_worked: None,
        odometer_start: payload.odometer_start,
        odometer_end: None,
        miles_driven: None,
        earnings: calculations::normalize_decimal(Decimal::ZERO),
        tips: calculations::normalize_decimal(Decimal::ZERO),
        gas_cost: calculations::normalize_decimal(Decimal::ZERO),
        day_total: calculations::normalize_decimal(Decimal::ZERO),
        hourly_pay: None,
        notes: None,
    };

    // Create returns Option<T>
    let shift: Option<Shift> = db.create("shifts").content(record).await?;
    let shift = shift.ok_or_else(|| {
        AppError::Database(Box::new(surrealdb::Error::Api(
            surrealdb::error::Api::Query("Failed to create shift".to_string()),
        )))
    })?;

    info!("Shift started successfully: id={:?}", shift.id);
    Ok(Json(shift))
}

pub async fn end_shift(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(session_id): Extension<SessionId>,
    Json(payload): Json<EndShiftRequest>,
) -> Result<Json<Shift>> {
    info!("Ending shift: id={}", id);
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    let shift = get_shift_by_id(&db, &id).await?;

    // Validate inputs
    validation::validate_odometer(shift.odometer_start, payload.odometer_end)?;

    let earnings = calculations::normalize_decimal(payload.earnings.unwrap_or(Decimal::ZERO));
    let tips = calculations::normalize_decimal(payload.tips.unwrap_or(Decimal::ZERO));
    let gas_cost = calculations::normalize_decimal(payload.gas_cost.unwrap_or(Decimal::ZERO));

    validation::validate_monetary_values(&earnings, &tips, &gas_cost)?;

    let notes = validation::sanitize_notes(payload.notes);

    // Calculate derived fields (already normalized by calculation functions)
    let now = Utc::now();
    let miles_driven = calculations::calculate_miles(shift.odometer_start, payload.odometer_end);
    let hours_worked = calculations::calculate_hours(shift.start_time, now);
    let day_total = calculations::calculate_day_total(&earnings, &tips, &gas_cost);
    let hourly_pay = calculations::calculate_hourly_pay(&day_total, &hours_worked);

    // Create update struct with proper SurrealDB types
    let update = ShiftUpdate {
        start_time: None,
        end_time: Some(now.into()),
        odometer_start: None,
        odometer_end: Some(payload.odometer_end),
        miles_driven: Some(miles_driven),
        hours_worked: Some(hours_worked),
        earnings: Some(earnings),
        tips: Some(tips),
        gas_cost: Some(gas_cost),
        day_total: Some(day_total),
        hourly_pay,
        notes,
    };

    // Update the shift - returns Option<T> when using record ID
    let updated_shift: Option<Shift> = db.update(("shifts", id.as_str())).merge(update).await?;

    let updated_shift = updated_shift.ok_or(AppError::ShiftNotFound)?;

    // Update all maintenance items with new remaining mileage
    update_all_maintenance_remaining_mileage(&db, payload.odometer_end).await?;

    info!(
        "Shift ended successfully: id={}, hours={}, miles={}",
        id, hours_worked, miles_driven
    );
    Ok(Json(updated_shift))
}

pub async fn update_shift(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(session_id): Extension<SessionId>,
    Json(payload): Json<UpdateShiftRequest>,
) -> Result<Json<Shift>> {
    info!("Updating shift: id={}", id);
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    let shift = get_shift_by_id(&db, &id).await?;

    // Parse datetime strings if provided
    let new_start_time: Option<DateTime<Utc>> = if let Some(ref start_str) = payload.start_time {
        Some(start_str.parse().map_err(|e| {
            warn!("Invalid start_time format: {}", e);
            AppError::Database(Box::new(surrealdb::Error::Api(
                surrealdb::error::Api::Query("Invalid start_time format".to_string()),
            )))
        })?)
    } else {
        None
    };

    let new_end_time: Option<DateTime<Utc>> = if let Some(ref end_str) = payload.end_time {
        Some(end_str.parse().map_err(|e| {
            warn!("Invalid end_time format: {}", e);
            AppError::Database(Box::new(surrealdb::Error::Api(
                surrealdb::error::Api::Query("Invalid end_time format".to_string()),
            )))
        })?)
    } else {
        None
    };

    // Determine final start and end times
    let final_start_time = new_start_time.unwrap_or(shift.start_time);
    let final_end_time = new_end_time.or(shift.end_time);

    // Validate: end_time must be after start_time if both exist
    if let Some(end) = final_end_time
        && end <= final_start_time
    {
        warn!("Invalid time range: end_time must be after start_time");
        return Err(AppError::Database(Box::new(surrealdb::Error::Api(
            surrealdb::error::Api::Query("End time must be after start time".to_string()),
        ))));
    }

    // Merge updates with existing values, normalizing user inputs
    let odometer_start = payload.odometer_start.unwrap_or(shift.odometer_start);
    let odometer_end = payload.odometer_end.or(shift.odometer_end);
    let earnings = calculations::normalize_decimal(payload.earnings.unwrap_or(shift.earnings));
    let tips = calculations::normalize_decimal(payload.tips.unwrap_or(shift.tips));
    let gas_cost = calculations::normalize_decimal(payload.gas_cost.unwrap_or(shift.gas_cost));

    // Handle notes: if payload.notes is Some, it means the field was included in the request
    // The inner Option tells us if it should be Some(value) or None (cleared)
    let notes = match payload.notes {
        Some(inner) => validation::sanitize_notes(inner),
        None => shift.notes, // Field wasn't included, keep existing value
    };

    // Validate monetary values
    validation::validate_monetary_values(&earnings, &tips, &gas_cost)?;

    // Validate odometer if both values exist
    if let Some(end) = odometer_end {
        validation::validate_odometer(odometer_start, end)?;
    }

    // Recalculate derived fields (already normalized by calculation functions)
    let miles_driven = odometer_end.map(|end| calculations::calculate_miles(odometer_start, end));

    // Recalculate hours_worked using final times
    let hours_worked =
        final_end_time.map(|end_time| calculations::calculate_hours(final_start_time, end_time));

    let day_total = calculations::calculate_day_total(&earnings, &tips, &gas_cost);

    let hourly_pay = hours_worked
        .as_ref()
        .and_then(|hw| calculations::calculate_hourly_pay(&day_total, hw));

    // Create update struct with proper SurrealDB types
    let update = ShiftUpdate {
        start_time: new_start_time.map(|t| t.into()),
        end_time: new_end_time.map(|t| t.into()),
        odometer_start: Some(odometer_start),
        odometer_end,
        miles_driven,
        hours_worked,
        earnings: Some(earnings),
        tips: Some(tips),
        gas_cost: Some(gas_cost),
        day_total: Some(day_total),
        hourly_pay,
        notes,
    };

    // Update the shift - returns Option<T> when using record ID
    let updated_shift: Option<Shift> = db.update(("shifts", id.as_str())).merge(update).await?;

    let updated_shift = updated_shift.ok_or(AppError::ShiftNotFound)?;

    // Update all maintenance items with new remaining mileage if odometer_end changed
    if let Some(new_odometer_end) = odometer_end
        && shift.odometer_end != Some(new_odometer_end)
    {
        update_all_maintenance_remaining_mileage(&db, new_odometer_end).await?;
    }

    info!("Shift updated successfully: id={}", id);
    Ok(Json(updated_shift))
}

pub async fn delete_shift(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(session_id): Extension<SessionId>,
) -> Result<StatusCode> {
    info!("Deleting shift: id={}", id);
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    // Delete the shift - returns Option<T> when using record ID
    let deleted_shift: Option<Shift> = db.delete(("shifts", id.as_str())).await?;

    let _deleted_shift = deleted_shift.ok_or(AppError::ShiftNotFound)?;

    info!("Shift deleted successfully: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn export_csv(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session_id): Extension<SessionId>,
    Query(params): Query<OptionalDateRangeQuery>,
) -> Result<impl IntoResponse> {
    info!("Exporting shifts to CSV");
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    // Fetch shifts based on whether date range is provided
    let shifts = if let (Some(start_str), Some(end_str)) = (params.start, params.end) {
        info!("Exporting shifts in range: {} to {}", start_str, end_str);

        // Parse the ISO 8601 datetime strings
        let start_time: DateTime<Utc> = start_str.parse().map_err(|e| {
            warn!("Invalid start date format: {}", e);
            AppError::InvalidInput(format!("Invalid start date format: {}", e))
        })?;

        let end_time: DateTime<Utc> = end_str.parse().map_err(|e| {
            warn!("Invalid end date format: {}", e);
            AppError::InvalidInput(format!("Invalid end date format: {}", e))
        })?;

        // Convert to SurrealDB datetime for query
        let start_surreal: surrealdb::sql::Datetime = start_time.into();
        let end_surreal: surrealdb::sql::Datetime = end_time.into();

        // Query shifts within the date range
        let query = "SELECT * FROM shifts WHERE start_time >= $start AND start_time <= $end ORDER BY start_time ASC";
        query_shifts_with_date_range(&db, query, start_surreal, end_surreal).await?
    } else {
        info!("Exporting all shifts");
        query_shifts(&db, "SELECT * FROM shifts ORDER BY start_time ASC").await?
    };

    let mut csv = String::from(
        "ID,Start Time,End Time,Hours Worked,Odometer Start,Odometer End,Miles Driven,Earnings,Tips,Gas Cost,Day Total,Hourly Pay,Notes\n",
    );

    for shift in &shifts {
        // Extract numeric ID from Thing
        let id_str = shift.id.id.to_string();

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            id_str,
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
                "attachment; filename=\"lastmile_shifts.csv\"",
            ),
        ],
        csv,
    ))
}
