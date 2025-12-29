//! Data source connectors for widgets

pub mod sql;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::Result;

/// Data source connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConnection {
    pub id: Uuid,
    pub name: String,
    pub connection_type: ConnectionType,
    pub config: ConnectionConfig,
    pub is_active: bool,
}

/// Connection types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    PostgreSql,
    MySql,
    Sqlite,
    MsSql,
    Oracle,
    Rest,
    GraphQL,
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectionConfig {
    Database {
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
        ssl: bool,
    },
    Rest {
        base_url: String,
        auth_type: AuthType,
        auth_config: HashMap<String, String>,
    },
    GraphQL {
        endpoint: String,
        auth_type: AuthType,
        auth_config: HashMap<String, String>,
    },
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    ApiKey,
    OAuth2,
}

/// Data source trait
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    /// Test connection
    async fn test_connection(&self) -> Result<bool>;

    /// Execute query and return results
    async fn execute(&self, query: &str, params: &[serde_json::Value]) -> Result<QueryResult>;

    /// Get schema information
    async fn get_schema(&self) -> Result<SchemaInfo>;
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// Schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
}

/// Table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub columns: Vec<ColumnInfo>,
}

/// Create data source from connection
pub fn create_data_source(connection: &DataSourceConnection) -> Result<Box<dyn DataSource>> {
    match &connection.connection_type {
        ConnectionType::PostgreSql | ConnectionType::MySql | ConnectionType::Sqlite => {
            Ok(Box::new(sql::SqlDataSource::new(connection.clone())?))
        }
        _ => {
            Err(crate::DashboardError::DataSourceError(
                format!("Unsupported data source type: {:?}", connection.connection_type)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_type() {
        let conn = DataSourceConnection {
            id: Uuid::new_v4(),
            name: "Test DB".to_string(),
            connection_type: ConnectionType::PostgreSql,
            config: ConnectionConfig::Database {
                host: "localhost".to_string(),
                port: 5432,
                database: "test".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
                ssl: false,
            },
            is_active: true,
        };

        assert_eq!(conn.connection_type, ConnectionType::PostgreSql);
    }
}
