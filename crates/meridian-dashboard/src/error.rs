//! Error types for the dashboard system

use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Result type alias for dashboard operations
pub type Result<T> = std::result::Result<T, DashboardError>;

/// Dashboard error types
#[derive(Error, Debug)]
pub enum DashboardError {
    #[error("Dashboard not found: {0}")]
    DashboardNotFound(uuid::Uuid),

    #[error("Widget not found: {0}")]
    WidgetNotFound(uuid::Uuid),

    #[error("Report not found: {0}")]
    ReportNotFound(uuid::Uuid),

    #[error("Invalid dashboard configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Invalid widget configuration: {0}")]
    InvalidWidgetConfig(String),

    #[error("Invalid layout: {0}")]
    InvalidLayout(String),

    #[error("Data source error: {0}")]
    DataSourceError(String),

    #[error("Query execution failed: {0}")]
    QueryExecutionError(String),

    #[error("Export failed: {0}")]
    ExportError(String),

    #[error("Schedule error: {0}")]
    ScheduleError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("PDF generation error: {0}")]
    PdfError(String),

    #[error("Excel generation error: {0}")]
    ExcelError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for DashboardError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            DashboardError::DashboardNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            DashboardError::WidgetNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            DashboardError::ReportNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            DashboardError::InvalidConfiguration(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            DashboardError::InvalidWidgetConfig(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            DashboardError::InvalidLayout(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            DashboardError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            DashboardError::PermissionDenied(_) => (StatusCode::FORBIDDEN, self.to_string()),
            DashboardError::DataSourceError(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            DashboardError::QueryExecutionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DashboardError::ExportError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DashboardError::ScheduleError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DashboardError::PdfError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DashboardError::ExcelError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DashboardError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            DashboardError::SerializationError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error".to_string()),
            DashboardError::HttpError(_) => (StatusCode::BAD_GATEWAY, "External service error".to_string()),
            DashboardError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string()),
            DashboardError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_error_display() {
        let id = Uuid::new_v4();
        let error = DashboardError::DashboardNotFound(id);
        assert!(error.to_string().contains(&id.to_string()));
    }

    #[test]
    fn test_validation_error() {
        let error = DashboardError::ValidationError("Invalid input".to_string());
        assert_eq!(error.to_string(), "Validation error: Invalid input");
    }
}
