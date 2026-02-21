use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::state::AppState;

pub async fn health_check(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let health_data = state.db_provider.check_health().await;

    let status_code = if health_data.get("status").and_then(|s| s.as_str()) == Some("ok") {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health_data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, DbProvider, DemoDbProvider, SingleDbProvider};
    use axum::extract::Extension;
    use surrealdb::{Surreal, engine::local::Mem};

    #[tokio::test]
    async fn test_health_check_persistent_mode() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        let provider = SingleDbProvider { db };
        let app_state = Arc::new(AppState {
            db_provider: Arc::new(DbProvider::Single(provider)),
            is_demo_mode: false,
        });

        let response = health_check(Extension(app_state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        use http_body_util::BodyExt;
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["status"], "ok");
        assert_eq!(body_json["mode"], "persistent");
        assert!(body_json.get("db_version").is_some());
    }

    #[tokio::test]
    async fn test_health_check_demo_mode() {
        let provider = DemoDbProvider::new();
        let app_state = Arc::new(AppState {
            db_provider: Arc::new(DbProvider::Demo(provider)),
            is_demo_mode: true,
        });

        // Use the handler
        let response = health_check(Extension(app_state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        use http_body_util::BodyExt;
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["status"], "ok");
        assert_eq!(body_json["mode"], "demo");
        assert_eq!(body_json["active_sessions"], 0);
    }
}
