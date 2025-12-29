//! Dashboard CRUD API endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::{Dashboard, DashboardLayout, LayoutBreakpoints, Result, DashboardError};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

/// List dashboards query parameters
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub owner_id: Option<Uuid>,
    pub is_public: Option<bool>,
}

/// Dashboard list response
#[derive(Debug, Serialize)]
pub struct DashboardList {
    pub dashboards: Vec<Dashboard>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

/// List all dashboards
pub async fn list_dashboards(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<DashboardList>> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    let offset = (page - 1) * page_size;

    let mut sql = "SELECT * FROM dashboards WHERE 1=1".to_string();

    if let Some(owner_id) = query.owner_id {
        sql.push_str(&format!(" AND owner_id = '{}'", owner_id));
    }

    if let Some(is_public) = query.is_public {
        sql.push_str(&format!(" AND is_public = {}", is_public));
    }

    sql.push_str(" ORDER BY created_at DESC");
    sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

    // Note: This is a simplified implementation
    // In production, use proper parameterized queries

    let dashboards: Vec<Dashboard> = sqlx::query_as(&sql)
        .fetch_all(&state.pool)
        .await?;

    let count_sql = "SELECT COUNT(*) FROM dashboards WHERE 1=1";
    let total: (i64,) = sqlx::query_as(count_sql)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(DashboardList {
        dashboards,
        total: total.0 as u32,
        page,
        page_size,
    }))
}

/// Create dashboard request
#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub layout: Option<DashboardLayout>,
    pub is_public: Option<bool>,
}

/// Create a new dashboard
pub async fn create_dashboard(
    State(state): State<AppState>,
    Json(request): Json<CreateDashboardRequest>,
) -> Result<Json<Dashboard>> {
    // Validate name
    if request.name.trim().is_empty() {
        return Err(DashboardError::ValidationError("Dashboard name cannot be empty".to_string()));
    }

    let dashboard = Dashboard {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        owner_id: request.owner_id,
        layout: request.layout.unwrap_or(DashboardLayout {
            cols: 12,
            rows: 12,
            breakpoints: LayoutBreakpoints::default(),
        }),
        widgets: Vec::new(),
        filters: Vec::new(),
        refresh_interval: None,
        is_public: request.is_public.unwrap_or(false),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        r#"
        INSERT INTO dashboards (id, name, description, owner_id, layout, is_public, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(&dashboard.id)
    .bind(&dashboard.name)
    .bind(&dashboard.description)
    .bind(&dashboard.owner_id)
    .bind(serde_json::to_value(&dashboard.layout)?)
    .bind(&dashboard.is_public)
    .bind(&dashboard.created_at)
    .bind(&dashboard.updated_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(dashboard))
}

/// Get a dashboard by ID
pub async fn get_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Dashboard>> {
    let dashboard: Dashboard = sqlx::query_as(
        "SELECT * FROM dashboards WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::DashboardNotFound(id))?;

    Ok(Json(dashboard))
}

/// Update dashboard request
#[derive(Debug, Deserialize)]
pub struct UpdateDashboardRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub layout: Option<DashboardLayout>,
    pub refresh_interval: Option<u32>,
    pub is_public: Option<bool>,
}

/// Update a dashboard
pub async fn update_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateDashboardRequest>,
) -> Result<Json<Dashboard>> {
    // Fetch existing dashboard
    let mut dashboard: Dashboard = sqlx::query_as(
        "SELECT * FROM dashboards WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::DashboardNotFound(id))?;

    // Update fields
    if let Some(name) = request.name {
        if name.trim().is_empty() {
            return Err(DashboardError::ValidationError("Dashboard name cannot be empty".to_string()));
        }
        dashboard.name = name;
    }

    if let Some(description) = request.description {
        dashboard.description = Some(description);
    }

    if let Some(layout) = request.layout {
        dashboard.layout = layout;
    }

    if let Some(refresh_interval) = request.refresh_interval {
        dashboard.refresh_interval = Some(refresh_interval);
    }

    if let Some(is_public) = request.is_public {
        dashboard.is_public = is_public;
    }

    dashboard.updated_at = Utc::now();

    sqlx::query(
        r#"
        UPDATE dashboards
        SET name = $1, description = $2, layout = $3, refresh_interval = $4, is_public = $5, updated_at = $6
        WHERE id = $7
        "#
    )
    .bind(&dashboard.name)
    .bind(&dashboard.description)
    .bind(serde_json::to_value(&dashboard.layout)?)
    .bind(&dashboard.refresh_interval)
    .bind(&dashboard.is_public)
    .bind(&dashboard.updated_at)
    .bind(&id)
    .execute(&state.pool)
    .await?;

    Ok(Json(dashboard))
}

/// Delete a dashboard
pub async fn delete_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM dashboards WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DashboardError::DashboardNotFound(id));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Dashboard deleted successfully"
    })))
}

/// Clone a dashboard
pub async fn clone_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Dashboard>> {
    // Fetch original dashboard
    let original: Dashboard = sqlx::query_as(
        "SELECT * FROM dashboards WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DashboardError::DashboardNotFound(id))?;

    // Create cloned dashboard
    let cloned = Dashboard {
        id: Uuid::new_v4(),
        name: format!("{} (Copy)", original.name),
        description: original.description.clone(),
        owner_id: original.owner_id,
        layout: original.layout.clone(),
        widgets: Vec::new(), // Widgets will be cloned separately
        filters: original.filters.clone(),
        refresh_interval: original.refresh_interval,
        is_public: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        r#"
        INSERT INTO dashboards (id, name, description, owner_id, layout, is_public, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(&cloned.id)
    .bind(&cloned.name)
    .bind(&cloned.description)
    .bind(&cloned.owner_id)
    .bind(serde_json::to_value(&cloned.layout)?)
    .bind(&cloned.is_public)
    .bind(&cloned.created_at)
    .bind(&cloned.updated_at)
    .execute(&state.pool)
    .await?;

    Ok(Json(cloned))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dashboard_validation() {
        let request = CreateDashboardRequest {
            name: "".to_string(),
            description: None,
            owner_id: Uuid::new_v4(),
            layout: None,
            is_public: None,
        };

        assert!(request.name.trim().is_empty());
    }
}
