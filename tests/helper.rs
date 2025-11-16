use chrono::Utc;
use rust_decimal_macros::dec;

use lastmile::db::helpers::*;
use lastmile::models::*;

mod common;

#[tokio::test]
async fn test_query_shifts_empty() {
    let db = common::setup_test_db().await;

    let shifts = query_shifts(&db, "SELECT * FROM shifts ORDER BY start_time DESC")
        .await
        .unwrap();

    assert_eq!(shifts.len(), 0);
}

#[tokio::test]
async fn test_query_shifts_with_data() {
    let db = common::setup_test_db().await;

    let now = Utc::now();
    let record1 = ShiftRecord {
        start_time: now.into(),
        end_time: None,
        hours_worked: None,
        odometer_start: 1000,
        odometer_end: None,
        miles_driven: None,
        earnings: dec!(100.0),
        tips: dec!(20.0),
        gas_cost: dec!(15.0),
        day_total: dec!(105.0),
        hourly_pay: None,
        notes: None,
    };

    let _: Option<Shift> = db.create("shifts").content(record1).await.unwrap();

    let shifts = query_shifts(&db, "SELECT * FROM shifts ORDER BY start_time DESC")
        .await
        .unwrap();

    assert_eq!(shifts.len(), 1);
    assert_eq!(shifts[0].odometer_start, 1000);
}

#[tokio::test]
async fn test_query_shifts_with_date_range() {
    let db = common::setup_test_db().await;

    let start_time = chrono::Utc::now() - chrono::Duration::days(10);
    let mid_time = chrono::Utc::now() - chrono::Duration::days(5);

    let record1 = ShiftRecord {
        start_time: start_time.into(),
        end_time: None,
        hours_worked: None,
        odometer_start: 1000,
        odometer_end: None,
        miles_driven: None,
        earnings: dec!(100.0),
        tips: dec!(20.0),
        gas_cost: dec!(15.0),
        day_total: dec!(105.0),
        hourly_pay: None,
        notes: None,
    };

    let record2 = ShiftRecord {
        start_time: mid_time.into(),
        end_time: None,
        hours_worked: None,
        odometer_start: 1100,
        odometer_end: None,
        miles_driven: None,
        earnings: dec!(150.0),
        tips: dec!(30.0),
        gas_cost: dec!(20.0),
        day_total: dec!(160.0),
        hourly_pay: None,
        notes: None,
    };

    let _: Option<Shift> = db.create("shifts").content(record1).await.unwrap();
    let _: Option<Shift> = db.create("shifts").content(record2).await.unwrap();

    let query_start = (start_time - chrono::Duration::days(1)).into();
    let query_end = (mid_time + chrono::Duration::days(1)).into();

    let query = "SELECT * FROM shifts WHERE start_time >= $start AND start_time <= $end ORDER BY start_time DESC";
    let shifts = query_shifts_with_date_range(&db, query, query_start, query_end)
        .await
        .unwrap();

    assert_eq!(shifts.len(), 2);
}

#[tokio::test]
async fn test_has_active_shift_false() {
    let db = common::setup_test_db().await;

    let has_active = has_active_shift(&db).await.unwrap();
    assert!(!has_active);
}

#[tokio::test]
async fn test_has_active_shift_true() {
    let db = common::setup_test_db().await;

    let now = Utc::now();
    let record = ShiftRecord {
        start_time: now.into(),
        end_time: None,
        hours_worked: None,
        odometer_start: 1000,
        odometer_end: None,
        miles_driven: None,
        earnings: dec!(0.0),
        tips: dec!(0.0),
        gas_cost: dec!(0.0),
        day_total: dec!(0.0),
        hourly_pay: None,
        notes: None,
    };

    let _: Option<Shift> = db.create("shifts").content(record).await.unwrap();

    let has_active = has_active_shift(&db).await.unwrap();
    assert!(has_active);
}

#[tokio::test]
async fn test_has_active_shift_only_ended_shifts() {
    let db = common::setup_test_db().await;

    let now = Utc::now();
    let record = ShiftRecord {
        start_time: now.into(),
        end_time: Some(now.into()),
        hours_worked: Some(dec!(1.0)),
        odometer_start: 1000,
        odometer_end: Some(1050),
        miles_driven: Some(50),
        earnings: dec!(100.0),
        tips: dec!(20.0),
        gas_cost: dec!(15.0),
        day_total: dec!(105.0),
        hourly_pay: Some(dec!(105.0)),
        notes: None,
    };

    let _: Option<Shift> = db.create("shifts").content(record).await.unwrap();

    let has_active = has_active_shift(&db).await.unwrap();
    assert!(!has_active);
}

#[tokio::test]
async fn test_get_shift_by_id_found() {
    let db = common::setup_test_db().await;

    let now = Utc::now();
    let record = ShiftRecord {
        start_time: now.into(),
        end_time: None,
        hours_worked: None,
        odometer_start: 1234,
        odometer_end: None,
        miles_driven: None,
        earnings: dec!(0.0),
        tips: dec!(0.0),
        gas_cost: dec!(0.0),
        day_total: dec!(0.0),
        hourly_pay: None,
        notes: None,
    };

    let created: Option<Shift> = db.create("shifts").content(record).await.unwrap();
    let created = created.unwrap();
    let id = created.id.id.to_string();

    let shift = get_shift_by_id(&db, &id).await.unwrap();
    assert_eq!(shift.odometer_start, 1234);
}

#[tokio::test]
async fn test_get_shift_by_id_not_found() {
    let db = common::setup_test_db().await;

    let result = get_shift_by_id(&db, "nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        lastmile::error::AppError::ShiftNotFound
    ));
}

#[tokio::test]
async fn test_query_maintenance_items_empty() {
    let db = common::setup_test_db().await;

    let items = query_maitenance_items(&db, "SELECT * FROM maintenance")
        .await
        .unwrap();

    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_query_maintenance_items_with_data() {
    let db = common::setup_test_db().await;

    let record = MaintenanceItemRecord {
        name: "Oil Change".to_string(),
        mileage_interval: 3000,
        last_service_mileage: 10000,
        enabled: true,
        notes: None,
    };

    let _: Option<MaintenanceItem> = db.create("maintenance").content(record).await.unwrap();

    let items = query_maitenance_items(&db, "SELECT * FROM maintenance")
        .await
        .unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "Oil Change");
    assert_eq!(items[0].mileage_interval, 3000);
}

#[tokio::test]
async fn test_get_maintenance_item_by_id_found() {
    let db = common::setup_test_db().await;

    let record = MaintenanceItemRecord {
        name: "Tire Rotation".to_string(),
        mileage_interval: 5000,
        last_service_mileage: 15000,
        enabled: true,
        notes: Some("Every 5k miles".to_string()),
    };

    let created: Option<MaintenanceItem> = db.create("maintenance").content(record).await.unwrap();
    let created = created.unwrap();
    let id = created.id.id.to_string();

    let item = get_maitenance_item_by_id(&db, &id).await.unwrap();
    assert_eq!(item.name, "Tire Rotation");
    assert_eq!(item.notes, Some("Every 5k miles".to_string()));
}

#[tokio::test]
async fn test_get_maintenance_item_by_id_not_found() {
    let db = common::setup_test_db().await;

    let result = get_maitenance_item_by_id(&db, "nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        lastmile::error::AppError::MaintenanceItemNotFound
    ));
}
