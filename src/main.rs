use crate::{
    handlers::shifts::{
        end_shift, export_csv, get_active_shift, get_all_shifts, get_shifts_by_range, start_shift,
        update_shift,
    },
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post, put},
};
use std::sync::Arc;
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;

mod calculations;
mod db;
mod error;
mod handlers;
mod models;
mod state;
mod validation;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    // Get configuration from environment or use defaults
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data".to_string());

    info!("Initializing SurrealDB at {}", db_path);

    // Create the data directory if it doesn't exist
    std::fs::create_dir_all(&db_path).expect("Failed to create data directory");

    // Initialize SurrealDB with RocksDB backend
    let db: Surreal<Db> = Surreal::new::<RocksDb>(db_path.clone())
        .await
        .expect("Failed to initialize SurrealDB");

    // Use namespace and database
    db.use_ns("uber_eats_tracker")
        .use_db("main")
        .await
        .expect("Failed to use namespace and database");

    info!("Setting up database schema...");
    db::setup_database(&db).await;

    let state = Arc::new(AppState { db });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // API routes
        .route("/api/shifts", get(get_all_shifts))
        .route("/api/shifts/range", get(get_shifts_by_range))
        .route("/api/shifts/active", get(get_active_shift))
        .route("/api/shifts/start", post(start_shift))
        .route("/api/shifts/{id}/end", post(end_shift))
        .route("/api/shifts/{id}", put(update_shift))
        .route("/api/shifts/export", get(export_csv))
        .with_state(state)
        .layer(cors)
        // Serve static files from ./static directory as fallback
        .fallback_service(ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Server running on http://0.0.0.0:3000");
    info!("Database location: {}", db_path);
    info!("Note: For database management, use SurrealDB CLI:");
    info!(
        "  surreal sql --endpoint file://{} --namespace uber_eats_tracker --database main",
        db_path
    );
    axum::serve(listener, app).await.unwrap();
}
