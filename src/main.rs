use std::{
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    Router,
    body::Body,
    http::{
        Request, Response,
        header::{CACHE_CONTROL, HeaderValue},
    },
    routing::{delete, get, post, put},
};
use futures::{FutureExt, future::BoxFuture};
use http_body::Body as HttpBody;
use lastmile::{db, handlers, state};
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};
use tower::{Layer, Service};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;

use crate::{
    handlers::{
        maintenance::{
            calculate_required_maintenance, create_maintenance_item, delete_maintenance_item,
            get_all_maintenance_items, update_maintenance_item,
        },
        shifts::{
            delete_shift, end_shift, export_csv, get_active_shift, get_all_shifts,
            get_shifts_by_range, start_shift, update_shift,
        },
    },
    state::AppState,
};

// Custom Cache layer
#[derive(Clone)]
pub struct CacheControlLayer;

impl<S> Layer<S> for CacheControlLayer {
    type Service = CacheControlMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheControlMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct CacheControlMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<Body>> for CacheControlMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: 'static,
    B: HttpBody + Send + 'static, // ✅ Generic body support here
{
    type Response = Response<B>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let path = req.uri().path().to_lowercase();

        async move {
            let mut res = inner.call(req).await?;

            let value = if path.ends_with(".html") {
                "no-cache"
            } else {
                "public, max-age=604800" // 7 days
            };

            res.headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static(value));

            Ok(res)
        }
        .boxed()
    }
}

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
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

    info!("Initializing SurrealDB at {}", db_path);

    // Create the data directory if it doesn't exist
    std::fs::create_dir_all(&db_path).expect("Failed to create data directory");

    // Initialize SurrealDB with RocksDB backend
    let db: Surreal<Db> = Surreal::new::<RocksDb>(db_path.clone())
        .await
        .expect("Failed to initialize SurrealDB");

    // Use namespace and database
    db.use_ns("lastmile")
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

    let static_files = CacheControlLayer.layer(ServeDir::new("static"));

    let app = Router::new()
        // API routes
        // Shifts
        .route("/api/shifts", get(get_all_shifts))
        .route("/api/shifts/range", get(get_shifts_by_range))
        .route("/api/shifts/active", get(get_active_shift))
        .route("/api/shifts/start", post(start_shift))
        .route("/api/shifts/{id}/end", post(end_shift))
        .route("/api/shifts/{id}", put(update_shift))
        .route("/api/shifts/{id}", delete(delete_shift))
        .route("/api/shifts/export", get(export_csv))
        // Maintenance
        .route("/api/maintenance", get(get_all_maintenance_items))
        .route("/api/maintenance/create", post(create_maintenance_item))
        .route("/api/maintenance/{id}", put(update_maintenance_item))
        .route("/api/maintenance/{id}", delete(delete_maintenance_item))
        .route(
            "/api/maintenance/calculate",
            get(calculate_required_maintenance),
        )
        .with_state(state)
        .layer(cors)
        // Serve static files from ./static directory as fallback
        .fallback_service(static_files)
        .layer(CompressionLayer::new());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("Failed to bind port");

    info!("Server running on http://0.0.0.0:{}", port);
    info!("Database location: {}", db_path);
    info!("Note: For database management, use SurrealDB CLI:");
    info!(
        "  surreal sql --endpoint file://{} --namespace lastmile --database main",
        db_path
    );
    axum::serve(listener, app).await.unwrap();
}
