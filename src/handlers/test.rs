use axum::{Json, extract::State};
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
    State(state): State<Arc<AppState>>,
) -> Result<Json<TeardownResponse>> {
    info!("TEARDOWN: Clearing all database data");

    // Delete all shifts
    let shifts_query = "DELETE shifts;";
    let mut shifts_result = state.db.query(shifts_query).await?;
    let shifts_deleted: Vec<serde_json::Value> = shifts_result.take(0)?;

    // Delete all maintenance items
    let maintenance_query = "DELETE maintenance;";
    let mut maintenance_result = state.db.query(maintenance_query).await?;
    let maintenance_deleted: Vec<serde_json::Value> = maintenance_result.take(0)?;

    info!(
        "TEARDOWN: Deleted {} shifts and {} maintenance items",
        shifts_deleted.len(),
        maintenance_deleted.len()
    );

    Ok(Json(TeardownResponse {
        message: "All data cleared successfully".to_string(),
        shifts_deleted: shifts_deleted.len(),
        maintenance_deleted: maintenance_deleted.len(),
    }))
}
