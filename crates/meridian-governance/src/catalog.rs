//! Data catalog with metadata management for datasets, tables, and fields

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Data catalog for managing dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCatalog {
    /// Registered datasets
    datasets: IndexMap<String, DatasetMetadata>,
    /// Registered tables
    tables: IndexMap<String, TableMetadata>,
    /// Tag index for fast lookups
    tag_index: HashMap<String, HashSet<String>>,
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// Unique identifier
    pub id: Uuid,
    /// Dataset name
    pub name: String,
    /// Dataset description
    pub description: String,
    /// Dataset owner
    pub owner: String,
    /// Data steward responsible for quality
    pub steward: Option<String>,
    /// Dataset domain (e.g., "geospatial", "customer", "financial")
    pub domain: String,
    /// Source system
    pub source_system: String,
    /// Dataset location/path
    pub location: String,
    /// Dataset format (e.g., "parquet", "shapefile", "geojson")
    pub format: DatasetFormat,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Last accessed timestamp
    pub last_accessed: Option<DateTime<Utc>>,
    /// Tags for categorization
    pub tags: HashSet<String>,
    /// Custom metadata properties
    pub properties: HashMap<String, String>,
    /// Tables contained in this dataset
    pub tables: Vec<String>,
    /// Data classification
    pub classification: Option<String>,
    /// Quality score (0.0 - 1.0)
    pub quality_score: Option<f64>,
    /// Dataset status
    pub status: DatasetStatus,
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    /// Unique identifier
    pub id: Uuid,
    /// Table name
    pub name: String,
    /// Fully qualified name (dataset.table)
    pub qualified_name: String,
    /// Table description
    pub description: String,
    /// Parent dataset
    pub dataset_id: String,
    /// Schema definition
    pub schema: TableSchema,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Row count (if available)
    pub row_count: Option<u64>,
    /// Size in bytes
    pub size_bytes: Option<u64>,
    /// Partition keys
    pub partition_keys: Vec<String>,
    /// Sort keys
    pub sort_keys: Vec<String>,
    /// Tags for categorization
    pub tags: HashSet<String>,
    /// Custom metadata properties
    pub properties: HashMap<String, String>,
}

/// Table schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// Schema fields
    pub fields: Vec<FieldMetadata>,
    /// Primary keys
    pub primary_keys: Vec<String>,
    /// Foreign key constraints
    pub foreign_keys: Vec<ForeignKeyConstraint>,
    /// Unique constraints
    pub unique_constraints: Vec<Vec<String>>,
}

/// Field metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldMetadata {
    /// Field name
    pub name: String,
    /// Field data type
    pub data_type: DataType,
    /// Field description
    pub description: Option<String>,
    /// Whether field is nullable
    pub nullable: bool,
    /// Default value
    pub default_value: Option<String>,
    /// Whether field is part of primary key
    pub is_primary_key: bool,
    /// Whether field is indexed
    pub is_indexed: bool,
    /// Business term reference
    pub business_term: Option<String>,
    /// Data classification
    pub classification: Option<String>,
    /// Tags for categorization
    pub tags: HashSet<String>,
    /// Custom metadata properties
    pub properties: HashMap<String, String>,
}

/// Foreign key constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyConstraint {
    /// Name of the constraint
    pub name: String,
    /// Source fields
    pub source_fields: Vec<String>,
    /// Referenced table
    pub referenced_table: String,
    /// Referenced fields
    pub referenced_fields: Vec<String>,
}

/// Data type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataType {
    /// Boolean type
    Boolean,
    /// 8-bit integer
    Int8,
    /// 16-bit integer
    Int16,
    /// 32-bit integer
    Int32,
    /// 64-bit integer
    Int64,
    /// 32-bit floating point
    Float32,
    /// 64-bit floating point
    Float64,
    /// Decimal with precision and scale
    Decimal { precision: u8, scale: u8 },
    /// String type
    String,
    /// Binary data
    Binary,
    /// Date type
    Date,
    /// Timestamp without timezone
    Timestamp,
    /// Timestamp with timezone
    TimestampTz,
    /// Time type
    Time,
    /// UUID type
    Uuid,
    /// JSON type
    Json,
    /// Geometry type (for GIS data)
    Geometry { srid: Option<i32> },
    /// Geography type (for GIS data)
    Geography { srid: Option<i32> },
    /// Array type
    Array { element_type: Box<DataType> },
    /// Struct/Object type
    Struct { fields: Vec<FieldMetadata> },
    /// Custom type
    Custom(String),
}

/// Dataset format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatasetFormat {
    Parquet,
    Shapefile,
    GeoJSON,
    GeoPackage,
    CSV,
    JSON,
    Avro,
    ORC,
    PostGIS,
    Custom(String),
}

/// Dataset status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatasetStatus {
    /// Active and ready for use
    Active,
    /// Deprecated, not recommended for new use
    Deprecated,
    /// Archived, read-only access
    Archived,
    /// Under development
    Development,
    /// Decommissioned, should not be used
    Decommissioned,
}

impl DataCatalog {
    /// Create a new data catalog
    pub fn new() -> Self {
        Self {
            datasets: IndexMap::new(),
            tables: IndexMap::new(),
            tag_index: HashMap::new(),
        }
    }

    /// Register a new dataset
    pub fn register_dataset(&mut self, metadata: DatasetMetadata) -> Result<()> {
        if self.datasets.contains_key(&metadata.name) {
            return Err(GovernanceError::catalog(format!(
                "Dataset already exists: {}",
                metadata.name
            )));
        }

        // Index tags
        for tag in &metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(metadata.name.clone());
        }

        self.datasets.insert(metadata.name.clone(), metadata);
        Ok(())
    }

    /// Register a new table
    pub fn register_table(&mut self, metadata: TableMetadata) -> Result<()> {
        // Verify parent dataset exists
        if !self.datasets.contains_key(&metadata.dataset_id) {
            return Err(GovernanceError::DatasetNotFound(
                metadata.dataset_id.clone(),
            ));
        }

        if self.tables.contains_key(&metadata.qualified_name) {
            return Err(GovernanceError::catalog(format!(
                "Table already exists: {}",
                metadata.qualified_name
            )));
        }

        // Index tags
        for tag in &metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(metadata.qualified_name.clone());
        }

        // Add table to dataset
        if let Some(dataset) = self.datasets.get_mut(&metadata.dataset_id) {
            dataset.tables.push(metadata.qualified_name.clone());
        }

        self.tables
            .insert(metadata.qualified_name.clone(), metadata);
        Ok(())
    }

    /// Get dataset metadata by name
    pub fn get_dataset(&self, name: &str) -> Result<&DatasetMetadata> {
        self.datasets
            .get(name)
            .ok_or_else(|| GovernanceError::DatasetNotFound(name.to_string()))
    }

    /// Get mutable dataset metadata by name
    pub fn get_dataset_mut(&mut self, name: &str) -> Result<&mut DatasetMetadata> {
        self.datasets
            .get_mut(name)
            .ok_or_else(|| GovernanceError::DatasetNotFound(name.to_string()))
    }

    /// Get table metadata by qualified name
    pub fn get_table(&self, qualified_name: &str) -> Result<&TableMetadata> {
        self.tables
            .get(qualified_name)
            .ok_or_else(|| GovernanceError::catalog(format!("Table not found: {}", qualified_name)))
    }

    /// Get mutable table metadata by qualified name
    pub fn get_table_mut(&mut self, qualified_name: &str) -> Result<&mut TableMetadata> {
        self.tables.get_mut(qualified_name).ok_or_else(|| {
            GovernanceError::catalog(format!("Table not found: {}", qualified_name))
        })
    }

    /// Search datasets by tags
    pub fn search_by_tags(&self, tags: &[String]) -> Vec<&DatasetMetadata> {
        let mut matching_names = HashSet::new();

        for tag in tags {
            if let Some(names) = self.tag_index.get(tag) {
                if matching_names.is_empty() {
                    matching_names = names.clone();
                } else {
                    matching_names.retain(|name| names.contains(name));
                }
            } else {
                return Vec::new();
            }
        }

        matching_names
            .iter()
            .filter_map(|name| self.datasets.get(name))
            .collect()
    }

    /// Search datasets by domain
    pub fn search_by_domain(&self, domain: &str) -> Vec<&DatasetMetadata> {
        self.datasets
            .values()
            .filter(|ds| ds.domain == domain)
            .collect()
    }

    /// Search datasets by owner
    pub fn search_by_owner(&self, owner: &str) -> Vec<&DatasetMetadata> {
        self.datasets
            .values()
            .filter(|ds| ds.owner == owner)
            .collect()
    }

    /// Search datasets by status
    pub fn search_by_status(&self, status: DatasetStatus) -> Vec<&DatasetMetadata> {
        self.datasets
            .values()
            .filter(|ds| ds.status == status)
            .collect()
    }

    /// Update dataset metadata
    pub fn update_dataset(&mut self, name: &str, metadata: DatasetMetadata) -> Result<()> {
        if !self.datasets.contains_key(name) {
            return Err(GovernanceError::DatasetNotFound(name.to_string()));
        }

        // Update tag index
        if let Some(old_metadata) = self.datasets.get(name) {
            for tag in &old_metadata.tags {
                if let Some(names) = self.tag_index.get_mut(tag) {
                    names.remove(name);
                }
            }
        }

        for tag in &metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(metadata.name.clone());
        }

        self.datasets.insert(name.to_string(), metadata);
        Ok(())
    }

    /// Update table metadata
    pub fn update_table(&mut self, qualified_name: &str, metadata: TableMetadata) -> Result<()> {
        if !self.tables.contains_key(qualified_name) {
            return Err(GovernanceError::catalog(format!(
                "Table not found: {}",
                qualified_name
            )));
        }

        // Update tag index
        if let Some(old_metadata) = self.tables.get(qualified_name) {
            for tag in &old_metadata.tags {
                if let Some(names) = self.tag_index.get_mut(tag) {
                    names.remove(qualified_name);
                }
            }
        }

        for tag in &metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(metadata.qualified_name.clone());
        }

        self.tables.insert(qualified_name.to_string(), metadata);
        Ok(())
    }

    /// Remove a dataset from the catalog
    pub fn remove_dataset(&mut self, name: &str) -> Result<DatasetMetadata> {
        let metadata = self
            .datasets
            .shift_remove(name)
            .ok_or_else(|| GovernanceError::DatasetNotFound(name.to_string()))?;

        // Remove from tag index
        for tag in &metadata.tags {
            if let Some(names) = self.tag_index.get_mut(tag) {
                names.remove(name);
            }
        }

        // Remove associated tables
        for table_name in &metadata.tables {
            self.tables.shift_remove(table_name);
        }

        Ok(metadata)
    }

    /// List all datasets
    pub fn list_datasets(&self) -> Vec<&DatasetMetadata> {
        self.datasets.values().collect()
    }

    /// List all tables
    pub fn list_tables(&self) -> Vec<&TableMetadata> {
        self.tables.values().collect()
    }

    /// Get catalog statistics
    pub fn get_statistics(&self) -> CatalogStatistics {
        let total_size_bytes: u64 = self
            .tables
            .values()
            .filter_map(|t| t.size_bytes)
            .sum();

        let total_row_count: u64 = self
            .tables
            .values()
            .filter_map(|t| t.row_count)
            .sum();

        let datasets_by_status: HashMap<String, usize> = self
            .datasets
            .values()
            .fold(HashMap::new(), |mut acc, ds| {
                *acc.entry(format!("{:?}", ds.status)).or_insert(0) += 1;
                acc
            });

        CatalogStatistics {
            total_datasets: self.datasets.len(),
            total_tables: self.tables.len(),
            total_size_bytes,
            total_row_count,
            datasets_by_status,
            unique_tags: self.tag_index.len(),
        }
    }
}

/// Catalog statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogStatistics {
    pub total_datasets: usize,
    pub total_tables: usize,
    pub total_size_bytes: u64,
    pub total_row_count: u64,
    pub datasets_by_status: HashMap<String, usize>,
    pub unique_tags: usize,
}

impl Default for DataCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_creation() {
        let catalog = DataCatalog::new();
        assert_eq!(catalog.list_datasets().len(), 0);
    }

    #[test]
    fn test_register_dataset() {
        let mut catalog = DataCatalog::new();
        let metadata = DatasetMetadata {
            id: Uuid::new_v4(),
            name: "test_dataset".to_string(),
            description: "Test dataset".to_string(),
            owner: "test_owner".to_string(),
            steward: None,
            domain: "geospatial".to_string(),
            source_system: "test_system".to_string(),
            location: "/data/test".to_string(),
            format: DatasetFormat::Parquet,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed: None,
            tags: HashSet::new(),
            properties: HashMap::new(),
            tables: Vec::new(),
            classification: None,
            quality_score: None,
            status: DatasetStatus::Active,
        };

        catalog.register_dataset(metadata).unwrap();
        assert_eq!(catalog.list_datasets().len(), 1);
    }
}
