//! # Meridian Governance
//!
//! Data governance, lineage tracking, and compliance management for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **Data Catalog**: Comprehensive metadata management for datasets, tables, and fields
//! - **Lineage Tracking**: DAG-based data lineage from source to sink
//! - **Data Quality**: Rules, validation, and quality scoring
//! - **Classification**: Sensitivity labels and data classification
//! - **Compliance**: GDPR, CCPA, SOC2 compliance frameworks
//! - **Retention Policies**: Data lifecycle management
//! - **Audit Trails**: Complete access audit logging
//! - **Data Masking**: Anonymization and masking strategies
//! - **Schema Registry**: Version control for schemas
//! - **Impact Analysis**: Change impact assessment
//! - **Stewardship**: Workflow management for data stewards
//! - **Business Glossary**: Terms, definitions, and data dictionary
//!
//! ## Example
//!
//! ```rust
//! use meridian_governance::{
//!     catalog::DataCatalog,
//!     lineage::LineageTracker,
//!     quality::QualityManager,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a data catalog
//! let mut catalog = DataCatalog::new();
//!
//! // Create a lineage tracker
//! let mut lineage = LineageTracker::new();
//!
//! // Create a quality manager
//! let mut quality = QualityManager::new();
//!
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod audit;
pub mod catalog;
pub mod classification;
pub mod compliance;
pub mod error;
pub mod glossary;
pub mod impact;
pub mod lineage;
pub mod masking;
pub mod quality;
pub mod retention;
pub mod schema;
pub mod stewardship;

// Re-export commonly used types
pub use error::{GovernanceError, Result};

/// Governance platform version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Governance platform facade providing integrated access to all governance functions
#[derive(Debug)]
pub struct GovernancePlatform {
    /// Data catalog
    pub catalog: catalog::DataCatalog,
    /// Lineage tracker
    pub lineage: lineage::LineageTracker,
    /// Quality manager
    pub quality: quality::QualityManager,
    /// Classification manager
    pub classification: classification::ClassificationManager,
    /// Compliance manager
    pub compliance: compliance::ComplianceManager,
    /// Retention manager
    pub retention: retention::RetentionManager,
    /// Audit manager
    pub audit: audit::AuditManager,
    /// Masking manager
    pub masking: masking::MaskingManager,
    /// Schema registry
    pub schema_registry: schema::SchemaRegistry,
    /// Stewardship manager
    pub stewardship: stewardship::StewardshipManager,
    /// Business glossary
    pub glossary: glossary::BusinessGlossary,
}

impl GovernancePlatform {
    /// Create a new governance platform instance
    pub fn new() -> Self {
        Self {
            catalog: catalog::DataCatalog::new(),
            lineage: lineage::LineageTracker::new(),
            quality: quality::QualityManager::new(),
            classification: classification::ClassificationManager::new(),
            compliance: compliance::ComplianceManager::new(),
            retention: retention::RetentionManager::new(),
            audit: audit::AuditManager::new(),
            masking: masking::MaskingManager::new(),
            schema_registry: schema::SchemaRegistry::new(),
            stewardship: stewardship::StewardshipManager::new(),
            glossary: glossary::BusinessGlossary::new(),
        }
    }

    /// Create an impact analyzer
    pub fn create_impact_analyzer(&self) -> impact::ImpactAnalyzer {
        impact::ImpactAnalyzer::new(
            self.lineage.clone(),
            self.schema_registry.clone(),
        )
    }

    /// Get platform version
    pub fn version(&self) -> &str {
        VERSION
    }
}

impl Default for GovernancePlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_creation() {
        let platform = GovernancePlatform::new();
        assert_eq!(platform.version(), VERSION);
    }

    #[test]
    fn test_integrated_workflow() {
        use catalog::{DatasetFormat, DatasetMetadata, DatasetStatus};
        use chrono::Utc;
        use std::collections::{HashMap, HashSet};
        use uuid::Uuid;

        let mut platform = GovernancePlatform::new();

        // Register a dataset in the catalog
        let dataset = DatasetMetadata {
            id: Uuid::new_v4(),
            name: "customer_data".to_string(),
            description: "Customer information dataset".to_string(),
            owner: "data_team".to_string(),
            steward: Some("steward1".to_string()),
            domain: "customer".to_string(),
            source_system: "crm".to_string(),
            location: "/data/customers".to_string(),
            format: DatasetFormat::Parquet,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed: None,
            tags: HashSet::new(),
            properties: HashMap::new(),
            tables: Vec::new(),
            classification: Some("confidential".to_string()),
            quality_score: None,
            status: DatasetStatus::Active,
        };

        platform.catalog.register_dataset(dataset).unwrap();

        // Classify the dataset
        platform
            .classification
            .classify(
                "customer_data".to_string(),
                classification::EntityType::Dataset,
                "confidential".to_string(),
                "data_steward".to_string(),
                Some("Contains PII".to_string()),
            )
            .unwrap();

        // Verify classification
        let classification = platform
            .classification
            .get_classification("customer_data")
            .unwrap();
        assert_eq!(classification.level, "confidential");

        // Verify dataset is in catalog
        let retrieved = platform.catalog.get_dataset("customer_data").unwrap();
        assert_eq!(retrieved.name, "customer_data");
    }
}
