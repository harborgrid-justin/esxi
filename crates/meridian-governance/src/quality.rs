//! Data quality rules and validation framework

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Data quality manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityManager {
    /// Quality rules by rule ID
    rules: HashMap<String, QualityRule>,
    /// Rule results cache
    results: HashMap<String, QualityResult>,
}

/// Data quality rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRule {
    /// Unique rule identifier
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Target dataset/table
    pub target: String,
    /// Rule category
    pub category: QualityCategory,
    /// Rule type and configuration
    pub rule_type: RuleType,
    /// Severity level
    pub severity: Severity,
    /// Whether rule is enabled
    pub enabled: bool,
    /// Rule owner
    pub owner: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Quality rule category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum QualityCategory {
    /// Completeness checks (null values, missing data)
    Completeness,
    /// Accuracy checks (correct values)
    Accuracy,
    /// Consistency checks (referential integrity, cross-field validation)
    Consistency,
    /// Validity checks (format, range, domain)
    Validity,
    /// Uniqueness checks (duplicates)
    Uniqueness,
    /// Timeliness checks (data freshness)
    Timeliness,
    /// Conformity checks (schema compliance)
    Conformity,
    /// Custom category
    Custom(String),
}

/// Rule type and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    /// Check for null/missing values
    NotNull {
        /// Field to check
        field: String,
        /// Maximum allowed null percentage (0.0 - 1.0)
        max_null_percentage: f64,
    },
    /// Check value range
    Range {
        /// Field to check
        field: String,
        /// Minimum value (inclusive)
        min: Option<f64>,
        /// Maximum value (inclusive)
        max: Option<f64>,
    },
    /// Check against allowed values
    AllowedValues {
        /// Field to check
        field: String,
        /// Allowed values
        values: Vec<String>,
    },
    /// Check format using regex
    Format {
        /// Field to check
        field: String,
        /// Regex pattern
        pattern: String,
    },
    /// Check uniqueness
    Unique {
        /// Fields that must be unique
        fields: Vec<String>,
    },
    /// Check referential integrity
    ForeignKey {
        /// Source field
        source_field: String,
        /// Referenced table
        referenced_table: String,
        /// Referenced field
        referenced_field: String,
    },
    /// Custom SQL expression
    CustomSql {
        /// SQL expression that should evaluate to true
        expression: String,
    },
    /// Row count threshold
    RowCount {
        /// Minimum row count
        min: Option<u64>,
        /// Maximum row count
        max: Option<u64>,
    },
    /// Data freshness check
    Freshness {
        /// Timestamp field to check
        timestamp_field: String,
        /// Maximum age in seconds
        max_age_seconds: i64,
    },
    /// Aggregate check (sum, avg, etc.)
    Aggregate {
        /// Field to aggregate
        field: String,
        /// Aggregation function
        function: AggregateFunction,
        /// Expected value
        expected: f64,
        /// Tolerance (absolute)
        tolerance: f64,
    },
    /// Custom validation function
    Custom {
        /// Validation logic description
        description: String,
        /// Configuration parameters
        config: HashMap<String, Value>,
    },
}

/// Aggregate function type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AggregateFunction {
    Sum,
    Avg,
    Min,
    Max,
    Count,
    Median,
    StdDev,
}

/// Rule severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Quality validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityResult {
    /// Rule ID
    pub rule_id: String,
    /// Target dataset/table
    pub target: String,
    /// Validation status
    pub status: ValidationStatus,
    /// Execution timestamp
    pub executed_at: DateTime<Utc>,
    /// Number of records checked
    pub records_checked: u64,
    /// Number of failed records
    pub records_failed: u64,
    /// Failure percentage
    pub failure_percentage: f64,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Detailed metrics
    pub metrics: HashMap<String, f64>,
    /// Sample failed records
    pub sample_failures: Vec<Value>,
}

/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Validation passed
    Passed,
    /// Validation failed
    Failed,
    /// Validation warning
    Warning,
    /// Validation skipped
    Skipped,
    /// Error during validation
    Error,
}

/// Quality scorecard for a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScorecard {
    /// Dataset identifier
    pub dataset: String,
    /// Overall quality score (0.0 - 1.0)
    pub overall_score: f64,
    /// Scores by category
    pub category_scores: HashMap<QualityCategory, f64>,
    /// Total rules evaluated
    pub total_rules: usize,
    /// Passed rules
    pub passed_rules: usize,
    /// Failed rules
    pub failed_rules: usize,
    /// Warning rules
    pub warning_rules: usize,
    /// Critical failures
    pub critical_failures: usize,
    /// Evaluation timestamp
    pub evaluated_at: DateTime<Utc>,
}

/// Quality trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Quality score
    pub score: f64,
    /// Number of rules
    pub rule_count: usize,
    /// Number of failures
    pub failure_count: usize,
}

impl QualityManager {
    /// Create a new quality manager
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            results: HashMap::new(),
        }
    }

    /// Add a quality rule
    pub fn add_rule(&mut self, rule: QualityRule) -> Result<()> {
        if self.rules.contains_key(&rule.id) {
            return Err(GovernanceError::QualityRule(format!(
                "Rule already exists: {}",
                rule.id
            )));
        }

        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Get a quality rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Result<&QualityRule> {
        self.rules.get(rule_id).ok_or_else(|| {
            GovernanceError::QualityRule(format!("Rule not found: {}", rule_id))
        })
    }

    /// Update a quality rule
    pub fn update_rule(&mut self, rule: QualityRule) -> Result<()> {
        if !self.rules.contains_key(&rule.id) {
            return Err(GovernanceError::QualityRule(format!(
                "Rule not found: {}",
                rule.id
            )));
        }

        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Delete a quality rule
    pub fn delete_rule(&mut self, rule_id: &str) -> Result<QualityRule> {
        self.rules.remove(rule_id).ok_or_else(|| {
            GovernanceError::QualityRule(format!("Rule not found: {}", rule_id))
        })
    }

    /// Enable a rule
    pub fn enable_rule(&mut self, rule_id: &str) -> Result<()> {
        let rule = self.rules.get_mut(rule_id).ok_or_else(|| {
            GovernanceError::QualityRule(format!("Rule not found: {}", rule_id))
        })?;

        rule.enabled = true;
        Ok(())
    }

    /// Disable a rule
    pub fn disable_rule(&mut self, rule_id: &str) -> Result<()> {
        let rule = self.rules.get_mut(rule_id).ok_or_else(|| {
            GovernanceError::QualityRule(format!("Rule not found: {}", rule_id))
        })?;

        rule.enabled = false;
        Ok(())
    }

    /// Get all rules for a target
    pub fn get_rules_for_target(&self, target: &str) -> Vec<&QualityRule> {
        self.rules
            .values()
            .filter(|rule| rule.target == target)
            .collect()
    }

    /// Get enabled rules for a target
    pub fn get_enabled_rules_for_target(&self, target: &str) -> Vec<&QualityRule> {
        self.rules
            .values()
            .filter(|rule| rule.target == target && rule.enabled)
            .collect()
    }

    /// Get rules by category
    pub fn get_rules_by_category(&self, category: &QualityCategory) -> Vec<&QualityRule> {
        self.rules
            .values()
            .filter(|rule| &rule.category == category)
            .collect()
    }

    /// Get rules by severity
    pub fn get_rules_by_severity(&self, severity: &Severity) -> Vec<&QualityRule> {
        self.rules
            .values()
            .filter(|rule| &rule.severity == severity)
            .collect()
    }

    /// Store a validation result
    pub fn store_result(&mut self, result: QualityResult) {
        let key = format!("{}:{}", result.target, result.rule_id);
        self.results.insert(key, result);
    }

    /// Get validation result
    pub fn get_result(&self, target: &str, rule_id: &str) -> Option<&QualityResult> {
        let key = format!("{}:{}", target, rule_id);
        self.results.get(&key)
    }

    /// Get all results for a target
    pub fn get_results_for_target(&self, target: &str) -> Vec<&QualityResult> {
        self.results
            .values()
            .filter(|result| result.target == target)
            .collect()
    }

    /// Calculate quality scorecard for a target
    pub fn calculate_scorecard(&self, target: &str) -> Result<QualityScorecard> {
        let results = self.get_results_for_target(target);

        if results.is_empty() {
            return Err(GovernanceError::quality_validation(format!(
                "No quality results found for target: {}",
                target
            )));
        }

        let total_rules = results.len();
        let passed_rules = results
            .iter()
            .filter(|r| r.status == ValidationStatus::Passed)
            .count();
        let failed_rules = results
            .iter()
            .filter(|r| r.status == ValidationStatus::Failed)
            .count();
        let warning_rules = results
            .iter()
            .filter(|r| r.status == ValidationStatus::Warning)
            .count();

        // Count critical failures
        let critical_failures = results
            .iter()
            .filter(|r| {
                r.status == ValidationStatus::Failed
                    && self
                        .get_rule(&r.rule_id)
                        .map(|rule| rule.severity == Severity::Critical)
                        .unwrap_or(false)
            })
            .count();

        // Calculate overall score
        let overall_score = if total_rules > 0 {
            passed_rules as f64 / total_rules as f64
        } else {
            0.0
        };

        // Calculate category scores
        let mut category_scores = HashMap::new();
        for category in [
            QualityCategory::Completeness,
            QualityCategory::Accuracy,
            QualityCategory::Consistency,
            QualityCategory::Validity,
            QualityCategory::Uniqueness,
            QualityCategory::Timeliness,
            QualityCategory::Conformity,
        ] {
            let category_results: Vec<_> = results
                .iter()
                .filter(|r| {
                    self.get_rule(&r.rule_id)
                        .map(|rule| rule.category == category)
                        .unwrap_or(false)
                })
                .collect();

            if !category_results.is_empty() {
                let passed = category_results
                    .iter()
                    .filter(|r| r.status == ValidationStatus::Passed)
                    .count();
                let score = passed as f64 / category_results.len() as f64;
                category_scores.insert(category, score);
            }
        }

        Ok(QualityScorecard {
            dataset: target.to_string(),
            overall_score,
            category_scores,
            total_rules,
            passed_rules,
            failed_rules,
            warning_rules,
            critical_failures,
            evaluated_at: Utc::now(),
        })
    }

    /// Validate data using a specific rule (mock implementation)
    pub fn validate(&mut self, rule_id: &str, data: &Value) -> Result<QualityResult> {
        let rule = self.get_rule(rule_id)?.clone();

        if !rule.enabled {
            return Ok(QualityResult {
                rule_id: rule.id.clone(),
                target: rule.target.clone(),
                status: ValidationStatus::Skipped,
                executed_at: Utc::now(),
                records_checked: 0,
                records_failed: 0,
                failure_percentage: 0.0,
                error_message: Some("Rule is disabled".to_string()),
                metrics: HashMap::new(),
                sample_failures: Vec::new(),
            });
        }

        // Mock validation - in production, this would execute actual validation logic
        let result = QualityResult {
            rule_id: rule.id.clone(),
            target: rule.target.clone(),
            status: ValidationStatus::Passed,
            executed_at: Utc::now(),
            records_checked: 1,
            records_failed: 0,
            failure_percentage: 0.0,
            error_message: None,
            metrics: HashMap::new(),
            sample_failures: Vec::new(),
        };

        self.store_result(result.clone());
        Ok(result)
    }

    /// List all rules
    pub fn list_rules(&self) -> Vec<&QualityRule> {
        self.rules.values().collect()
    }
}

impl Default for QualityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_manager_creation() {
        let manager = QualityManager::new();
        assert_eq!(manager.list_rules().len(), 0);
    }

    #[test]
    fn test_add_rule() {
        let mut manager = QualityManager::new();
        let rule = QualityRule {
            id: "rule1".to_string(),
            name: "Not Null Check".to_string(),
            description: "Check for null values".to_string(),
            target: "test_table".to_string(),
            category: QualityCategory::Completeness,
            rule_type: RuleType::NotNull {
                field: "id".to_string(),
                max_null_percentage: 0.0,
            },
            severity: Severity::Error,
            enabled: true,
            owner: "test_owner".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["critical".to_string()],
        };

        manager.add_rule(rule).unwrap();
        assert_eq!(manager.list_rules().len(), 1);
    }
}
