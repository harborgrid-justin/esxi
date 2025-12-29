//! Meridian Dashboard - Analytics Dashboard and Reporting System
//!
//! This crate provides comprehensive dashboard and analytics capabilities for the
//! Meridian GIS Platform, including:
//! - Interactive dashboard creation and management
//! - Real-time data visualization
//! - Report generation and scheduling
//! - PDF and Excel export functionality
//! - Multi-source data integration

pub mod error;
pub mod api;
pub mod datasource;
pub mod scheduler;

pub use error::{DashboardError, Result};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub layout: DashboardLayout,
    pub widgets: Vec<Widget>,
    pub filters: Vec<DashboardFilter>,
    pub refresh_interval: Option<u32>, // seconds
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub cols: u32,
    pub rows: u32,
    pub breakpoints: LayoutBreakpoints,
}

/// Responsive breakpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBreakpoints {
    pub lg: u32, // >= 1200px
    pub md: u32, // >= 996px
    pub sm: u32, // >= 768px
    pub xs: u32, // >= 480px
    pub xxs: u32, // < 480px
}

impl Default for LayoutBreakpoints {
    fn default() -> Self {
        Self {
            lg: 12,
            md: 10,
            sm: 6,
            xs: 4,
            xxs: 2,
        }
    }
}

/// Widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub widget_type: WidgetType,
    pub title: String,
    pub description: Option<String>,
    pub position: WidgetPosition,
    pub config: WidgetConfig,
    pub data_source: DataSourceConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Widget types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    Map,
    Chart,
    Table,
    Kpi,
    Timeline,
    Filter,
}

/// Widget position in grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub min_w: Option<u32>,
    pub min_h: Option<u32>,
    pub max_w: Option<u32>,
    pub max_h: Option<u32>,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetConfig {
    Map {
        center: [f64; 2],
        zoom: u8,
        layers: Vec<String>,
    },
    Chart {
        chart_type: ChartType,
        x_axis: String,
        y_axis: Vec<String>,
        options: ChartOptions,
    },
    Table {
        columns: Vec<TableColumn>,
        pagination: bool,
        page_size: u32,
    },
    Kpi {
        metric: String,
        aggregation: AggregationType,
        comparison: Option<KpiComparison>,
        format: String,
    },
    Timeline {
        date_field: String,
        title_field: String,
        description_field: Option<String>,
    },
    Filter {
        field: String,
        filter_type: FilterType,
        default_value: Option<serde_json::Value>,
    },
}

/// Chart types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Scatter,
    Heatmap,
    Area,
    Column,
    Donut,
}

/// Chart options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    pub legend: bool,
    pub grid: bool,
    pub tooltip: bool,
    pub animation: bool,
    pub stacked: bool,
    pub colors: Option<Vec<String>>,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            legend: true,
            grid: true,
            tooltip: true,
            animation: true,
            stacked: false,
            colors: None,
        }
    }
}

/// Table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub field: String,
    pub header: String,
    pub width: Option<u32>,
    pub sortable: bool,
    pub filterable: bool,
    pub format: Option<String>,
}

/// Aggregation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AggregationType {
    Sum,
    Avg,
    Min,
    Max,
    Count,
    CountDistinct,
}

/// KPI comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiComparison {
    pub period: ComparisonPeriod,
    pub show_change: bool,
    pub show_percentage: bool,
}

/// Comparison periods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonPeriod {
    PreviousDay,
    PreviousWeek,
    PreviousMonth,
    PreviousYear,
    Custom { days: u32 },
}

/// Filter types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    Select,
    MultiSelect,
    DateRange,
    NumberRange,
    Text,
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DataSourceConfig {
    Sql {
        connection_id: Uuid,
        query: String,
        parameters: Vec<QueryParameter>,
    },
    Api {
        url: String,
        method: String,
        headers: std::collections::HashMap<String, String>,
        body: Option<String>,
    },
    Static {
        data: serde_json::Value,
    },
}

/// Query parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParameter {
    pub name: String,
    pub value: serde_json::Value,
    pub param_type: ParameterType,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Date,
    Array,
}

/// Dashboard filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardFilter {
    pub id: Uuid,
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
    pub applies_to: Vec<Uuid>, // Widget IDs
}

/// Filter operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
    Between,
}

/// Report definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub dashboard_id: Uuid,
    pub format: ReportFormat,
    pub schedule: Option<ReportSchedule>,
    pub recipients: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Report formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    Pdf,
    Excel,
    Csv,
    Json,
}

/// Report schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSchedule {
    pub cron: String,
    pub timezone: String,
    pub enabled: bool,
}

/// Dashboard service
pub struct DashboardService {
    pool: sqlx::PgPool,
}

impl DashboardService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Get dashboard by ID
    pub async fn get_dashboard(&self, id: Uuid) -> Result<Dashboard> {
        // Implementation will query database
        todo!("Implement get_dashboard")
    }

    /// Create new dashboard
    pub async fn create_dashboard(&self, dashboard: Dashboard) -> Result<Dashboard> {
        // Implementation will insert into database
        todo!("Implement create_dashboard")
    }

    /// Update dashboard
    pub async fn update_dashboard(&self, id: Uuid, dashboard: Dashboard) -> Result<Dashboard> {
        // Implementation will update database
        todo!("Implement update_dashboard")
    }

    /// Delete dashboard
    pub async fn delete_dashboard(&self, id: Uuid) -> Result<()> {
        // Implementation will delete from database
        todo!("Implement delete_dashboard")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_creation() {
        let widget = Widget {
            id: Uuid::new_v4(),
            dashboard_id: Uuid::new_v4(),
            widget_type: WidgetType::Chart,
            title: "Test Chart".to_string(),
            description: None,
            position: WidgetPosition {
                x: 0,
                y: 0,
                w: 6,
                h: 4,
                min_w: Some(2),
                min_h: Some(2),
                max_w: None,
                max_h: None,
            },
            config: WidgetConfig::Chart {
                chart_type: ChartType::Line,
                x_axis: "date".to_string(),
                y_axis: vec!["value".to_string()],
                options: ChartOptions::default(),
            },
            data_source: DataSourceConfig::Static {
                data: serde_json::json!([]),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(widget.widget_type, WidgetType::Chart);
    }
}
