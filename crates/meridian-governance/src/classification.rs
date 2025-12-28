//! Data classification and sensitivity labeling system

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Data classification manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationManager {
    /// Defined classification levels
    levels: HashMap<String, ClassificationLevel>,
    /// Classification assignments
    assignments: HashMap<String, DataClassification>,
    /// Auto-classification rules
    auto_rules: Vec<AutoClassificationRule>,
}

/// Classification level definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationLevel {
    /// Level identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Sensitivity rank (higher = more sensitive)
    pub sensitivity_rank: u8,
    /// Color code for UI display
    pub color: String,
    /// Required access controls
    pub access_controls: Vec<String>,
    /// Allowed operations
    pub allowed_operations: Vec<Operation>,
    /// Encryption requirement
    pub requires_encryption: bool,
    /// Audit logging requirement
    pub requires_audit: bool,
    /// Data masking requirement
    pub requires_masking: bool,
    /// Retention requirements
    pub retention_requirements: Option<String>,
}

/// Data classification assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataClassification {
    /// Entity identifier (dataset, table, field)
    pub entity_id: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Classification level
    pub level: String,
    /// Assigned by
    pub assigned_by: String,
    /// Assignment timestamp
    pub assigned_at: DateTime<Utc>,
    /// Justification/reason
    pub justification: Option<String>,
    /// Review date
    pub review_date: Option<DateTime<Utc>>,
    /// Tags for categorization
    pub tags: HashSet<String>,
    /// Custom properties
    pub properties: HashMap<String, String>,
}

/// Entity type for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    Dataset,
    Table,
    Field,
    Column,
    View,
    Report,
}

/// Allowed operations based on classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Operation {
    Read,
    Write,
    Update,
    Delete,
    Export,
    Share,
    Print,
    Download,
    Copy,
}

/// Auto-classification rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoClassificationRule {
    /// Rule identifier
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Classification level to assign
    pub target_level: String,
    /// Rule conditions
    pub conditions: Vec<RuleCondition>,
    /// Whether rule is enabled
    pub enabled: bool,
    /// Priority (higher = evaluated first)
    pub priority: u8,
}

/// Classification rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Match field name pattern
    FieldNamePattern { pattern: String },
    /// Match table name pattern
    TableNamePattern { pattern: String },
    /// Match data type
    DataType { data_type: String },
    /// Contains keyword
    ContainsKeyword { keyword: String },
    /// Has tag
    HasTag { tag: String },
    /// Custom condition
    Custom { expression: String },
}

/// Predefined sensitivity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SensitivityLevel {
    /// Public data, no restrictions
    Public,
    /// Internal use only
    Internal,
    /// Confidential data
    Confidential,
    /// Restricted data (PII, PHI)
    Restricted,
    /// Highly restricted (financial, legal)
    HighlyRestricted,
}

/// Classification recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRecommendation {
    /// Entity identifier
    pub entity_id: String,
    /// Recommended level
    pub recommended_level: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Matching rules
    pub matching_rules: Vec<String>,
    /// Reasoning
    pub reasoning: String,
}

impl ClassificationManager {
    /// Create a new classification manager
    pub fn new() -> Self {
        let mut manager = Self {
            levels: HashMap::new(),
            assignments: HashMap::new(),
            auto_rules: Vec::new(),
        };

        // Initialize default classification levels
        manager.initialize_default_levels();

        manager
    }

    /// Initialize default classification levels
    fn initialize_default_levels(&mut self) {
        let levels = vec![
            ClassificationLevel {
                id: "public".to_string(),
                name: "Public".to_string(),
                description: "Data that can be freely shared".to_string(),
                sensitivity_rank: 1,
                color: "#28a745".to_string(),
                access_controls: vec![],
                allowed_operations: vec![
                    Operation::Read,
                    Operation::Write,
                    Operation::Export,
                    Operation::Share,
                    Operation::Print,
                    Operation::Download,
                    Operation::Copy,
                ],
                requires_encryption: false,
                requires_audit: false,
                requires_masking: false,
                retention_requirements: None,
            },
            ClassificationLevel {
                id: "internal".to_string(),
                name: "Internal".to_string(),
                description: "Internal use only data".to_string(),
                sensitivity_rank: 2,
                color: "#007bff".to_string(),
                access_controls: vec!["authenticated".to_string()],
                allowed_operations: vec![
                    Operation::Read,
                    Operation::Write,
                    Operation::Export,
                    Operation::Download,
                ],
                requires_encryption: false,
                requires_audit: true,
                requires_masking: false,
                retention_requirements: None,
            },
            ClassificationLevel {
                id: "confidential".to_string(),
                name: "Confidential".to_string(),
                description: "Confidential business data".to_string(),
                sensitivity_rank: 3,
                color: "#ffc107".to_string(),
                access_controls: vec!["authenticated".to_string(), "authorized".to_string()],
                allowed_operations: vec![Operation::Read, Operation::Write],
                requires_encryption: true,
                requires_audit: true,
                requires_masking: false,
                retention_requirements: Some("7 years".to_string()),
            },
            ClassificationLevel {
                id: "restricted".to_string(),
                name: "Restricted".to_string(),
                description: "Restricted data (PII, PHI)".to_string(),
                sensitivity_rank: 4,
                color: "#fd7e14".to_string(),
                access_controls: vec![
                    "authenticated".to_string(),
                    "authorized".to_string(),
                    "need_to_know".to_string(),
                ],
                allowed_operations: vec![Operation::Read],
                requires_encryption: true,
                requires_audit: true,
                requires_masking: true,
                retention_requirements: Some("As required by regulation".to_string()),
            },
            ClassificationLevel {
                id: "highly_restricted".to_string(),
                name: "Highly Restricted".to_string(),
                description: "Highly sensitive data (financial, legal)".to_string(),
                sensitivity_rank: 5,
                color: "#dc3545".to_string(),
                access_controls: vec![
                    "authenticated".to_string(),
                    "authorized".to_string(),
                    "need_to_know".to_string(),
                    "executive_approval".to_string(),
                ],
                allowed_operations: vec![Operation::Read],
                requires_encryption: true,
                requires_audit: true,
                requires_masking: true,
                retention_requirements: Some("As required by law".to_string()),
            },
        ];

        for level in levels {
            self.levels.insert(level.id.clone(), level);
        }
    }

    /// Add a custom classification level
    pub fn add_level(&mut self, level: ClassificationLevel) -> Result<()> {
        if self.levels.contains_key(&level.id) {
            return Err(GovernanceError::Classification(format!(
                "Classification level already exists: {}",
                level.id
            )));
        }

        self.levels.insert(level.id.clone(), level);
        Ok(())
    }

    /// Get a classification level
    pub fn get_level(&self, level_id: &str) -> Result<&ClassificationLevel> {
        self.levels.get(level_id).ok_or_else(|| {
            GovernanceError::InvalidSensitivityLevel(level_id.to_string())
        })
    }

    /// Classify an entity
    pub fn classify(
        &mut self,
        entity_id: String,
        entity_type: EntityType,
        level: String,
        assigned_by: String,
        justification: Option<String>,
    ) -> Result<()> {
        // Verify level exists
        if !self.levels.contains_key(&level) {
            return Err(GovernanceError::InvalidSensitivityLevel(level));
        }

        let classification = DataClassification {
            entity_id: entity_id.clone(),
            entity_type,
            level,
            assigned_by,
            assigned_at: Utc::now(),
            justification,
            review_date: None,
            tags: HashSet::new(),
            properties: HashMap::new(),
        };

        self.assignments.insert(entity_id, classification);
        Ok(())
    }

    /// Get classification for an entity
    pub fn get_classification(&self, entity_id: &str) -> Option<&DataClassification> {
        self.assignments.get(entity_id)
    }

    /// Remove classification from an entity
    pub fn unclassify(&mut self, entity_id: &str) -> Result<DataClassification> {
        self.assignments.remove(entity_id).ok_or_else(|| {
            GovernanceError::Classification(format!(
                "No classification found for entity: {}",
                entity_id
            ))
        })
    }

    /// Get all classifications for a level
    pub fn get_classifications_by_level(&self, level: &str) -> Vec<&DataClassification> {
        self.assignments
            .values()
            .filter(|c| c.level == level)
            .collect()
    }

    /// Get all classifications by entity type
    pub fn get_classifications_by_type(&self, entity_type: &EntityType) -> Vec<&DataClassification> {
        self.assignments
            .values()
            .filter(|c| &c.entity_type == entity_type)
            .collect()
    }

    /// Add an auto-classification rule
    pub fn add_auto_rule(&mut self, rule: AutoClassificationRule) -> Result<()> {
        // Verify target level exists
        if !self.levels.contains_key(&rule.target_level) {
            return Err(GovernanceError::InvalidSensitivityLevel(
                rule.target_level.clone(),
            ));
        }

        self.auto_rules.push(rule);
        // Sort by priority (descending)
        self.auto_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(())
    }

    /// Get recommendation for entity classification
    pub fn get_recommendation(
        &self,
        entity_id: &str,
        entity_name: &str,
        tags: &HashSet<String>,
    ) -> Option<ClassificationRecommendation> {
        let mut matching_rules = Vec::new();
        let mut max_priority = 0;
        let mut recommended_level = None;

        for rule in &self.auto_rules {
            if !rule.enabled {
                continue;
            }

            let mut matches = true;
            for condition in &rule.conditions {
                match condition {
                    RuleCondition::FieldNamePattern { pattern } => {
                        if !entity_name.contains(pattern) {
                            matches = false;
                            break;
                        }
                    }
                    RuleCondition::TableNamePattern { pattern } => {
                        if !entity_name.contains(pattern) {
                            matches = false;
                            break;
                        }
                    }
                    RuleCondition::HasTag { tag } => {
                        if !tags.contains(tag) {
                            matches = false;
                            break;
                        }
                    }
                    RuleCondition::ContainsKeyword { keyword } => {
                        if !entity_name.to_lowercase().contains(&keyword.to_lowercase()) {
                            matches = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if matches && rule.priority >= max_priority {
                max_priority = rule.priority;
                recommended_level = Some(rule.target_level.clone());
                matching_rules.push(rule.id.clone());
            }
        }

        recommended_level.map(|level| {
            let confidence = if max_priority > 0 {
                (max_priority as f64) / 100.0
            } else {
                0.5
            };

            ClassificationRecommendation {
                entity_id: entity_id.to_string(),
                recommended_level: level.clone(),
                confidence,
                matching_rules: matching_rules.clone(),
                reasoning: format!(
                    "Matched {} auto-classification rules with priority {}",
                    matching_rules.len(),
                    max_priority
                ),
            }
        })
    }

    /// Check if operation is allowed for an entity
    pub fn is_operation_allowed(&self, entity_id: &str, operation: &Operation) -> Result<bool> {
        let classification = self.get_classification(entity_id).ok_or_else(|| {
            GovernanceError::Classification(format!(
                "No classification found for entity: {}",
                entity_id
            ))
        })?;

        let level = self.get_level(&classification.level)?;
        Ok(level.allowed_operations.contains(operation))
    }

    /// Get sensitivity rank for an entity
    pub fn get_sensitivity_rank(&self, entity_id: &str) -> Result<u8> {
        let classification = self.get_classification(entity_id).ok_or_else(|| {
            GovernanceError::Classification(format!(
                "No classification found for entity: {}",
                entity_id
            ))
        })?;

        let level = self.get_level(&classification.level)?;
        Ok(level.sensitivity_rank)
    }

    /// List all classification levels
    pub fn list_levels(&self) -> Vec<&ClassificationLevel> {
        let mut levels: Vec<_> = self.levels.values().collect();
        levels.sort_by_key(|l| l.sensitivity_rank);
        levels
    }

    /// List all classifications
    pub fn list_classifications(&self) -> Vec<&DataClassification> {
        self.assignments.values().collect()
    }

    /// Get classification statistics
    pub fn get_statistics(&self) -> ClassificationStatistics {
        let mut by_level = HashMap::new();
        let mut by_type = HashMap::new();

        for classification in self.assignments.values() {
            *by_level.entry(classification.level.clone()).or_insert(0) += 1;
            *by_type
                .entry(format!("{:?}", classification.entity_type))
                .or_insert(0) += 1;
        }

        ClassificationStatistics {
            total_classifications: self.assignments.len(),
            by_level,
            by_type,
            total_levels: self.levels.len(),
            total_auto_rules: self.auto_rules.len(),
        }
    }
}

/// Classification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationStatistics {
    pub total_classifications: usize,
    pub by_level: HashMap<String, usize>,
    pub by_type: HashMap<String, usize>,
    pub total_levels: usize,
    pub total_auto_rules: usize,
}

impl Default for ClassificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_manager_creation() {
        let manager = ClassificationManager::new();
        assert!(manager.list_levels().len() >= 5);
    }

    #[test]
    fn test_classify_entity() {
        let mut manager = ClassificationManager::new();
        manager
            .classify(
                "test_table".to_string(),
                EntityType::Table,
                "confidential".to_string(),
                "test_user".to_string(),
                Some("Contains sensitive data".to_string()),
            )
            .unwrap();

        let classification = manager.get_classification("test_table").unwrap();
        assert_eq!(classification.level, "confidential");
    }
}
