//! Report scheduling system

use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error};

use crate::{Report, ReportFormat, Result, DashboardError};

/// Report scheduler
pub struct ReportScheduler {
    scheduler: JobScheduler,
    pool: sqlx::PgPool,
}

impl ReportScheduler {
    /// Create new scheduler
    pub async fn new(pool: sqlx::PgPool) -> Result<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| DashboardError::ScheduleError(e.to_string()))?;

        Ok(Self { scheduler, pool })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        self.scheduler
            .start()
            .await
            .map_err(|e| DashboardError::ScheduleError(e.to_string()))?;

        info!("Report scheduler started");
        Ok(())
    }

    /// Stop the scheduler
    pub async fn shutdown(&self) -> Result<()> {
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| DashboardError::ScheduleError(e.to_string()))?;

        info!("Report scheduler stopped");
        Ok(())
    }

    /// Schedule a report
    pub async fn schedule_report(&self, report: &Report) -> Result<Uuid> {
        let schedule = report.schedule.as_ref()
            .ok_or_else(|| DashboardError::ScheduleError("No schedule defined".to_string()))?;

        if !schedule.enabled {
            return Err(DashboardError::ScheduleError("Schedule is disabled".to_string()));
        }

        let report_id = report.id;
        let dashboard_id = report.dashboard_id;
        let format = report.format.clone();
        let recipients = report.recipients.clone();
        let pool = self.pool.clone();

        // Create cron job
        let job = Job::new_async(schedule.cron.as_str(), move |_uuid, _l| {
            let report_id = report_id;
            let dashboard_id = dashboard_id;
            let format = format.clone();
            let recipients = recipients.clone();
            let pool = pool.clone();

            Box::pin(async move {
                info!("Executing scheduled report: {}", report_id);

                match generate_and_send_report(report_id, dashboard_id, format, recipients, pool).await {
                    Ok(_) => {
                        info!("Report {} generated and sent successfully", report_id);
                    }
                    Err(e) => {
                        error!("Failed to generate report {}: {}", report_id, e);
                    }
                }
            })
        })
        .map_err(|e| DashboardError::ScheduleError(e.to_string()))?;

        let job_id = self.scheduler
            .add(job)
            .await
            .map_err(|e| DashboardError::ScheduleError(e.to_string()))?;

        info!("Scheduled report {} with job ID {}", report_id, job_id);
        Ok(job_id)
    }

    /// Unschedule a report
    pub async fn unschedule_report(&self, job_id: Uuid) -> Result<()> {
        self.scheduler
            .remove(&job_id)
            .await
            .map_err(|e| DashboardError::ScheduleError(e.to_string()))?;

        info!("Unscheduled job {}", job_id);
        Ok(())
    }

    /// Load all scheduled reports from database
    pub async fn load_scheduled_reports(&self) -> Result<()> {
        let reports: Vec<Report> = sqlx::query_as(
            "SELECT * FROM reports WHERE schedule IS NOT NULL"
        )
        .fetch_all(&self.pool)
        .await?;

        for report in reports {
            if let Some(schedule) = &report.schedule {
                if schedule.enabled {
                    match self.schedule_report(&report).await {
                        Ok(job_id) => {
                            info!("Loaded scheduled report {} with job ID {}", report.id, job_id);
                        }
                        Err(e) => {
                            error!("Failed to schedule report {}: {}", report.id, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Generate and send report
async fn generate_and_send_report(
    report_id: Uuid,
    dashboard_id: Uuid,
    format: ReportFormat,
    recipients: Vec<String>,
    pool: sqlx::PgPool,
) -> Result<()> {
    // Fetch dashboard
    let dashboard: crate::Dashboard = sqlx::query_as(
        "SELECT * FROM dashboards WHERE id = $1"
    )
    .bind(dashboard_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(DashboardError::DashboardNotFound(dashboard_id))?;

    // Generate report based on format
    let report_data = match format {
        ReportFormat::Pdf => {
            generate_pdf_report(&dashboard).await?
        }
        ReportFormat::Excel => {
            generate_excel_report(&dashboard).await?
        }
        ReportFormat::Csv => {
            generate_csv_report(&dashboard).await?
        }
        ReportFormat::Json => {
            generate_json_report(&dashboard).await?
        }
    };

    // Send to recipients
    for recipient in recipients {
        send_report_email(&recipient, &dashboard.name, &report_data, &format).await?;
    }

    // Record execution
    sqlx::query(
        "INSERT INTO report_executions (id, report_id, executed_at, status) VALUES ($1, $2, $3, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(report_id)
    .bind(Utc::now())
    .bind("success")
    .execute(&pool)
    .await?;

    Ok(())
}

/// Generate PDF report
async fn generate_pdf_report(dashboard: &crate::Dashboard) -> Result<Vec<u8>> {
    use printpdf::*;

    let (doc, page1, layer1) = PdfDocument::new(
        &dashboard.name,
        Mm(210.0),
        Mm(297.0),
        "Layer 1"
    );

    let current_layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| DashboardError::PdfError(e.to_string()))?;

    current_layer.use_text(&dashboard.name, 24.0, Mm(10.0), Mm(280.0), &font);

    let bytes = doc.save_to_bytes()
        .map_err(|e| DashboardError::PdfError(e.to_string()))?;

    Ok(bytes)
}

/// Generate Excel report
async fn generate_excel_report(dashboard: &crate::Dashboard) -> Result<Vec<u8>> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.write_string(0, 0, &dashboard.name)
        .map_err(|e| DashboardError::ExcelError(e.to_string()))?;

    let bytes = workbook.save_to_buffer()
        .map_err(|e| DashboardError::ExcelError(e.to_string()))?;

    Ok(bytes)
}

/// Generate CSV report
async fn generate_csv_report(dashboard: &crate::Dashboard) -> Result<Vec<u8>> {
    let csv = format!("Dashboard,{}\n", dashboard.name);
    Ok(csv.into_bytes())
}

/// Generate JSON report
async fn generate_json_report(dashboard: &crate::Dashboard) -> Result<Vec<u8>> {
    let json = serde_json::to_vec_pretty(dashboard)?;
    Ok(json)
}

/// Send report via email
async fn send_report_email(
    recipient: &str,
    subject: &str,
    data: &[u8],
    format: &ReportFormat,
) -> Result<()> {
    // This is a placeholder - in production, integrate with email service
    info!(
        "Sending {} report '{}' to {} ({} bytes)",
        format_name(format),
        subject,
        recipient,
        data.len()
    );

    // TODO: Implement actual email sending
    // - Use SMTP library like lettre
    // - Or integrate with email service API (SendGrid, AWS SES, etc.)

    Ok(())
}

/// Get format name
fn format_name(format: &ReportFormat) -> &str {
    match format {
        ReportFormat::Pdf => "PDF",
        ReportFormat::Excel => "Excel",
        ReportFormat::Csv => "CSV",
        ReportFormat::Json => "JSON",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_name() {
        assert_eq!(format_name(&ReportFormat::Pdf), "PDF");
        assert_eq!(format_name(&ReportFormat::Excel), "Excel");
        assert_eq!(format_name(&ReportFormat::Csv), "CSV");
        assert_eq!(format_name(&ReportFormat::Json), "JSON");
    }
}
