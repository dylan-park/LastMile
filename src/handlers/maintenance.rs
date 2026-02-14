use crate::middleware::SessionId;
use axum::{
    Json,
    extract::{Extension, Path},
};
use std::sync::Arc;
use tracing::info;

use crate::{
    calculations::{calculate_is_maintenance_required, calculate_remaining_mileage},
    db::helpers::{get_maitenance_item_by_id, query_maitenance_items, query_shifts},
    error::{AppError, Result},
    models::{
        CreateMaintenanceItemRequest, MaintenanceItem, MaintenanceItemRecord,
        MaintenanceItemUpdate, RequiredMaintenanceResponse, UpdateMaintenanceItemRequest,
    },
    state::AppState,
    validation,
};

pub async fn get_all_maintenance_items(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session): Extension<SessionId>,
) -> Result<Json<Vec<MaintenanceItem>>> {
    info!("Fetching all maintenance items for session {}", session.0);
    let db = state.db_provider.get_db(Some(&session.0)).await?;

    let maintenance_items = query_maitenance_items(&db, "SELECT * FROM maintenance").await?;

    info!("Retrieved {} maintenance items", maintenance_items.len());
    Ok(Json(maintenance_items))
}

pub async fn create_maintenance_item(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session): Extension<SessionId>,
    Json(payload): Json<CreateMaintenanceItemRequest>,
) -> Result<Json<MaintenanceItem>> {
    info!("Creating new maintenance item: {}", payload.name);
    let db = state.db_provider.get_db(Some(&session.0)).await?;

    // Get latest odometer reading to calculate remaining mileage
    let latest_mileage: i32 = query_shifts(
        &db,
        "SELECT * FROM shifts WHERE odometer_end != NONE ORDER BY start_time DESC LIMIT 1;",
    )
    .await?
    .first()
    .and_then(|shift| shift.odometer_end)
    .unwrap_or(0);

    let last_service_mileage = payload.last_service_mileage.unwrap_or(0);
    let remaining_mileage = calculate_remaining_mileage(
        latest_mileage,
        last_service_mileage,
        payload.mileage_interval,
    );

    let record = MaintenanceItemRecord {
        name: payload.name,
        mileage_interval: payload.mileage_interval,
        last_service_mileage,
        remaining_mileage,
        enabled: payload.enabled,
        notes: payload.notes,
    };

    // Create returns Option<T>
    let maintenance_item: Option<MaintenanceItem> =
        db.create("maintenance").content(record).await?;
    let maintenance_item = maintenance_item.ok_or_else(|| {
        AppError::Database(Box::new(surrealdb::Error::Api(
            surrealdb::error::Api::Query("Failed to create maintenance item".to_string()),
        )))
    })?;

    info!(
        "Maintenance Item created successfully: id={:?}",
        maintenance_item.id
    );
    Ok(Json(maintenance_item))
}

pub async fn update_maintenance_item(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(session): Extension<SessionId>,
    Json(payload): Json<UpdateMaintenanceItemRequest>,
) -> Result<Json<MaintenanceItem>> {
    info!("Updating maintenance item: id={}", id);
    let db = state.db_provider.get_db(Some(&session.0)).await?;

    let maintenance_item = get_maitenance_item_by_id(&db, &id).await?;

    // Merge updates with existing values
    let name = payload.name.unwrap_or(maintenance_item.name);
    let mileage_interval = payload
        .mileage_interval
        .unwrap_or(maintenance_item.mileage_interval);
    let last_service_mileage = payload
        .last_service_mileage
        .unwrap_or(maintenance_item.last_service_mileage);
    let enabled = payload.enabled.unwrap_or(maintenance_item.enabled);

    // Handle notes: if payload.notes is Some, it means the field was included in the request
    // The inner Option tells us if it should be Some(value) or None (cleared)
    let notes = match payload.notes {
        Some(inner) => validation::sanitize_notes(inner),
        None => maintenance_item.notes, // Field wasn't included, keep existing value
    };

    // Get latest odometer reading to recalculate remaining mileage
    let latest_mileage: i32 = query_shifts(
        &db,
        "SELECT * FROM shifts WHERE odometer_end != NONE ORDER BY start_time DESC LIMIT 1;",
    )
    .await?
    .first()
    .and_then(|shift| shift.odometer_end)
    .unwrap_or(0);

    let remaining_mileage =
        calculate_remaining_mileage(latest_mileage, last_service_mileage, mileage_interval);

    // Create update struct with proper SurrealDB types
    let update = MaintenanceItemUpdate {
        name: Some(name),
        mileage_interval: Some(mileage_interval),
        last_service_mileage: Some(last_service_mileage),
        remaining_mileage: Some(remaining_mileage),
        enabled: Some(enabled),
        notes,
    };

    // Update the maintenance item - returns Option<T> when using record ID
    let updated_maintenance_item: Option<MaintenanceItem> = db
        .update(("maintenance", id.as_str()))
        .merge(update)
        .await?;

    let updated_maintenance_item =
        updated_maintenance_item.ok_or(AppError::MaintenanceItemNotFound)?;

    info!("Maintenance Item updated successfully: id={}", id);
    Ok(Json(updated_maintenance_item))
}

pub async fn delete_maintenance_item(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(session): Extension<SessionId>,
) -> Result<Json<MaintenanceItem>> {
    info!("Deleting maintenance item: id={}", id);
    let db = state.db_provider.get_db(Some(&session.0)).await?;

    // Delete the maintenance item - returns Option<T> when using record ID
    let deleted_maintenance_item: Option<MaintenanceItem> =
        db.delete(("maintenance", id.as_str())).await?;

    let deleted_maintenance_item =
        deleted_maintenance_item.ok_or(AppError::MaintenanceItemNotFound)?;

    info!("Maintenance Item deleted successfully: id={}", id);
    Ok(Json(deleted_maintenance_item))
}

pub async fn calculate_required_maintenance(
    Extension(state): Extension<Arc<AppState>>,
    Extension(session): Extension<SessionId>,
) -> Result<Json<RequiredMaintenanceResponse>> {
    info!("Calculating required maintenance items");
    let db = state.db_provider.get_db(Some(&session.0)).await?;

    let latest_mileage: Option<i32> = query_shifts(
        &db,
        "SELECT * FROM shifts WHERE odometer_end != NONE ORDER BY start_time DESC LIMIT 1;",
    )
    .await?
    .first()
    .and_then(|shift| shift.odometer_end);

    let latest_mileage = match latest_mileage {
        Some(m) => m,
        None => {
            return Ok(Json(RequiredMaintenanceResponse {
                required_maintenance_items: vec![],
            }));
        }
    };

    let maintenance_items =
        query_maitenance_items(&db, "SELECT * FROM maintenance WHERE enabled = true;").await?;

    let required_maintenance_items: Vec<_> = maintenance_items
        .into_iter()
        .filter(|item| {
            calculate_is_maintenance_required(
                latest_mileage,
                item.last_service_mileage,
                item.mileage_interval,
            )
        })
        .collect();

    info!("Calculated required maintenance items");
    Ok(Json(RequiredMaintenanceResponse {
        required_maintenance_items,
    }))
}
