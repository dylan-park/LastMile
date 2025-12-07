use axum::Json;
use axum::extract::State;
use rust_decimal_macros::dec;
use std::sync::Arc;

use lastmile::db::helpers::get_maitenance_item_by_id;
use lastmile::handlers::maintenance::*;
use lastmile::handlers::shifts::*;
use lastmile::models::*;
use lastmile::state::AppState;

mod common;

#[tokio::test]
async fn test_start_shift() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let request = StartShiftRequest {
        odometer_start: 12345,
    };

    let result = start_shift(State(state), Json(request)).await;
    assert!(result.is_ok());

    let shift = result.unwrap().0;
    assert_eq!(shift.odometer_start, 12345);
    assert_eq!(shift.earnings, dec!(0.0));
    assert_eq!(shift.tips, dec!(0.0));
    assert_eq!(shift.gas_cost, dec!(0.0));
    assert_eq!(shift.day_total, dec!(0.0));
    assert!(shift.end_time.is_none());
}

#[tokio::test]
async fn test_start_shift_when_active_exists() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let request1 = StartShiftRequest {
        odometer_start: 12345,
    };
    let _ = start_shift(State(state.clone()), Json(request1))
        .await
        .unwrap();

    let request2 = StartShiftRequest {
        odometer_start: 12400,
    };
    let result = start_shift(State(state), Json(request2)).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        lastmile::error::AppError::ActiveShiftExists
    ));
}

#[tokio::test]
async fn test_end_shift() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let start_request = StartShiftRequest {
        odometer_start: 12345,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    // Sleep for 20 second to ensure measurable time difference
    tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;

    let end_request = EndShiftRequest {
        odometer_end: 12445,
        earnings: Some(dec!(100.0)),
        tips: Some(dec!(20.0)),
        gas_cost: Some(dec!(15.0)),
        notes: Some("Good shift".to_string()),
    };

    let result = end_shift(
        State(state),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;
    assert!(result.is_ok());

    let ended_shift = result.unwrap().0;
    print!("{:?}", ended_shift);
    assert_eq!(ended_shift.odometer_end, Some(12445));
    assert_eq!(ended_shift.earnings, dec!(100.0));
    assert_eq!(ended_shift.tips, dec!(20.0));
    assert_eq!(ended_shift.gas_cost, dec!(15.0));
    assert_eq!(ended_shift.day_total, dec!(105.0));
    assert_eq!(ended_shift.miles_driven, Some(100));
    assert!(ended_shift.end_time.is_some());
    assert!(ended_shift.hours_worked.is_some());
    assert!(ended_shift.hourly_pay.is_some());
    assert_eq!(ended_shift.notes, Some("Good shift".to_string()));
}

#[tokio::test]
async fn test_end_shift_invalid_odometer() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let start_request = StartShiftRequest {
        odometer_start: 12445,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 12345,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
    };

    let result = end_shift(
        State(state),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        lastmile::error::AppError::InvalidOdometer { .. }
    ));
}

#[tokio::test]
async fn test_get_active_shift() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let result = get_active_shift(State(state.clone())).await.unwrap();
    assert!(result.0.is_none());

    let start_request = StartShiftRequest {
        odometer_start: 12345,
    };
    let _ = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap();

    let result = get_active_shift(State(state)).await.unwrap();
    assert!(result.0.is_some());
    let active = result.0.unwrap();
    assert_eq!(active.odometer_start, 12345);
}

#[tokio::test]
async fn test_update_shift() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let start_request = StartShiftRequest {
        odometer_start: 12345,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let update_request = UpdateShiftRequest {
        odometer_start: Some(12350),
        odometer_end: Some(12450),
        earnings: Some(dec!(150.0)),
        tips: Some(dec!(30.0)),
        gas_cost: Some(dec!(20.0)),
        notes: Some(Some("Updated".to_string())),
        start_time: None,
        end_time: None,
    };

    let result = update_shift(
        State(state),
        axum::extract::Path(shift_id),
        Json(update_request),
    )
    .await;
    assert!(result.is_ok());

    let updated = result.unwrap().0;
    assert_eq!(updated.odometer_start, 12350);
    assert_eq!(updated.odometer_end, Some(12450));
    assert_eq!(updated.earnings, dec!(150.0));
    assert_eq!(updated.tips, dec!(30.0));
    assert_eq!(updated.gas_cost, dec!(20.0));
    assert_eq!(updated.day_total, dec!(160.0));
    assert_eq!(updated.miles_driven, Some(100));
    assert_eq!(updated.notes, Some("Updated".to_string()));
}

// TODO: Test Delete Shift
#[tokio::test]
async fn test_delete_shift() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let start_request = StartShiftRequest {
        odometer_start: 12445,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 12445,
        earnings: Some(dec!(100.0)),
        tips: Some(dec!(20.0)),
        gas_cost: Some(dec!(15.0)),
        notes: Some("Good shift".to_string()),
    };

    let result = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await;
    assert!(result.is_ok());

    let result = delete_shift(State(state.clone()), axum::extract::Path(shift_id)).await;
    assert!(result.is_ok());

    let all_items = get_all_shifts(State(state)).await.unwrap();
    assert_eq!(all_items.0.len(), 0);
}

#[tokio::test]
async fn test_get_all_shifts() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let _ = start_shift(
        State(state.clone()),
        Json(StartShiftRequest {
            odometer_start: 1000,
        }),
    )
    .await;

    let result = get_all_shifts(State(state)).await.unwrap();
    assert_eq!(result.0.len(), 1);
}

#[tokio::test]
async fn test_create_maintenance_item() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(10000),
        enabled: true,
        notes: Some("Full synthetic".to_string()),
    };

    let result = create_maintenance_item(State(state), Json(request)).await;
    assert!(result.is_ok());

    let item = result.unwrap().0;
    assert_eq!(item.name, "Oil Change");
    assert_eq!(item.mileage_interval, 3000);
    assert_eq!(item.last_service_mileage, 10000);
    assert_eq!(item.enabled, true);
    assert_eq!(item.notes, Some("Full synthetic".to_string()));
    // With no shifts, remaining_mileage should be interval (3000 - (0 - 10000) = 3000, clamped)
    assert_eq!(item.remaining_mileage, 3000);
}

#[tokio::test]
async fn test_update_maintenance_item() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let create_request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(10000),
        enabled: true,
        notes: None,
    };
    let item = create_maintenance_item(State(state.clone()), Json(create_request))
        .await
        .unwrap()
        .0;
    let item_id = item.id.id.to_string();

    let update_request = UpdateMaintenanceItemRequest {
        name: Some("Oil Change - Updated".to_string()),
        mileage_interval: Some(5000),
        last_service_mileage: Some(15000),
        enabled: Some(false),
        notes: Some(Some("Now using synthetic".to_string())),
    };

    let result = update_maintenance_item(
        State(state),
        axum::extract::Path(item_id),
        Json(update_request),
    )
    .await;
    assert!(result.is_ok());

    let updated = result.unwrap().0;
    assert_eq!(updated.name, "Oil Change - Updated");
    assert_eq!(updated.mileage_interval, 5000);
    assert_eq!(updated.last_service_mileage, 15000);
    assert_eq!(updated.enabled, false);
    assert_eq!(updated.notes, Some("Now using synthetic".to_string()));
}

#[tokio::test]
async fn test_delete_maintenance_item() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let create_request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(10000),
        enabled: true,
        notes: None,
    };
    let item = create_maintenance_item(State(state.clone()), Json(create_request))
        .await
        .unwrap()
        .0;
    let item_id = item.id.id.to_string();

    let result =
        delete_maintenance_item(State(state.clone()), axum::extract::Path(item_id.clone())).await;
    assert!(result.is_ok());

    let all_items = get_all_maintenance_items(State(state)).await.unwrap();
    assert_eq!(all_items.0.len(), 0);
}

#[tokio::test]
async fn test_calculate_required_maintenance_none_required() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let create_request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(10000),
        enabled: true,
        notes: None,
    };
    let _ = create_maintenance_item(State(state.clone()), Json(create_request)).await;

    let start_request = StartShiftRequest {
        odometer_start: 12000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 12500,
        earnings: Some(dec!(100.0)),
        tips: None,
        gas_cost: None,
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;

    let result = calculate_required_maintenance(State(state)).await.unwrap();
    assert_eq!(result.0.required_maintenance_items.len(), 0);
}

#[tokio::test]
async fn test_calculate_required_maintenance_required() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let create_request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(10000),
        enabled: true,
        notes: None,
    };
    let _ = create_maintenance_item(State(state.clone()), Json(create_request)).await;

    let start_request = StartShiftRequest {
        odometer_start: 12000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 13500,
        earnings: Some(dec!(100.0)),
        tips: None,
        gas_cost: None,
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;

    let result = calculate_required_maintenance(State(state)).await.unwrap();
    assert_eq!(result.0.required_maintenance_items.len(), 1);
    assert_eq!(result.0.required_maintenance_items[0].name, "Oil Change");
}

#[tokio::test]
async fn test_calculate_required_maintenance_disabled_item() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let create_request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(10000),
        enabled: false,
        notes: None,
    };
    let _ = create_maintenance_item(State(state.clone()), Json(create_request)).await;

    let start_request = StartShiftRequest {
        odometer_start: 12000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 13500,
        earnings: Some(dec!(100.0)),
        tips: None,
        gas_cost: None,
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;

    let result = calculate_required_maintenance(State(state)).await.unwrap();
    assert_eq!(result.0.required_maintenance_items.len(), 0);
}

#[tokio::test]
async fn test_create_maintenance_item_with_existing_shift() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Create a shift with odometer reading
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 12000,
        earnings: Some(dec!(100.0)),
        tips: None,
        gas_cost: None,
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;

    // Create maintenance item
    // Last service at 8000, interval 3000, current mileage 12000
    // Remaining: 3000 - (12000 - 8000) = -1000, clamped to 0
    let request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(8000),
        enabled: true,
        notes: None,
    };

    let result = create_maintenance_item(State(state), Json(request)).await;
    assert!(result.is_ok());

    let item = result.unwrap().0;
    assert_eq!(item.remaining_mileage, 0); // Overdue, clamped to 0
}

#[tokio::test]
async fn test_update_maintenance_item_recalculates_remaining_mileage() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Create a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 11000,
        earnings: Some(dec!(100.0)),
        tips: None,
        gas_cost: None,
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;

    // Create maintenance item
    let create_request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 5000,
        last_service_mileage: Some(8000),
        enabled: true,
        notes: None,
    };
    let item = create_maintenance_item(State(state.clone()), Json(create_request))
        .await
        .unwrap()
        .0;
    let item_id = item.id.id.to_string();

    // Initial: interval 5000, last service 8000, current 11000
    // Remaining: 5000 - (11000 - 8000) = 2000
    assert_eq!(item.remaining_mileage, 2000);

    // Update last_service_mileage to 10000
    // New remaining: 5000 - (11000 - 10000) = 4000
    let update_request = UpdateMaintenanceItemRequest {
        name: None,
        mileage_interval: None,
        last_service_mileage: Some(10000),
        enabled: None,
        notes: None,
    };

    let result = update_maintenance_item(
        State(state),
        axum::extract::Path(item_id),
        Json(update_request),
    )
    .await;
    assert!(result.is_ok());

    let updated = result.unwrap().0;
    assert_eq!(updated.remaining_mileage, 4000);
}

#[tokio::test]
async fn test_end_shift_updates_maintenance_remaining_mileage() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Create maintenance item first
    let create_request = CreateMaintenanceItemRequest {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: Some(10000),
        enabled: true,
        notes: None,
    };
    let item = create_maintenance_item(State(state.clone()), Json(create_request))
        .await
        .unwrap()
        .0;
    let item_id = item.id.id.to_string();

    // Initial remaining_mileage with no shifts (defaults to 0 latest mileage)
    // Remaining: 3000 - (0 - 10000) = 3000 (clamped)
    assert_eq!(item.remaining_mileage, 3000);

    // Start and end a shift
    let start_request = StartShiftRequest {
        odometer_start: 11000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 12500,
        earnings: Some(dec!(100.0)),
        tips: None,
        gas_cost: None,
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id),
        Json(end_request),
    )
    .await;

    // Fetch the maintenance item again
    // New remaining: 3000 - (12500 - 10000) = 500
    let updated_item = get_maitenance_item_by_id(&state.db, &item_id)
        .await
        .unwrap();
    assert_eq!(updated_item.remaining_mileage, 500);
}

#[tokio::test]
async fn test_update_shift_odometer_updates_maintenance_remaining_mileage() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Create maintenance item
    let create_request = CreateMaintenanceItemRequest {
        name: "Tire Rotation".to_string(),
        mileage_interval: 5000,
        last_service_mileage: Some(5000),
        enabled: true,
        notes: None,
    };
    let item = create_maintenance_item(State(state.clone()), Json(create_request))
        .await
        .unwrap()
        .0;
    let item_id = item.id.id.to_string();

    // Create a shift
    let start_request = StartShiftRequest {
        odometer_start: 8000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 9000,
        earnings: Some(dec!(100.0)),
        tips: None,
        gas_cost: None,
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await;

    // Check initial remaining: 5000 - (9000 - 5000) = 1000
    let item_after_shift = get_maitenance_item_by_id(&state.db, &item_id)
        .await
        .unwrap();
    assert_eq!(item_after_shift.remaining_mileage, 1000);

    // Update shift odometer_end to 9500
    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: Some(9500),
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: None,
        end_time: None,
    };

    let _ = update_shift(
        State(state.clone()),
        axum::extract::Path(shift_id),
        Json(update_request),
    )
    .await;

    // Check updated remaining: 5000 - (9500 - 5000) = 500
    let item_after_update = get_maitenance_item_by_id(&state.db, &item_id)
        .await
        .unwrap();
    assert_eq!(item_after_update.remaining_mileage, 500);
}

#[tokio::test]
async fn test_update_shift_start_time() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Start a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();
    let original_start = shift.start_time;

    // End the shift
    let end_request = EndShiftRequest {
        odometer_end: 10100,
        earnings: Some(dec!(50.0)),
        tips: Some(dec!(10.0)),
        gas_cost: Some(dec!(5.0)),
        notes: None,
    };
    let ended_shift = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await
    .unwrap()
    .0;

    let original_hours = ended_shift.hours_worked.unwrap();

    // Update start_time to 1 hour earlier
    let new_start_time = original_start - chrono::Duration::hours(1);
    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: Some(new_start_time.to_rfc3339()),
        end_time: None,
    };

    let result = update_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(update_request),
    )
    .await;

    assert!(result.is_ok());
    let updated_shift = result.unwrap().0;

    // Verify start_time changed
    assert_eq!(updated_shift.start_time, new_start_time);

    // Verify hours_worked increased (should be ~1 hour more than original)
    assert!(updated_shift.hours_worked.unwrap() > original_hours);
}

#[tokio::test]
async fn test_update_shift_end_time() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Start and end a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 10100,
        earnings: Some(dec!(60.0)),
        tips: Some(dec!(10.0)),
        gas_cost: Some(dec!(5.0)),
        notes: None,
    };
    let ended_shift = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await
    .unwrap()
    .0;

    let original_end = ended_shift.end_time.unwrap();
    let original_hours = ended_shift.hours_worked.unwrap();

    // Update end_time to 2 hours later
    let new_end_time = original_end + chrono::Duration::hours(2);
    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: None,
        end_time: Some(new_end_time.to_rfc3339()),
    };

    let result = update_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(update_request),
    )
    .await;

    assert!(result.is_ok());
    let updated_shift = result.unwrap().0;

    // Verify end_time changed
    assert_eq!(updated_shift.end_time.unwrap(), new_end_time);

    // Verify hours_worked increased by ~2 hours
    assert!(updated_shift.hours_worked.unwrap() > original_hours + dec!(1.9));

    // Verify hourly_pay recalculated
    let expected_hourly = dec!(65.0) / updated_shift.hours_worked.unwrap();
    assert_eq!(updated_shift.hourly_pay.unwrap(), expected_hourly);
}

#[tokio::test]
async fn test_update_shift_both_times() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Start and end a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();
    let original_start = shift.start_time;

    let end_request = EndShiftRequest {
        odometer_end: 10100,
        earnings: Some(dec!(100.0)),
        tips: Some(dec!(20.0)),
        gas_cost: Some(dec!(10.0)),
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await
    .unwrap();

    // Update both times to create a 3-hour shift
    let new_start_time = original_start;
    let new_end_time = original_start + chrono::Duration::hours(3);

    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: Some(new_start_time.to_rfc3339()),
        end_time: Some(new_end_time.to_rfc3339()),
    };

    let result = update_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(update_request),
    )
    .await;

    assert!(result.is_ok());
    let updated_shift = result.unwrap().0;

    // Verify times changed
    assert_eq!(updated_shift.start_time, new_start_time);
    assert_eq!(updated_shift.end_time.unwrap(), new_end_time);

    // Verify hours_worked is exactly 3
    assert_eq!(updated_shift.hours_worked.unwrap(), dec!(3.0));

    // Verify hourly_pay recalculated: (100 + 20 - 10) / 3 = 36.666...
    let expected_hourly = lastmile::calculations::normalize_decimal(dec!(110.0) / dec!(3.0));
    assert_eq!(updated_shift.hourly_pay.unwrap(), expected_hourly);
}

#[tokio::test]
async fn test_update_shift_invalid_end_before_start() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Start and end a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 10100,
        earnings: Some(dec!(50.0)),
        tips: Some(dec!(10.0)),
        gas_cost: Some(dec!(5.0)),
        notes: None,
    };
    let ended_shift = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await
    .unwrap()
    .0;

    // Try to set end_time before start_time
    let invalid_end_time = ended_shift.start_time - chrono::Duration::hours(1);

    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: None,
        end_time: Some(invalid_end_time.to_rfc3339()),
    };

    let result = update_shift(
        State(state),
        axum::extract::Path(shift_id),
        Json(update_request),
    )
    .await;

    // Should return error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_shift_invalid_start_after_end() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Start and end a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 10100,
        earnings: Some(dec!(50.0)),
        tips: Some(dec!(10.0)),
        gas_cost: Some(dec!(5.0)),
        notes: None,
    };
    let ended_shift = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await
    .unwrap()
    .0;

    // Try to set start_time after end_time
    let invalid_start_time = ended_shift.end_time.unwrap() + chrono::Duration::hours(1);

    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: Some(invalid_start_time.to_rfc3339()),
        end_time: None,
    };

    let result = update_shift(
        State(state),
        axum::extract::Path(shift_id),
        Json(update_request),
    )
    .await;

    // Should return error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_shift_invalid_datetime_format() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Start a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();

    // Try to update with invalid datetime format
    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: Some("not-a-valid-datetime".to_string()),
        end_time: None,
    };

    let result = update_shift(
        State(state),
        axum::extract::Path(shift_id),
        Json(update_request),
    )
    .await;

    // Should return error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_shift_time_recalculates_hourly_pay() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Start and end a shift
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift_id = shift.id.id.to_string();
    let start_time = shift.start_time;

    let end_request = EndShiftRequest {
        odometer_end: 10100,
        earnings: Some(dec!(120.0)),
        tips: Some(dec!(30.0)),
        gas_cost: Some(dec!(10.0)),
        notes: None,
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift_id.clone()),
        Json(end_request),
    )
    .await
    .unwrap();

    // Update to create exactly 4 hour shift
    let new_end_time = start_time + chrono::Duration::hours(4);

    let update_request = UpdateShiftRequest {
        odometer_start: None,
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        start_time: None,
        end_time: Some(new_end_time.to_rfc3339()),
    };

    let result = update_shift(
        State(state),
        axum::extract::Path(shift_id),
        Json(update_request),
    )
    .await;

    assert!(result.is_ok());
    let updated_shift = result.unwrap().0;

    // Verify hours_worked is 4
    assert_eq!(updated_shift.hours_worked.unwrap(), dec!(4.0));

    // Verify hourly_pay: (120 + 30 - 10) / 4 = 35.00
    assert_eq!(updated_shift.hourly_pay.unwrap(), dec!(35.0));
}

#[tokio::test]
async fn test_export_csv_all_shifts() {
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Create multiple shifts
    for i in 0..3 {
        let start_request = StartShiftRequest {
            odometer_start: 10000 + (i * 100),
        };
        let shift = start_shift(State(state.clone()), Json(start_request))
            .await
            .unwrap()
            .0;
        let shift_id = shift.id.id.to_string();

        let end_request = EndShiftRequest {
            odometer_end: 10100 + (i * 100),
            earnings: Some(dec!(100.0)),
            tips: Some(dec!(20.0)),
            gas_cost: Some(dec!(15.0)),
            notes: Some(format!("Shift {}", i)),
        };
        let _ = end_shift(
            State(state.clone()),
            axum::extract::Path(shift_id),
            Json(end_request),
        )
        .await;
    }

    // Export all shifts (no query parameters)
    let params = OptionalDateRangeQuery {
        start: None,
        end: None,
    };
    let result = export_csv(State(state), axum::extract::Query(params)).await;
    assert!(result.is_ok());

    // Convert to response and extract CSV content
    let response = result.unwrap().into_response();
    let status = response.status();
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    let csv = String::from_utf8(bytes.to_vec()).unwrap();

    // Verify status code
    assert_eq!(status, axum::http::StatusCode::OK);

    // Verify CSV content
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 4); // Header + 3 shifts

    // Verify header row
    assert!(lines[0].contains("ID,Start Time,End Time"));
    assert!(lines[0].contains("Hours Worked"));
    assert!(lines[0].contains("Odometer Start,Odometer End"));
    assert!(lines[0].contains("Earnings,Tips,Gas Cost"));

    // Verify each shift is in the CSV
    assert!(csv.contains("Shift 0"));
    assert!(csv.contains("Shift 1"));
    assert!(csv.contains("Shift 2"));

    // Verify monetary values are present
    assert!(csv.contains("100")); // earnings
    assert!(csv.contains("20")); // tips
    assert!(csv.contains("15")); // gas cost
    assert!(csv.contains("105")); // day total (100 + 20 - 15)
}

#[tokio::test]
async fn test_export_csv_with_date_range() {
    use axum::response::IntoResponse;
    use chrono::Utc;
    use http_body_util::BodyExt;

    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    // Create shifts with specific times
    let base_time = Utc::now();

    // Shift 1: 2 days ago (start and END it)
    let start_request = StartShiftRequest {
        odometer_start: 10000,
    };
    let shift1 = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift1_id = shift1.id.id.to_string();

    // End shift1 first
    let end_request1 = EndShiftRequest {
        odometer_end: 10100,
        earnings: Some(dec!(50.0)),
        tips: None,
        gas_cost: None,
        notes: Some("Old shift".to_string()),
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift1_id.clone()),
        Json(end_request1),
    )
    .await;

    // Now update shift1's start time to 2 days ago
    let old_start = (base_time - chrono::Duration::days(2)).to_rfc3339();
    let update1 = UpdateShiftRequest {
        start_time: Some(old_start),
        odometer_end: None,
        earnings: None,
        tips: None,
        gas_cost: None,
        notes: None,
        end_time: None,
        odometer_start: None,
    };
    let _ = update_shift(
        State(state.clone()),
        axum::extract::Path(shift1_id),
        Json(update1),
    )
    .await;

    // Shift 2: today
    let start_request = StartShiftRequest {
        odometer_start: 10100,
    };
    let shift2 = start_shift(State(state.clone()), Json(start_request))
        .await
        .unwrap()
        .0;
    let shift2_id = shift2.id.id.to_string();

    let end_request = EndShiftRequest {
        odometer_end: 10200,
        earnings: Some(dec!(100.0)),
        tips: Some(dec!(20.0)),
        gas_cost: Some(dec!(15.0)),
        notes: Some("Recent shift".to_string()),
    };
    let _ = end_shift(
        State(state.clone()),
        axum::extract::Path(shift2_id),
        Json(end_request),
    )
    .await;

    // Export only today's shifts
    let start_of_today = (base_time - chrono::Duration::hours(12)).to_rfc3339();
    let end_of_today = (base_time + chrono::Duration::hours(12)).to_rfc3339();

    let params = OptionalDateRangeQuery {
        start: Some(start_of_today),
        end: Some(end_of_today),
    };
    let result = export_csv(State(state), axum::extract::Query(params)).await;
    assert!(result.is_ok());

    // Convert to response and extract CSV content
    let response = result.unwrap().into_response();
    let status = response.status();
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    let csv = String::from_utf8(bytes.to_vec()).unwrap();

    // Verify status code
    assert_eq!(status, axum::http::StatusCode::OK);

    // Verify CSV content
    let lines: Vec<&str> = csv.lines().collect();

    // Should only have header + 1 shift (the recent one)
    assert_eq!(lines.len(), 2);

    // Verify only the recent shift is included
    assert!(csv.contains("Recent shift"));
    assert!(!csv.contains("Old shift"));

    // Verify the recent shift's data
    assert!(csv.contains("10100")); // odometer start
    assert!(csv.contains("10200")); // odometer end
    assert!(csv.contains("100")); // earnings
}

#[tokio::test]
async fn test_export_csv_empty_database() {
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let params = OptionalDateRangeQuery {
        start: None,
        end: None,
    };
    let result = export_csv(State(state), axum::extract::Query(params)).await;
    assert!(result.is_ok());

    // Convert to response and extract CSV content
    let response = result.unwrap().into_response();
    let status = response.status();
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    let csv = String::from_utf8(bytes.to_vec()).unwrap();

    // Verify status code
    assert_eq!(status, axum::http::StatusCode::OK);

    // Verify CSV content
    let lines: Vec<&str> = csv.lines().collect();

    // Should only have header
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("ID,Start Time,End Time"));
    assert!(lines[0].contains("Hours Worked"));
    assert!(lines[0].contains("Odometer Start,Odometer End"));

    // Verify no data rows
    assert!(!csv.contains("10000")); // No odometer readings
    assert!(!csv.contains("100.0")); // No earnings
}

#[tokio::test]
async fn test_export_csv_invalid_date_format() {
    let db = common::setup_test_db().await;
    let state = Arc::new(AppState { db });

    let params = OptionalDateRangeQuery {
        start: Some("invalid-date".to_string()),
        end: Some("2025-12-31T23:59:59Z".to_string()),
    };
    let result = export_csv(State(state), axum::extract::Query(params)).await;

    // Verify the export fails with invalid date format
    assert!(result.is_err());
}
