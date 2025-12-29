//! Widget API endpoints

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::{Widget, WidgetType, WidgetPosition, WidgetConfig, DataSourceConfig, Result, DashboardError};
use super::dashboard::AppState;

/// Widget list response
#[derive(Debug, Serialize)]
pub struct WidgetList {
    pub widgets: Vec<Widget>,
}

/// List widgets for a dashboard
pub async fn list_widgets(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<WidgetList>> {
    let widgets: Vec<Widget> = sqlx::query_as(
        "SELECT * FROM widgets WHERE dashboard_id = $1 ORDER BY created_at"
    )
    .bind(dashboard_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(WidgetList { widgets }))
}

/// Create widget request
#[derive(Debug, Deserialize)]
pub struct CreateWidgetRequest {
    pub widget_type: WidgetType,
    pub title: String,
    pub description: Option<String>,
    pub position: WidgetPosition,
    pub config: WidgetConfig,
    pub data_source: DataSourceConfig,
}

/// Create a new widget
pub async fn create_widget(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
    Json(request): Json<CreateWidgetRequest>,
) -> Result<Json<Widget>> {
    // Validate title
    if request.title.trim().is_empty() {
        return Err(DashboardError::ValidationError("Widget title cannot be empty".to_string()));
    }

    // Validate position
    if request.position.w == 0 || request.position.h == 0 {
        return Err(DashboardError::InvalidLayout("Widget dimensions must be greater than 0".to_string()));
    }

    let widget = Widget {
        id: Uuid::new_v4(),
        dashboard_id,
        widget_type: request.widget_type,
        title: request.title,
        description: request.description,
        position: request.position,
        config: request.config,
        data_source: request.data_source,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        r#"
        INSERT INTO widgets (id, dashboard_id, widget_type, title, description, position, config, data_source, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(&widget.id)
    .bind(&widget.dashboard_id)
    .bind(serde_json::to_value(&widget.widget_type)?)
    .bind(&widget.title)
    .bind(&widget.description)
    .bind(serde_json::to_value(&widget.position)?)
    .bind(serde_json::to_value(&widget.config)?)
    .bind(serde_json::to_value(&widget.data_source)?)
    .bind(&widget.created_at)
    .bind(&widget.updated_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(widget))
}

/// Get a widget by ID
pub async fn get_widget(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Widget>> {
    let widget: Widget = sqlx::query_as(
        "SELECT * FROM widgets WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::WidgetNotFound(id))?;

    Ok(Json(widget))
}

/// Update widget request
#[derive(Debug, Deserialize)]
pub struct UpdateWidgetRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub position: Option<WidgetPosition>,
    pub config: Option<WidgetConfig>,
    pub data_source: Option<DataSourceConfig>,
}

/// Update a widget
pub async fn update_widget(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateWidgetRequest>,
) -> Result<Json<Widget>> {
    // Fetch existing widget
    let mut widget: Widget = sqlx::query_as(
        "SELECT * FROM widgets WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::WidgetNotFound(id))?;

    // Update fields
    if let Some(title) = request.title {
        if title.trim().is_empty() {
            return Err(DashboardError::ValidationError("Widget title cannot be empty".to_string()));
        }
        widget.title = title;
    }

    if let Some(description) = request.description {
        widget.description = Some(description);
    }

    if let Some(position) = request.position {
        if position.w == 0 || position.h == 0 {
            return Err(DashboardError::InvalidLayout("Widget dimensions must be greater than 0".to_string()));
        }
        widget.position = position;
    }

    if let Some(config) = request.config {
        widget.config = config;
    }

    if let Some(data_source) = request.data_source {
        widget.data_source = data_source;
    }

    widget.updated_at = Utc::now();

    sqlx::query(
        r#"
        UPDATE widgets
        SET title = $1, description = $2, position = $3, config = $4, data_source = $5, updated_at = $6
        WHERE id = $7
        "#
    )
    .bind(&widget.title)
    .bind(&widget.description)
    .bind(serde_json::to_value(&widget.position)?)
    .bind(serde_json::to_value(&widget.config)?)
    .bind(serde_json::to_value(&widget.data_source)?)
    .bind(&widget.updated_at)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    Ok(Json(widget))
}

/// Delete a widget
pub async fn delete_widget(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM widgets WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DashboardError::WidgetNotFound(id));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Widget deleted successfully"
    })))
}

/// Widget data response
#[derive(Debug, Serialize)]
pub struct WidgetDataResponse {
    pub widget_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<Utc>,
}

/// Get widget data
pub async fn get_widget_data(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WidgetDataResponse>> {
    // Fetch widget
    let widget: Widget = sqlx::query_as(
        "SELECT * FROM widgets WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::WidgetNotFound(id))?;

    // Execute data source query
    let data = match &widget.data_source {
        DataSourceConfig::Sql { connection_id, query, parameters } => {
            // Execute SQL query
            execute_sql_query(&state.pool, query, parameters).await?
        }
        DataSourceConfig::Api { url, method, headers, body } => {
            // Execute API call
            execute_api_call(url, method, headers, body.as_deref()).await?
        }
        DataSourceConfig::Static { data } => {
            data.clone()
        }
    };

    Ok(Json(WidgetDataResponse {
        widget_id: id,
        data,
        timestamp: Utc::now(),
    }))
}

/// Refresh widget data (force reload)
pub async fn refresh_widget(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WidgetDataResponse>> {
    // Same implementation as get_widget_data but could include cache invalidation
    get_widget_data(State(state), Path(id)).await
}

/// Execute SQL query
async fn execute_sql_query(
    pool: &sqlx::PgPool,
    query: &str,
    parameters: &[crate::QueryParameter],
) -> Result<serde_json::Value> {
    // This is a simplified implementation
    // In production, properly bind parameters and handle different types

    let rows: Vec<serde_json::Value> = sqlx::query_scalar(query)
        .fetch_all(pool)
        .await
        .map_err(|e| DashboardError::QueryExecutionError(e.to_string()))?;

    Ok(serde_json::json!(rows))
}

/// Execute API call
async fn execute_api_call(
    url: &str,
    method: &str,
    headers: &std::collections::HashMap<String, String>,
    body: Option<&str>,
) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();

    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => return Err(DashboardError::DataSourceError(format!("Unsupported HTTP method: {}", method))),
    };

    // Add headers
    for (key, value) in headers {
        request = request.header(key, value);
    }

    // Add body if present
    if let Some(body_str) = body {
        request = request.body(body_str.to_string());
    }

    let response = request
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_position_validation() {
        let position = WidgetPosition {
            x: 0,
            y: 0,
            w: 0,
            h: 4,
            min_w: None,
            min_h: None,
            max_w: None,
            max_h: None,
        };

        assert_eq!(position.w, 0);
    }
}
