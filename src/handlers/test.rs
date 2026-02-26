use crate::middleware::SessionId;

use axum::{Extension, Json};
use serde::Serialize;
use std::sync::Arc;
use tracing::info;

use crate::{error::Result, state::AppState};

#[derive(Debug, Serialize)]
pub struct TeardownResponse {
    pub message: String,
    pub shifts_deleted: usize,
    pub maintenance_deleted: usize,
}

/// Endpoint to clear all data - for testing only
/// WARNING: This deletes ALL data from the database
pub async fn teardown_all_data(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session_id): Extension<SessionId>,
) -> Result<Json<TeardownResponse>> {
    info!(
        "TEARDOWN: Clearing all database data for session {}",
        session_id.0
    );
    let db = state.db_provider.get_db(Some(&session_id.0)).await?;

    // Count shifts, then delete and consume result handle to guarantee completion
    let mut shifts_count_result = db.query("SELECT count() FROM shifts GROUP ALL").await?;
    let shifts_counts: Vec<serde_json::Value> = shifts_count_result.take(0)?;
    let shifts_deleted = shifts_counts
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    db.query("DELETE shifts RETURN NONE")
        .await?
        .take::<Vec<serde_json::Value>>(0)?;

    // Count maintenance items, then delete and consume result handle to guarantee completion
    let mut maintenance_count_result = db
        .query("SELECT count() FROM maintenance GROUP ALL")
        .await?;
    let maintenance_counts: Vec<serde_json::Value> = maintenance_count_result.take(0)?;
    let maintenance_deleted = maintenance_counts
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    db.query("DELETE maintenance RETURN NONE")
        .await?
        .take::<Vec<serde_json::Value>>(0)?;

    info!(
        "TEARDOWN: Deleted {} shifts and {} maintenance items",
        shifts_deleted, maintenance_deleted
    );

    Ok(Json(TeardownResponse {
        message: "All data cleared successfully".to_string(),
        shifts_deleted: shifts_deleted,
        maintenance_deleted: maintenance_deleted,
    }))
}
