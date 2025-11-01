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
    let surreal_user = std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string());
    // let surreal_pass = std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string());
    let surreal_bind = std::env::var("SURREAL_BIND").unwrap_or_else(|_| "0.0.0.0:8000".to_string());

    info!("Initializing SurrealDB at {}", db_path);

    // Create the data directory if it doesn't exist
    std::fs::create_dir_all(&db_path).expect("Failed to create data directory");

    // Initialize SurrealDB with RocksDB backend
    let db: Surreal<Db> = Surreal::new::<RocksDb>(db_path.clone())
        .await
        .expect("Failed to initialize SurrealDB");

    // Note: Embedded RocksDB mode doesn't use authentication
    // Authentication is only for remote server connections
    info!(
        "Note: Authentication configured as {} (for reference only)",
        surreal_user
    );

    // Use namespace and database
    db.use_ns("uber_eats_tracker")
        .use_db("main")
        .await
        .expect("Failed to use namespace and database");

    info!("Setting up database schema...");
    db::setup_database(&db).await;

    // Start SurrealDB server for remote access
    let bind_addr = surreal_bind.clone();
    tokio::spawn(async move {
        info!("Starting SurrealDB server on {}", bind_addr);
        // Note: In-process server with remote access requires running a separate SurrealDB instance
        // For now, the local RocksDB instance is accessible only through the application
        info!(
            "SurrealDB is running in embedded mode. For remote access, use a separate SurrealDB server instance."
        );
    });

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
    info!("Note: For remote database access, use SurrealDB CLI:");
    info!(
        "  surreal sql --endpoint file://{} --namespace uber_eats_tracker --database main",
        db_path
    );
    axum::serve(listener, app).await.unwrap();
}
