use axum::Json;
use axum::extract::State;
use rust_decimal_macros::dec;
use std::sync::Arc;

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
