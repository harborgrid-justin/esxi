//! Schema registry and evolution management

use crate::catalog::{DataType, FieldMetadata, TableSchema};
use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema registry for managing schema versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRegistry {
    /// Schemas by subject name
    schemas: HashMap<String, Vec<SchemaVersion>>,
    /// Schema compatibility mode
    compatibility_mode: CompatibilityMode,
}

/// Schema version entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Version number
    pub version: u32,
    /// Schema definition
    pub schema: TableSchema,
    /// Schema hash for deduplication
    pub schema_hash: String,
    /// Registered timestamp
    pub registered_at: DateTime<Utc>,
    /// Registered by
    pub registered_by: String,
    /// Schema state
    pub state: SchemaState,
    /// Description of changes
    pub change_description: Option<String>,
    /// Compatibility with previous version
    pub compatibility: Option<CompatibilityResult>,
}

/// Schema state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaState {
    Active,
    Deprecated,
    Disabled,
}

/// Compatibility mode for schema evolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityMode {
    /// No compatibility checks
    None,
    /// New schema can read data written by old schema
    Backward,
    /// Old schema can read data written by new schema
    Forward,
    /// Both backward and forward compatible
    Full,
    /// All versions must be compatible with each other
    Transitive,
}

/// Schema compatibility check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    /// Whether schemas are compatible
    pub compatible: bool,
    /// Compatibility issues
    pub issues: Vec<CompatibilityIssue>,
    /// Compatibility mode used
    pub mode: CompatibilityMode,
}

/// Compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue type
    pub issue_type: IssueType,
    /// Field name (if applicable)
    pub field: Option<String>,
    /// Issue description
    pub description: String,
}

/// Issue severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Breaking,
}

/// Compatibility issue type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueType {
    FieldAdded,
    FieldRemoved,
    FieldTypeChanged,
    FieldRenamed,
    NullabilityChanged,
    DefaultValueChanged,
    PrimaryKeyChanged,
    ForeignKeyAdded,
    ForeignKeyRemoved,
    ConstraintAdded,
    ConstraintRemoved,
}

/// Schema change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// Change identifier
    pub id: String,
    /// Subject name
    pub subject: String,
    /// Old version
    pub old_version: Option<u32>,
    /// New version
    pub new_version: u32,
    /// Change type
    pub change_type: ChangeType,
    /// Changes detected
    pub changes: Vec<FieldChange>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Changed by
    pub changed_by: String,
}

/// Type of schema change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    Initial,
    Evolution,
    Breaking,
    Deprecation,
}

/// Field-level change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    /// Field name
    pub field: String,
    /// Change operation
    pub operation: FieldOperation,
    /// Old value (if applicable)
    pub old_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
}

/// Field operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FieldOperation {
    Add,
    Remove,
    Modify,
    Rename,
}

impl SchemaRegistry {
    /// Create a new schema registry
    pub fn new() -> Self {
        Self::with_compatibility_mode(CompatibilityMode::Backward)
    }

    /// Create a registry with specified compatibility mode
    pub fn with_compatibility_mode(mode: CompatibilityMode) -> Self {
        Self {
            schemas: HashMap::new(),
            compatibility_mode: mode,
        }
    }

    /// Register a new schema version
    pub fn register(
        &mut self,
        subject: String,
        schema: TableSchema,
        registered_by: String,
        change_description: Option<String>,
    ) -> Result<u32> {
        let schema_hash = Self::calculate_hash(&schema);

        // Check if schema already exists
        if let Some(versions) = self.schemas.get(&subject) {
            // Check for duplicate schema
            if let Some(existing) = versions.iter().find(|v| v.schema_hash == schema_hash) {
                return Err(GovernanceError::SchemaVersionConflict(format!(
                    "Schema already registered as version {}",
                    existing.version
                )));
            }

            // Check compatibility with latest version
            let latest = versions.last().unwrap();
            let compatibility = self.check_compatibility(&latest.schema, &schema)?;

            if !compatibility.compatible {
                let breaking_issues: Vec<_> = compatibility
                    .issues
                    .iter()
                    .filter(|i| i.severity == IssueSeverity::Breaking)
                    .collect();

                if !breaking_issues.is_empty() {
                    return Err(GovernanceError::IncompatibleSchemaChange(format!(
                        "Breaking changes detected: {:?}",
                        breaking_issues
                    )));
                }
            }

            let new_version = latest.version + 1;
            let version_entry = SchemaVersion {
                version: new_version,
                schema,
                schema_hash,
                registered_at: Utc::now(),
                registered_by,
                state: SchemaState::Active,
                change_description,
                compatibility: Some(compatibility),
            };

            self.schemas.get_mut(&subject).unwrap().push(version_entry);
            Ok(new_version)
        } else {
            // First version
            let version_entry = SchemaVersion {
                version: 1,
                schema,
                schema_hash,
                registered_at: Utc::now(),
                registered_by,
                state: SchemaState::Active,
                change_description,
                compatibility: None,
            };

            self.schemas.insert(subject, vec![version_entry]);
            Ok(1)
        }
    }

    /// Get latest schema version for a subject
    pub fn get_latest(&self, subject: &str) -> Result<&SchemaVersion> {
        self.schemas
            .get(subject)
            .and_then(|versions| versions.last())
            .ok_or_else(|| GovernanceError::SchemaNotFound(subject.to_string()))
    }

    /// Get specific schema version
    pub fn get_version(&self, subject: &str, version: u32) -> Result<&SchemaVersion> {
        self.schemas
            .get(subject)
            .and_then(|versions| versions.iter().find(|v| v.version == version))
            .ok_or_else(|| {
                GovernanceError::SchemaNotFound(format!("{} version {}", subject, version))
            })
    }

    /// Get all versions for a subject
    pub fn get_all_versions(&self, subject: &str) -> Result<&Vec<SchemaVersion>> {
        self.schemas
            .get(subject)
            .ok_or_else(|| GovernanceError::SchemaNotFound(subject.to_string()))
    }

    /// Delete a schema version
    pub fn delete_version(&mut self, subject: &str, version: u32) -> Result<()> {
        let versions = self.schemas.get_mut(subject).ok_or_else(|| {
            GovernanceError::SchemaNotFound(subject.to_string())
        })?;

        let index = versions
            .iter()
            .position(|v| v.version == version)
            .ok_or_else(|| {
                GovernanceError::SchemaNotFound(format!("{} version {}", subject, version))
            })?;

        versions.remove(index);

        if versions.is_empty() {
            self.schemas.remove(subject);
        }

        Ok(())
    }

    /// Check compatibility between two schemas
    pub fn check_compatibility(
        &self,
        old_schema: &TableSchema,
        new_schema: &TableSchema,
    ) -> Result<CompatibilityResult> {
        let mut issues = Vec::new();

        // Create field maps for easier comparison
        let old_fields: HashMap<_, _> = old_schema
            .fields
            .iter()
            .map(|f| (f.name.clone(), f))
            .collect();
        let new_fields: HashMap<_, _> = new_schema
            .fields
            .iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        // Check for removed fields
        for (name, old_field) in &old_fields {
            if !new_fields.contains_key(name) {
                issues.push(CompatibilityIssue {
                    severity: if old_field.nullable {
                        IssueSeverity::Warning
                    } else {
                        IssueSeverity::Breaking
                    },
                    issue_type: IssueType::FieldRemoved,
                    field: Some(name.clone()),
                    description: format!("Field '{}' was removed", name),
                });
            }
        }

        // Check for added and modified fields
        for (name, new_field) in &new_fields {
            if let Some(old_field) = old_fields.get(name) {
                // Check type changes
                if old_field.data_type != new_field.data_type {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Breaking,
                        issue_type: IssueType::FieldTypeChanged,
                        field: Some(name.clone()),
                        description: format!(
                            "Field '{}' type changed from {:?} to {:?}",
                            name, old_field.data_type, new_field.data_type
                        ),
                    });
                }

                // Check nullability changes
                if old_field.nullable && !new_field.nullable {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Breaking,
                        issue_type: IssueType::NullabilityChanged,
                        field: Some(name.clone()),
                        description: format!(
                            "Field '{}' changed from nullable to non-nullable",
                            name
                        ),
                    });
                }
            } else {
                // New field added
                let severity = if new_field.nullable || new_field.default_value.is_some() {
                    IssueSeverity::Info
                } else {
                    IssueSeverity::Breaking
                };

                issues.push(CompatibilityIssue {
                    severity,
                    issue_type: IssueType::FieldAdded,
                    field: Some(name.clone()),
                    description: format!("Field '{}' was added", name),
                });
            }
        }

        // Check primary key changes
        if old_schema.primary_keys != new_schema.primary_keys {
            issues.push(CompatibilityIssue {
                severity: IssueSeverity::Breaking,
                issue_type: IssueType::PrimaryKeyChanged,
                field: None,
                description: "Primary key definition changed".to_string(),
            });
        }

        let compatible = !issues.iter().any(|i| i.severity == IssueSeverity::Breaking);

        Ok(CompatibilityResult {
            compatible,
            issues,
            mode: self.compatibility_mode.clone(),
        })
    }

    /// Calculate schema hash
    fn calculate_hash(schema: &TableSchema) -> String {
        use sha2::{Digest, Sha256};
        let serialized = serde_json::to_string(schema).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// List all subjects
    pub fn list_subjects(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// Set compatibility mode
    pub fn set_compatibility_mode(&mut self, mode: CompatibilityMode) {
        self.compatibility_mode = mode;
    }

    /// Get registry statistics
    pub fn get_statistics(&self) -> RegistryStatistics {
        let total_subjects = self.schemas.len();
        let mut total_versions = 0;
        let mut active_schemas = 0;
        let mut deprecated_schemas = 0;

        for versions in self.schemas.values() {
            total_versions += versions.len();
            if let Some(latest) = versions.last() {
                match latest.state {
                    SchemaState::Active => active_schemas += 1,
                    SchemaState::Deprecated => deprecated_schemas += 1,
                    _ => {}
                }
            }
        }

        RegistryStatistics {
            total_subjects,
            total_versions,
            active_schemas,
            deprecated_schemas,
            compatibility_mode: self.compatibility_mode.clone(),
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    pub total_subjects: usize,
    pub total_versions: usize,
    pub active_schemas: usize,
    pub deprecated_schemas: usize,
    pub compatibility_mode: CompatibilityMode,
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_schema_registry_creation() {
        let registry = SchemaRegistry::new();
        assert_eq!(registry.list_subjects().len(), 0);
    }

    #[test]
    fn test_register_schema() {
        let mut registry = SchemaRegistry::new();
        let schema = TableSchema {
            fields: vec![FieldMetadata {
                name: "id".to_string(),
                data_type: DataType::Int64,
                description: None,
                nullable: false,
                default_value: None,
                is_primary_key: true,
                is_indexed: true,
                business_term: None,
                classification: None,
                tags: HashSet::new(),
                properties: HashMap::new(),
            }],
            primary_keys: vec!["id".to_string()],
            foreign_keys: vec![],
            unique_constraints: vec![],
        };

        let version = registry
            .register(
                "test_table".to_string(),
                schema,
                "test_user".to_string(),
                Some("Initial schema".to_string()),
            )
            .unwrap();

        assert_eq!(version, 1);
    }
}
