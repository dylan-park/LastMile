pub mod helpers;

use surrealdb::{Surreal, engine::local::Db};
use tracing::info;

pub async fn setup_database(db: &Surreal<Db>) {
    // Define the shifts table with schema
    let schema = r#"
        DEFINE TABLE shifts SCHEMAFULL;

        DEFINE FIELD start_time ON shifts TYPE datetime;
        DEFINE FIELD end_time ON shifts TYPE option<datetime>;
        DEFINE FIELD hours_worked ON shifts TYPE option<decimal>;
        DEFINE FIELD odometer_start ON shifts TYPE int;
        DEFINE FIELD odometer_end ON shifts TYPE option<int>;
        DEFINE FIELD miles_driven ON shifts TYPE option<int>;
        DEFINE FIELD earnings ON shifts TYPE decimal DEFAULT 0.00;
        DEFINE FIELD tips ON shifts TYPE decimal DEFAULT 0.00;
        DEFINE FIELD gas_cost ON shifts TYPE decimal DEFAULT 0.00;
        DEFINE FIELD day_total ON shifts TYPE decimal DEFAULT 0.00;
        DEFINE FIELD hourly_pay ON shifts TYPE option<decimal>;
        DEFINE FIELD notes ON shifts TYPE option<string>;

        DEFINE INDEX idx_start_time ON shifts FIELDS start_time;
        DEFINE INDEX idx_end_time ON shifts FIELDS end_time;
    "#;

    for statement in schema.trim().split(';').filter(|s| !s.trim().is_empty()) {
        if let Err(e) = db.query(statement).await {
            info!("Schema statement (might already exist): {}", e);
        }
    }

    info!("Database schema ready");
}
