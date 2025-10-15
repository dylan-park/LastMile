use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};
use bigdecimal::BigDecimal;
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

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

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost/uber_eats_tracker".to_string());

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

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
            notes TEXT,
            INDEX idx_start_time (start_time DESC)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    let state = Arc::new(AppState { db: pool });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/shifts", get(get_all_shifts))
        .route("/api/shifts/active", get(get_active_shift))
        .route("/api/shifts/start", post(start_shift))
        .route("/api/shifts/:id/end", post(end_shift))
        .route("/api/shifts/:id", put(update_shift))
        .route("/api/shifts/export", get(export_csv))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn get_all_shifts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Shift>>, StatusCode> {
    let shifts = sqlx::query_as::<_, Shift>("SELECT * FROM shifts ORDER BY start_time DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(shifts))
}

async fn get_active_shift(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<Shift>>, StatusCode> {
    let shift = sqlx::query_as::<_, Shift>(
        "SELECT * FROM shifts WHERE end_time IS NULL ORDER BY start_time DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(shift))
}

async fn start_shift(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<StartShiftRequest>,
) -> Result<Json<Shift>, StatusCode> {
    // Check if there's already an active shift
    let active = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM shifts WHERE end_time IS NULL")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if active > 0 {
        return Err(StatusCode::CONFLICT);
    }

    let now = Local::now().naive_local();

    let result = sqlx::query(
        r#"
        INSERT INTO shifts (start_time, odometer_start, earnings, tips, gas_cost, day_total)
        VALUES (?, ?, 0.00, 0.00, 0.00, 0.00)
        "#,
    )
    .bind(now)
    .bind(payload.odometer_start)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(result.last_insert_id())
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(shift))
}

async fn end_shift(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<EndShiftRequest>,
) -> Result<Json<Shift>, StatusCode> {
    let now = Local::now().naive_local();

    // Get the shift to calculate hours and miles
    let shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let miles_driven = payload.odometer_end - shift.odometer_start;
    let duration = now.signed_duration_since(shift.start_time);
    let hours_worked = BigDecimal::from(duration.num_seconds()) / BigDecimal::from(3600);

    let earnings = payload.earnings.unwrap_or(BigDecimal::from(0));
    let tips = payload.tips.unwrap_or(BigDecimal::from(0));
    let gas_cost = payload.gas_cost.unwrap_or(BigDecimal::from(0));
    let day_total = &earnings + &tips - &gas_cost;

    let notes = payload
        .notes
        .and_then(|n| if n.trim().is_empty() { None } else { Some(n) });

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
            notes = ?
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(payload.odometer_end)
    .bind(miles_driven)
    .bind(hours_worked)
    .bind(earnings)
    .bind(tips)
    .bind(gas_cost)
    .bind(day_total)
    .bind(notes)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated_shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_shift))
}

async fn update_shift(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateShiftRequest>,
) -> Result<Json<Shift>, StatusCode> {
    // Get current shift data
    let shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let odometer_start = payload.odometer_start.unwrap_or(shift.odometer_start);
    let odometer_end = payload.odometer_end.or(shift.odometer_end);
    let earnings = payload.earnings.unwrap_or(shift.earnings);
    let tips = payload.tips.unwrap_or(shift.tips);
    let gas_cost = payload.gas_cost.unwrap_or(shift.gas_cost);
    let notes = payload
        .notes
        .and_then(|n| if n.trim().is_empty() { None } else { Some(n) })
        .or(shift.notes);

    // Recalculate derived fields
    let miles_driven = odometer_end.map(|end| end - odometer_start);

    let hours_worked = if let Some(end_time) = shift.end_time {
        let duration = end_time.signed_duration_since(shift.start_time);
        Some(BigDecimal::from(duration.num_seconds()) / BigDecimal::from(3600))
    } else {
        None
    };

    let day_total = &earnings + &tips - &gas_cost;

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
            notes = ?
        WHERE id = ?
        "#,
    )
    .bind(odometer_start)
    .bind(odometer_end)
    .bind(miles_driven)
    .bind(hours_worked)
    .bind(earnings)
    .bind(tips)
    .bind(gas_cost)
    .bind(day_total)
    .bind(notes)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated_shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_shift))
}

async fn export_csv(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, StatusCode> {
    let shifts = sqlx::query_as::<_, Shift>("SELECT * FROM shifts ORDER BY start_time ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut csv = String::from(
        "ID,Start Time,End Time,Hours Worked,Odometer Start,Odometer End,Miles Driven,Earnings,Tips,Gas Cost,Day Total,Notes\n",
    );

    for shift in shifts {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            shift.id,
            shift.start_time.format("%Y-%m-%d %H:%M:%S"),
            shift
                .end_time
                .map_or(String::new(), |t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            shift.hours_worked.map_or(String::new(), |h| h.to_string()),
            shift.odometer_start,
            shift.odometer_end.map_or(String::new(), |o| o.to_string()),
            shift.miles_driven.map_or(String::new(), |m| m.to_string()),
            shift.earnings,
            shift.tips,
            shift.gas_cost,
            shift.day_total,
            shift.notes.as_deref().unwrap_or("")
        ));
    }

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
