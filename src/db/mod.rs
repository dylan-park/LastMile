pub mod helpers;

use sqlx::{MySql, Pool};
use tracing::info;

pub async fn setup_database(pool: &Pool<MySql>) {
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
