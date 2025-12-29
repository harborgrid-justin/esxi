//! Data retention policies and lifecycle management

use crate::error::{GovernanceError, Result};
use chrono::{Datelike, DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Retention policy manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionManager {
    /// Defined retention policies
    policies: HashMap<String, RetentionPolicy>,
    /// Policy assignments to datasets
    assignments: HashMap<String, String>,
}

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Policy identifier
    pub id: String,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Retention period
    pub retention_period: RetentionPeriod,
    /// Action to take when data expires
    pub expiration_action: ExpirationAction,
    /// Legal or regulatory basis
    pub legal_basis: Option<String>,
    /// Policy owner
    pub owner: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
    /// Whether policy is active
    pub active: bool,
    /// Exemptions or exceptions
    pub exemptions: Vec<String>,
}

/// Retention period specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPeriod {
    /// Fixed duration
    Duration {
        /// Number of days
        days: i64,
    },
    /// Duration from event
    FromEvent {
        /// Event type (e.g., "last_access", "creation", "last_modified")
        event: String,
        /// Number of days after event
        days: i64,
    },
    /// End of year/month/quarter
    EndOfPeriod {
        /// Period type
        period: PeriodType,
        /// Additional years/months/quarters to keep
        additional_periods: i64,
    },
    /// Indefinite retention
    Indefinite,
    /// Custom retention logic
    Custom(String),
}

/// Period type for end-of-period retention
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeriodType {
    Year,
    Quarter,
    Month,
}

/// Action to take when data expires
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpirationAction {
    /// Delete the data permanently
    Delete,
    /// Archive to cold storage
    Archive,
    /// Mark as expired but keep
    MarkExpired,
    /// Move to a specific location
    Move { destination: String },
    /// Anonymize the data
    Anonymize,
    /// Require manual review
    RequireReview,
}

/// Retention status for a dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionStatus {
    /// Dataset identifier
    pub dataset_id: String,
    /// Applied policy ID
    pub policy_id: String,
    /// Creation/baseline date
    pub baseline_date: DateTime<Utc>,
    /// Expiration date
    pub expiration_date: Option<DateTime<Utc>>,
    /// Current status
    pub status: DataLifecycleStatus,
    /// Last reviewed date
    pub last_reviewed: Option<DateTime<Utc>>,
    /// Next review date
    pub next_review: Option<DateTime<Utc>>,
    /// Legal hold applied
    pub legal_hold: bool,
    /// Legal hold reason
    pub legal_hold_reason: Option<String>,
}

/// Data lifecycle status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataLifecycleStatus {
    /// Active data
    Active,
    /// Approaching expiration
    NearExpiration,
    /// Expired and ready for action
    Expired,
    /// Archived
    Archived,
    /// Under legal hold
    LegalHold,
    /// Deleted
    Deleted,
}

/// Legal hold on data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHold {
    /// Hold identifier
    pub id: String,
    /// Hold name
    pub name: String,
    /// Reason for hold
    pub reason: String,
    /// Affected datasets
    pub affected_datasets: Vec<String>,
    /// Hold start date
    pub started_at: DateTime<Utc>,
    /// Hold end date (if known)
    pub ends_at: Option<DateTime<Utc>>,
    /// Responsible party
    pub responsible: String,
    /// Case or matter number
    pub case_number: Option<String>,
}

impl RetentionManager {
    /// Create a new retention manager
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            assignments: HashMap::new(),
        }
    }

    /// Add a retention policy
    pub fn add_policy(&mut self, policy: RetentionPolicy) -> Result<()> {
        if self.policies.contains_key(&policy.id) {
            return Err(GovernanceError::RetentionPolicy(format!(
                "Policy already exists: {}",
                policy.id
            )));
        }

        self.policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// Get a retention policy
    pub fn get_policy(&self, policy_id: &str) -> Result<&RetentionPolicy> {
        self.policies.get(policy_id).ok_or_else(|| {
            GovernanceError::RetentionPolicy(format!("Policy not found: {}", policy_id))
        })
    }

    /// Update a retention policy
    pub fn update_policy(&mut self, policy: RetentionPolicy) -> Result<()> {
        if !self.policies.contains_key(&policy.id) {
            return Err(GovernanceError::RetentionPolicy(format!(
                "Policy not found: {}",
                policy.id
            )));
        }

        self.policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// Delete a retention policy
    pub fn delete_policy(&mut self, policy_id: &str) -> Result<RetentionPolicy> {
        // Check if policy is in use
        let in_use = self.assignments.values().any(|p| p == policy_id);
        if in_use {
            return Err(GovernanceError::RetentionPolicy(format!(
                "Cannot delete policy in use: {}",
                policy_id
            )));
        }

        self.policies.remove(policy_id).ok_or_else(|| {
            GovernanceError::RetentionPolicy(format!("Policy not found: {}", policy_id))
        })
    }

    /// Assign a policy to a dataset
    pub fn assign_policy(&mut self, dataset_id: String, policy_id: String) -> Result<()> {
        // Verify policy exists
        if !self.policies.contains_key(&policy_id) {
            return Err(GovernanceError::RetentionPolicy(format!(
                "Policy not found: {}",
                policy_id
            )));
        }

        self.assignments.insert(dataset_id, policy_id);
        Ok(())
    }

    /// Remove policy assignment from a dataset
    pub fn remove_assignment(&mut self, dataset_id: &str) -> Result<String> {
        self.assignments.remove(dataset_id).ok_or_else(|| {
            GovernanceError::RetentionPolicy(format!(
                "No policy assigned to dataset: {}",
                dataset_id
            ))
        })
    }

    /// Get the policy assigned to a dataset
    pub fn get_assigned_policy(&self, dataset_id: &str) -> Result<&RetentionPolicy> {
        let policy_id = self.assignments.get(dataset_id).ok_or_else(|| {
            GovernanceError::RetentionPolicy(format!(
                "No policy assigned to dataset: {}",
                dataset_id
            ))
        })?;

        self.get_policy(policy_id)
    }

    /// Calculate expiration date for a dataset
    pub fn calculate_expiration(
        &self,
        dataset_id: &str,
        baseline_date: DateTime<Utc>,
    ) -> Result<Option<DateTime<Utc>>> {
        let policy = self.get_assigned_policy(dataset_id)?;

        let expiration = match &policy.retention_period {
            RetentionPeriod::Duration { days } => {
                Some(baseline_date + Duration::days(*days))
            }
            RetentionPeriod::FromEvent { event: _, days } => {
                // In production, would fetch the event date
                Some(baseline_date + Duration::days(*days))
            }
            RetentionPeriod::EndOfPeriod {
                period,
                additional_periods,
            } => {
                let end_of_period = match period {
                    PeriodType::Year => {
                        let year = baseline_date.year();
                        DateTime::parse_from_rfc3339(&format!(
                            "{}-12-31T23:59:59Z",
                            year + (*additional_periods as i32)
                        ))
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                    }
                    PeriodType::Quarter => {
                        // Simplified quarter calculation
                        Some(baseline_date + Duration::days(90 * (*additional_periods + 1)))
                    }
                    PeriodType::Month => {
                        Some(baseline_date + Duration::days(30 * (*additional_periods + 1)))
                    }
                };
                end_of_period
            }
            RetentionPeriod::Indefinite => None,
            RetentionPeriod::Custom(_) => None,
        };

        Ok(expiration)
    }

    /// Check if data is expired
    pub fn is_expired(&self, dataset_id: &str, baseline_date: DateTime<Utc>) -> Result<bool> {
        let expiration = self.calculate_expiration(dataset_id, baseline_date)?;

        match expiration {
            Some(exp_date) => Ok(Utc::now() > exp_date),
            None => Ok(false),
        }
    }

    /// Get retention status for a dataset
    pub fn get_retention_status(
        &self,
        dataset_id: &str,
        baseline_date: DateTime<Utc>,
    ) -> Result<RetentionStatus> {
        let policy = self.get_assigned_policy(dataset_id)?;
        let expiration_date = self.calculate_expiration(dataset_id, baseline_date)?;

        let status = if let Some(exp_date) = expiration_date {
            let now = Utc::now();
            let days_until_expiration = (exp_date - now).num_days();

            if days_until_expiration < 0 {
                DataLifecycleStatus::Expired
            } else if days_until_expiration < 30 {
                DataLifecycleStatus::NearExpiration
            } else {
                DataLifecycleStatus::Active
            }
        } else {
            DataLifecycleStatus::Active
        };

        Ok(RetentionStatus {
            dataset_id: dataset_id.to_string(),
            policy_id: policy.id.clone(),
            baseline_date,
            expiration_date,
            status,
            last_reviewed: None,
            next_review: expiration_date.map(|exp| exp - Duration::days(30)),
            legal_hold: false,
            legal_hold_reason: None,
        })
    }

    /// List all policies
    pub fn list_policies(&self) -> Vec<&RetentionPolicy> {
        self.policies.values().collect()
    }

    /// List active policies
    pub fn list_active_policies(&self) -> Vec<&RetentionPolicy> {
        self.policies
            .values()
            .filter(|p| p.active)
            .collect()
    }

    /// Get datasets with a specific policy
    pub fn get_datasets_with_policy(&self, policy_id: &str) -> Vec<String> {
        self.assignments
            .iter()
            .filter(|(_, p)| *p == policy_id)
            .map(|(d, _)| d.clone())
            .collect()
    }

    /// Get retention statistics
    pub fn get_statistics(&self) -> RetentionStatistics {
        let total_policies = self.policies.len();
        let active_policies = self.list_active_policies().len();
        let total_assignments = self.assignments.len();

        let mut policies_by_action = HashMap::new();
        for policy in self.policies.values() {
            let action = format!("{:?}", policy.expiration_action);
            *policies_by_action.entry(action).or_insert(0) += 1;
        }

        RetentionStatistics {
            total_policies,
            active_policies,
            total_assignments,
            policies_by_action,
        }
    }
}

/// Retention statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionStatistics {
    pub total_policies: usize,
    pub active_policies: usize,
    pub total_assignments: usize,
    pub policies_by_action: HashMap<String, usize>,
}

impl Default for RetentionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_manager_creation() {
        let manager = RetentionManager::new();
        assert_eq!(manager.list_policies().len(), 0);
    }

    #[test]
    fn test_add_policy() {
        let mut manager = RetentionManager::new();
        let policy = RetentionPolicy {
            id: "7year_retention".to_string(),
            name: "7 Year Retention".to_string(),
            description: "Retain for 7 years".to_string(),
            retention_period: RetentionPeriod::Duration { days: 365 * 7 },
            expiration_action: ExpirationAction::Archive,
            legal_basis: Some("SOX".to_string()),
            owner: "compliance_team".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            active: true,
            exemptions: Vec::new(),
        };

        manager.add_policy(policy).unwrap();
        assert_eq!(manager.list_policies().len(), 1);
    }

    #[test]
    fn test_calculate_expiration() {
        let mut manager = RetentionManager::new();
        let policy = RetentionPolicy {
            id: "30day".to_string(),
            name: "30 Day Retention".to_string(),
            description: "Retain for 30 days".to_string(),
            retention_period: RetentionPeriod::Duration { days: 30 },
            expiration_action: ExpirationAction::Delete,
            legal_basis: None,
            owner: "data_team".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            active: true,
            exemptions: Vec::new(),
        };

        manager.add_policy(policy).unwrap();
        manager
            .assign_policy("test_dataset".to_string(), "30day".to_string())
            .unwrap();

        let baseline = Utc::now();
        let expiration = manager
            .calculate_expiration("test_dataset", baseline)
            .unwrap();

        assert!(expiration.is_some());
        let exp_date = expiration.unwrap();
        let days_diff = (exp_date - baseline).num_days();
        assert_eq!(days_diff, 30);
    }
}
