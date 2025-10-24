use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};
use bigdecimal::BigDecimal;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, Transaction, mysql::MySqlPoolOptions};
use std::sync::Arc;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

// ===== ERROR TYPES =====
#[derive(Debug, Error)]
enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Shift not found")]
    ShiftNotFound,

    #[error("Active shift already exists")]
    ActiveShiftExists,

    #[error(
        "Invalid odometer reading: end ({end}) must be greater than or equal to start ({start})"
    )]
    InvalidOdometer { start: i32, end: i32 },

    #[error("Invalid monetary value: {0} must be non-negative")]
    InvalidMonetaryValue(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Database(e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::ShiftNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::ActiveShiftExists => (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidOdometer { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidMonetaryValue(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

type Result<T> = std::result::Result<T, AppError>;

// ===== DOMAIN TYPES =====
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Shift {
    id: i32,
    start_time: NaiveDateTime,
    end_time: Option<NaiveDateTime>,
    hours_worked: Option<BigDecimal>,
    odometer_start: i32,
    odometer_end: Option<i32>,
    miles_driven: Option<i32>,
    earnings: BigDecimal,
    tips: BigDecimal,
    gas_cost: BigDecimal,
    day_total: BigDecimal,
    hourly_pay: Option<BigDecimal>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StartShiftRequest {
    odometer_start: i32,
}

#[derive(Debug, Deserialize)]
struct EndShiftRequest {
    odometer_end: i32,
    earnings: Option<BigDecimal>,
    tips: Option<BigDecimal>,
    gas_cost: Option<BigDecimal>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateShiftRequest {
    odometer_start: Option<i32>,
    odometer_end: Option<i32>,
    earnings: Option<BigDecimal>,
    tips: Option<BigDecimal>,
    gas_cost: Option<BigDecimal>,
    notes: Option<String>,
}

struct AppState {
    db: Pool<MySql>,
}

// ===== CALCULATION HELPERS =====
mod calculations {
    use super::*;
    use chrono::Duration;

    pub fn calculate_miles(odometer_start: i32, odometer_end: i32) -> i32 {
        odometer_end - odometer_start
    }

    pub fn calculate_hours(start_time: NaiveDateTime, end_time: NaiveDateTime) -> BigDecimal {
        let duration: Duration = end_time.signed_duration_since(start_time);
        BigDecimal::from(duration.num_seconds()) / BigDecimal::from(3600)
    }

    pub fn calculate_day_total(
        earnings: &BigDecimal,
        tips: &BigDecimal,
        gas_cost: &BigDecimal,
    ) -> BigDecimal {
        earnings + tips - gas_cost
    }

    pub fn calculate_hourly_pay(
        day_total: &BigDecimal,
        hours_worked: &BigDecimal,
    ) -> Option<BigDecimal> {
        if hours_worked > &BigDecimal::from(0) {
            Some(day_total / hours_worked)
        } else {
            None
        }
    }
}

// ===== VALIDATION =====
mod validation {
    use super::*;

    pub fn validate_odometer(start: i32, end: i32) -> Result<()> {
        if end < start {
            return Err(AppError::InvalidOdometer { start, end });
        }
        Ok(())
    }

    pub fn validate_monetary_value(name: &str, value: &BigDecimal) -> Result<()> {
        if value < &BigDecimal::from(0) {
            return Err(AppError::InvalidMonetaryValue(name.to_string()));
        }
        Ok(())
    }

    pub fn validate_monetary_values(
        earnings: &BigDecimal,
        tips: &BigDecimal,
        gas_cost: &BigDecimal,
    ) -> Result<()> {
        validate_monetary_value("earnings", earnings)?;
        validate_monetary_value("tips", tips)?;
        validate_monetary_value("gas_cost", gas_cost)?;
        Ok(())
    }

    pub fn sanitize_notes(notes: Option<String>) -> Option<String> {
        notes.and_then(|n| {
            let trimmed = n.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }
}

// ===== DATABASE HELPERS =====
async fn has_active_shift(tx: &mut Transaction<'_, MySql>) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM shifts WHERE end_time IS NULL FOR UPDATE",
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(count > 0)
}

async fn get_shift_by_id(db: &Pool<MySql>, id: i32) -> Result<Shift> {
    sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::ShiftNotFound)
}

// ===== MAIN =====
#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost/uber_eats_tracker".to_string());

    info!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    info!("Setting up database schema...");
    setup_database(&pool).await;

    let state = Arc::new(AppState { db: pool });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/shifts", get(get_all_shifts))
        .route("/api/shifts/active", get(get_active_shift))
        .route("/api/shifts/start", post(start_shift))
        .route("/api/shifts/{id}/end", post(end_shift))
        .route("/api/shifts/{id}", put(update_shift))
        .route("/api/shifts/export", get(export_csv))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn setup_database(pool: &Pool<MySql>) {
    // Create table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS shifts (
            id INT AUTO_INCREMENT PRIMARY KEY,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            hours_worked DECIMAL(10,2),
            odometer_start INT NOT NULL,
            odometer_end INT,
            miles_driven INT,
            earnings DECIMAL(10,2) NOT NULL DEFAULT 0.00,
            tips DECIMAL(10,2) NOT NULL DEFAULT 0.00,
            gas_cost DECIMAL(10,2) NOT NULL DEFAULT 0.00,
            day_total DECIMAL(10,2) NOT NULL DEFAULT 0.00,
            hourly_pay DECIMAL(10,2),
            notes TEXT,
            INDEX idx_start_time (start_time DESC),
            INDEX idx_end_time (end_time)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create table");

    // Add index if it doesn't exist (safe for existing databases)
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_end_time ON shifts(end_time)")
        .execute(pool)
        .await;

    info!("Database schema ready");
}

// ===== API HANDLERS =====
async fn get_all_shifts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Shift>>> {
    info!("Fetching all shifts");
    let shifts = sqlx::query_as::<_, Shift>("SELECT * FROM shifts ORDER BY start_time DESC")
        .fetch_all(&state.db)
        .await?;

    info!("Retrieved {} shifts", shifts.len());
    Ok(Json(shifts))
}

async fn get_active_shift(State(state): State<Arc<AppState>>) -> Result<Json<Option<Shift>>> {
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

async fn start_shift(
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

async fn end_shift(
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

async fn update_shift(
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

async fn export_csv(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
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
