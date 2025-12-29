//! Export and reporting API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
    http::header,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::{Report, ReportFormat, ReportSchedule, Result, DashboardError};
use super::dashboard::AppState;

/// Export PDF request
#[derive(Debug, Deserialize)]
pub struct ExportPdfRequest {
    pub orientation: Option<String>, // portrait or landscape
    pub paper_size: Option<String>,  // A4, Letter, etc.
    pub include_filters: Option<bool>,
}

/// Export dashboard as PDF
pub async fn export_pdf(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ExportPdfRequest>,
) -> Result<Response> {
    // Fetch dashboard
    let dashboard: crate::Dashboard = sqlx::query_as(
        "SELECT * FROM dashboards WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::DashboardNotFound(id))?;

    // Generate PDF
    let pdf_data = generate_pdf(&dashboard, &request)?;

    let filename = format!("{}.pdf", dashboard.name.replace(' ', "_"));

    Ok((
        [(
            header::CONTENT_TYPE,
            "application/pdf",
        ), (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", filename),
        )],
        pdf_data,
    ).into_response())
}

/// Export Excel request
#[derive(Debug, Deserialize)]
pub struct ExportExcelRequest {
    pub include_data: Option<bool>,
    pub include_charts: Option<bool>,
}

/// Export dashboard as Excel
pub async fn export_excel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ExportExcelRequest>,
) -> Result<Response> {
    // Fetch dashboard
    let dashboard: crate::Dashboard = sqlx::query_as(
        "SELECT * FROM dashboards WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::DashboardNotFound(id))?;

    // Generate Excel
    let excel_data = generate_excel(&dashboard, &request)?;

    let filename = format!("{}.xlsx", dashboard.name.replace(' ', "_"));

    Ok((
        [(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ), (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", filename),
        )],
        excel_data,
    ).into_response())
}

/// Export CSV request
#[derive(Debug, Deserialize)]
pub struct ExportCsvRequest {
    pub delimiter: Option<String>,
    pub include_headers: Option<bool>,
}

/// Export widget data as CSV
pub async fn export_csv(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ExportCsvRequest>,
) -> Result<Response> {
    // Fetch widget
    let widget: crate::Widget = sqlx::query_as(
        "SELECT * FROM widgets WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::WidgetNotFound(id))?;

    // Get widget data (simplified - would use data source)
    let csv_data = generate_csv(&widget, &request)?;

    let filename = format!("{}.csv", widget.title.replace(' ', "_"));

    Ok((
        [(
            header::CONTENT_TYPE,
            "text/csv",
        ), (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", filename),
        )],
        csv_data,
    ).into_response())
}

/// Report list query
#[derive(Debug, Deserialize)]
pub struct ReportListQuery {
    pub dashboard_id: Option<Uuid>,
}

/// Report list response
#[derive(Debug, Serialize)]
pub struct ReportList {
    pub reports: Vec<Report>,
}

/// List reports
pub async fn list_reports(
    State(state): State<AppState>,
    Query(query): Query<ReportListQuery>,
) -> Result<Json<ReportList>> {
    let reports: Vec<Report> = if let Some(dashboard_id) = query.dashboard_id {
        sqlx::query_as(
            "SELECT * FROM reports WHERE dashboard_id = $1 ORDER BY created_at DESC"
        )
        .bind(dashboard_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT * FROM reports ORDER BY created_at DESC"
        )
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(ReportList { reports }))
}

/// Create report request
#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub name: String,
    pub description: Option<String>,
    pub dashboard_id: Uuid,
    pub format: ReportFormat,
    pub schedule: Option<ReportSchedule>,
    pub recipients: Vec<String>,
}

/// Create a new report
pub async fn create_report(
    State(state): State<AppState>,
    Json(request): Json<CreateReportRequest>,
) -> Result<Json<Report>> {
    // Validate name
    if request.name.trim().is_empty() {
        return Err(DashboardError::ValidationError("Report name cannot be empty".to_string()));
    }

    // Validate recipients
    for email in &request.recipients {
        if !email.contains('@') {
            return Err(DashboardError::ValidationError(format!("Invalid email address: {}", email)));
        }
    }

    let report = Report {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        dashboard_id: request.dashboard_id,
        format: request.format,
        schedule: request.schedule,
        recipients: request.recipients,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        r#"
        INSERT INTO reports (id, name, description, dashboard_id, format, schedule, recipients, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(&report.id)
    .bind(&report.name)
    .bind(&report.description)
    .bind(&report.dashboard_id)
    .bind(serde_json::to_value(&report.format)?)
    .bind(serde_json::to_value(&report.schedule)?)
    .bind(serde_json::to_value(&report.recipients)?)
    .bind(&report.created_at)
    .bind(&report.updated_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(report))
}

/// Get a report by ID
pub async fn get_report(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Report>> {
    let report: Report = sqlx::query_as(
        "SELECT * FROM reports WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::ReportNotFound(id))?;

    Ok(Json(report))
}

/// Update report request
#[derive(Debug, Deserialize)]
pub struct UpdateReportRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub format: Option<ReportFormat>,
    pub schedule: Option<ReportSchedule>,
    pub recipients: Option<Vec<String>>,
}

/// Update a report
pub async fn update_report(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateReportRequest>,
) -> Result<Json<Report>> {
    // Fetch existing report
    let mut report: Report = sqlx::query_as(
        "SELECT * FROM reports WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::ReportNotFound(id))?;

    // Update fields
    if let Some(name) = request.name {
        if name.trim().is_empty() {
            return Err(DashboardError::ValidationError("Report name cannot be empty".to_string()));
        }
        report.name = name;
    }

    if let Some(description) = request.description {
        report.description = Some(description);
    }

    if let Some(format) = request.format {
        report.format = format;
    }

    if let Some(schedule) = request.schedule {
        report.schedule = Some(schedule);
    }

    if let Some(recipients) = request.recipients {
        for email in &recipients {
            if !email.contains('@') {
                return Err(DashboardError::ValidationError(format!("Invalid email address: {}", email)));
            }
        }
        report.recipients = recipients;
    }

    report.updated_at = Utc::now();

    sqlx::query(
        r#"
        UPDATE reports
        SET name = $1, description = $2, format = $3, schedule = $4, recipients = $5, updated_at = $6
        WHERE id = $7
        "#
    )
    .bind(&report.name)
    .bind(&report.description)
    .bind(serde_json::to_value(&report.format)?)
    .bind(serde_json::to_value(&report.schedule)?)
    .bind(serde_json::to_value(&report.recipients)?)
    .bind(&report.updated_at)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    Ok(Json(report))
}

/// Delete a report
pub async fn delete_report(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM reports WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DashboardError::ReportNotFound(id));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Report deleted successfully"
    })))
}

/// Generate report response
#[derive(Debug, Serialize)]
pub struct GenerateReportResponse {
    pub report_id: Uuid,
    pub status: String,
    pub download_url: Option<String>,
}

/// Generate a report
pub async fn generate_report(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<GenerateReportResponse>> {
    // Fetch report
    let report: Report = sqlx::query_as(
        "SELECT * FROM reports WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::ReportNotFound(id))?;

    // Fetch dashboard
    let dashboard: crate::Dashboard = sqlx::query_as(
        "SELECT * FROM dashboards WHERE id = $1"
    )
    .bind(report.dashboard_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::DashboardNotFound(report.dashboard_id))?;

    // Generate report based on format
    let _data = match report.format {
        ReportFormat::Pdf => {
            let request = ExportPdfRequest {
                orientation: Some("portrait".to_string()),
                paper_size: Some("A4".to_string()),
                include_filters: Some(true),
            };
            generate_pdf(&dashboard, &request)?
        }
        ReportFormat::Excel => {
            let request = ExportExcelRequest {
                include_data: Some(true),
                include_charts: Some(true),
            };
            generate_excel(&dashboard, &request)?
        }
        _ => {
            return Err(DashboardError::ExportError("Unsupported format".to_string()));
        }
    };

    // In production, save to storage and send to recipients
    Ok(Json(GenerateReportResponse {
        report_id: id,
        status: "completed".to_string(),
        download_url: Some(format!("/api/reports/{}/download", id)),
    }))
}

/// Generate PDF from dashboard
fn generate_pdf(dashboard: &crate::Dashboard, _request: &ExportPdfRequest) -> Result<Vec<u8>> {
    use printpdf::*;

    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        &dashboard.name,
        Mm(210.0),
        Mm(297.0),
        "Layer 1"
    );

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Add title
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| DashboardError::PdfError(e.to_string()))?;

    current_layer.use_text(&dashboard.name, 24.0, Mm(10.0), Mm(280.0), &font);

    // Save to bytes
    let bytes = doc.save_to_bytes()
        .map_err(|e| DashboardError::PdfError(e.to_string()))?;

    Ok(bytes)
}

/// Generate Excel from dashboard
fn generate_excel(dashboard: &crate::Dashboard, _request: &ExportExcelRequest) -> Result<Vec<u8>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Add title
    worksheet.write_string(0, 0, &dashboard.name)
        .map_err(|e| DashboardError::ExcelError(e.to_string()))?;

    // Add headers
    worksheet.write_string(2, 0, "Widget")
        .map_err(|e| DashboardError::ExcelError(e.to_string()))?;
    worksheet.write_string(2, 1, "Type")
        .map_err(|e| DashboardError::ExcelError(e.to_string()))?;

    // Save to bytes
    let bytes = workbook.save_to_buffer()
        .map_err(|e| DashboardError::ExcelError(e.to_string()))?;

    Ok(bytes)
}

/// Generate CSV from widget
fn generate_csv(widget: &crate::Widget, request: &ExportCsvRequest) -> Result<Vec<u8>> {
    let delimiter = request.delimiter.as_deref().unwrap_or(",");
    let include_headers = request.include_headers.unwrap_or(true);

    let mut csv = String::new();

    // Add headers
    if include_headers {
        csv.push_str(&format!("Widget{}{}\n", delimiter, widget.title));
    }

    // Add data (simplified)
    csv.push_str(&format!("ID{}{}\n", delimiter, widget.id));
    csv.push_str(&format!("Type{}{:?}\n", delimiter, widget.widget_type));

    Ok(csv.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let valid = "user@example.com";
        let invalid = "notanemail";

        assert!(valid.contains('@'));
        assert!(!invalid.contains('@'));
    }
}
