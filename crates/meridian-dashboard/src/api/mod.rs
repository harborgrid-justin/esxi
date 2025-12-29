//! API module for dashboard HTTP endpoints

pub mod dashboard;
pub mod widget;
pub mod export;

use axum::{
    Router,
    routing::{get, post, put, delete},
};

/// Create the dashboard API router
pub fn create_router() -> Router {
    Router::new()
        // Dashboard routes
        .route("/dashboards", get(dashboard::list_dashboards))
        .route("/dashboards", post(dashboard::create_dashboard))
        .route("/dashboards/:id", get(dashboard::get_dashboard))
        .route("/dashboards/:id", put(dashboard::update_dashboard))
        .route("/dashboards/:id", delete(dashboard::delete_dashboard))
        .route("/dashboards/:id/clone", post(dashboard::clone_dashboard))

        // Widget routes
        .route("/dashboards/:dashboard_id/widgets", get(widget::list_widgets))
        .route("/dashboards/:dashboard_id/widgets", post(widget::create_widget))
        .route("/widgets/:id", get(widget::get_widget))
        .route("/widgets/:id", put(widget::update_widget))
        .route("/widgets/:id", delete(widget::delete_widget))
        .route("/widgets/:id/data", get(widget::get_widget_data))
        .route("/widgets/:id/refresh", post(widget::refresh_widget))

        // Export routes
        .route("/dashboards/:id/export/pdf", post(export::export_pdf))
        .route("/dashboards/:id/export/excel", post(export::export_excel))
        .route("/widgets/:id/export/csv", post(export::export_csv))

        // Report routes
        .route("/reports", get(export::list_reports))
        .route("/reports", post(export::create_report))
        .route("/reports/:id", get(export::get_report))
        .route("/reports/:id", put(export::update_report))
        .route("/reports/:id", delete(export::delete_report))
        .route("/reports/:id/generate", post(export::generate_report))
}
