use axum::{Json, http::StatusCode, response::IntoResponse};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] Box<surrealdb::Error>),

    #[error("Shift not found")]
    ShiftNotFound,

    #[error("Maintenance Item not found")]
    MaintenanceItemNotFound,

    #[error("Active shift already exists")]
    ActiveShiftExists,

    #[error(
        "Invalid odometer reading: end ({end}) must be greater than or equal to start ({start})"
    )]
    InvalidOdometer { start: i32, end: i32 },

    #[error("Invalid monetary value: {0} must be non-negative")]
    InvalidMonetaryValue(String),
}

// Helper conversion to avoid .map_err(Box::new) everywhere
impl From<surrealdb::Error> for AppError {
    fn from(err: surrealdb::Error) -> Self {
        AppError::Database(Box::new(err))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Database(e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::ShiftNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::MaintenanceItemNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::ActiveShiftExists => (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidOdometer { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidMonetaryValue(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
