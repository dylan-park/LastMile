use crate::handlers::shifts::{
    end_shift, export_csv, get_active_shift, get_all_shifts, start_shift, update_shift,
};
use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post, put};
use sqlx::mysql::MySqlPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
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

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost/uber_eats_tracker".to_string());

    info!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    info!("Setting up database schema...");
    db::setup_database(&pool).await;

    let state = Arc::new(AppState { db: pool });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/shifts", get(get_all_shifts))
        .route("/api/shifts/active", get(get_active_shift))
        .route("/api/shifts/start", post(start_shift))
        .route("/api/shifts/{id}/end", post(end_shift))
        .route("/api/shifts/{id}", put(update_shift))
        .route("/api/shifts/export", get(export_csv))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
