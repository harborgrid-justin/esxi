//! SQL data source implementation

use sqlx::{Pool, Postgres, Any, Row};
use std::collections::HashMap;
use std::time::Instant;

use super::{DataSource, QueryResult, ColumnInfo, SchemaInfo, TableInfo, DataSourceConnection, ConnectionConfig};
use crate::{Result, DashboardError};

/// SQL data source
pub struct SqlDataSource {
    connection: DataSourceConnection,
    pool: Option<Pool<Postgres>>,
}

impl SqlDataSource {
    /// Create new SQL data source
    pub fn new(connection: DataSourceConnection) -> Result<Self> {
        Ok(Self {
            connection,
            pool: None,
        })
    }

    /// Get connection string
    fn get_connection_string(&self) -> Result<String> {
        match &self.connection.config {
            ConnectionConfig::Database { host, port, database, username, password, ssl } => {
                let ssl_mode = if *ssl { "require" } else { "disable" };
                Ok(format!(
                    "postgresql://{}:{}@{}:{}/{}?sslmode={}",
                    username, password, host, port, database, ssl_mode
                ))
            }
            _ => Err(DashboardError::DataSourceError("Invalid connection config for SQL".to_string())),
        }
    }

    /// Initialize connection pool
    async fn init_pool(&mut self) -> Result<()> {
        if self.pool.is_none() {
            let connection_string = self.get_connection_string()?;
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&connection_string)
                .await
                .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;

            self.pool = Some(pool);
        }
        Ok(())
    }

    /// Get pool reference
    async fn get_pool(&mut self) -> Result<&Pool<Postgres>> {
        self.init_pool().await?;
        self.pool.as_ref()
            .ok_or_else(|| DashboardError::DataSourceError("Pool not initialized".to_string()))
    }
}

#[async_trait::async_trait]
impl DataSource for SqlDataSource {
    async fn test_connection(&self) -> Result<bool> {
        let connection_string = self.get_connection_string()?;

        match sqlx::postgres::PgPool::connect(&connection_string).await {
            Ok(_) => Ok(true),
            Err(e) => Err(DashboardError::DataSourceError(format!("Connection test failed: {}", e))),
        }
    }

    async fn execute(&self, query: &str, params: &[serde_json::Value]) -> Result<QueryResult> {
        let start_time = Instant::now();

        // Create a new connection for this query
        let connection_string = self.get_connection_string()?;
        let pool = sqlx::postgres::PgPool::connect(&connection_string)
            .await
            .map_err(|e| DashboardError::QueryExecutionError(e.to_string()))?;

        // Execute query
        let rows = sqlx::query(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| DashboardError::QueryExecutionError(e.to_string()))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Convert rows to our format
        let mut result_rows = Vec::new();
        let mut columns = Vec::new();

        if let Some(first_row) = rows.first() {
            // Get column information from first row
            for column in first_row.columns() {
                columns.push(ColumnInfo {
                    name: column.name().to_string(),
                    data_type: format!("{:?}", column.type_info()),
                    nullable: true, // SQLx doesn't expose nullability easily
                });
            }
        }

        // Convert each row
        for row in rows {
            let mut result_row = HashMap::new();

            for (i, column) in columns.iter().enumerate() {
                // Try to get value as JSON
                let value: Option<serde_json::Value> = row.try_get(i).ok();
                result_row.insert(
                    column.name.clone(),
                    value.unwrap_or(serde_json::Value::Null)
                );
            }

            result_rows.push(result_row);
        }

        Ok(QueryResult {
            columns,
            row_count: result_rows.len(),
            rows: result_rows,
            execution_time_ms: execution_time,
        })
    }

    async fn get_schema(&self) -> Result<SchemaInfo> {
        let connection_string = self.get_connection_string()?;
        let pool = sqlx::postgres::PgPool::connect(&connection_string)
            .await
            .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;

        // Query for tables
        let table_query = r#"
            SELECT
                table_schema,
                table_name
            FROM information_schema.tables
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_schema, table_name
        "#;

        let table_rows = sqlx::query(table_query)
            .fetch_all(&pool)
            .await
            .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;

        let mut tables = Vec::new();

        for table_row in table_rows {
            let schema: String = table_row.try_get("table_schema")
                .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;
            let table_name: String = table_row.try_get("table_name")
                .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;

            // Query for columns
            let column_query = r#"
                SELECT
                    column_name,
                    data_type,
                    is_nullable
                FROM information_schema.columns
                WHERE table_schema = $1 AND table_name = $2
                ORDER BY ordinal_position
            "#;

            let column_rows = sqlx::query(column_query)
                .bind(&schema)
                .bind(&table_name)
                .fetch_all(&pool)
                .await
                .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;

            let mut columns = Vec::new();

            for column_row in column_rows {
                let column_name: String = column_row.try_get("column_name")
                    .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;
                let data_type: String = column_row.try_get("data_type")
                    .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;
                let is_nullable: String = column_row.try_get("is_nullable")
                    .map_err(|e| DashboardError::DataSourceError(e.to_string()))?;

                columns.push(ColumnInfo {
                    name: column_name,
                    data_type,
                    nullable: is_nullable == "YES",
                });
            }

            tables.push(TableInfo {
                name: table_name,
                schema: Some(schema),
                columns,
            });
        }

        Ok(SchemaInfo { tables })
    }
}

/// SQL query builder
pub struct SqlQueryBuilder {
    select: Vec<String>,
    from: Option<String>,
    joins: Vec<String>,
    where_clauses: Vec<String>,
    group_by: Vec<String>,
    order_by: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl SqlQueryBuilder {
    pub fn new() -> Self {
        Self {
            select: Vec::new(),
            from: None,
            joins: Vec::new(),
            where_clauses: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn select(mut self, fields: &[&str]) -> Self {
        self.select.extend(fields.iter().map(|s| s.to_string()));
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from = Some(table.to_string());
        self
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    pub fn group_by(mut self, fields: &[&str]) -> Self {
        self.group_by.extend(fields.iter().map(|s| s.to_string()));
        self
    }

    pub fn order_by(mut self, field: &str, direction: &str) -> Self {
        self.order_by.push(format!("{} {}", field, direction));
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(self) -> Result<String> {
        let mut query = String::new();

        // SELECT
        if self.select.is_empty() {
            query.push_str("SELECT *");
        } else {
            query.push_str(&format!("SELECT {}", self.select.join(", ")));
        }

        // FROM
        if let Some(from) = self.from {
            query.push_str(&format!(" FROM {}", from));
        } else {
            return Err(DashboardError::QueryExecutionError("FROM clause is required".to_string()));
        }

        // JOINs
        for join in &self.joins {
            query.push_str(&format!(" {}", join));
        }

        // WHERE
        if !self.where_clauses.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clauses.join(" AND ")));
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            query.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        // LIMIT
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        Ok(query)
    }
}

impl Default for SqlQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = SqlQueryBuilder::new()
            .select(&["id", "name", "created_at"])
            .from("users")
            .where_clause("active = true")
            .order_by("created_at", "DESC")
            .limit(10)
            .build()
            .unwrap();

        assert!(query.contains("SELECT id, name, created_at"));
        assert!(query.contains("FROM users"));
        assert!(query.contains("WHERE active = true"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("LIMIT 10"));
    }
}
