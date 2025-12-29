//! Recovery Time Objective (RTO) and Recovery Point Objective (RPO) monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{BackupError, Result};

/// RTO/RPO metric type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Rto,
    Rpo,
}

/// SLA (Service Level Agreement) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaConfig {
    pub id: Uuid,
    pub name: String,
    pub rto_minutes: u64,
    pub rpo_minutes: u64,
    pub enabled: bool,
}

/// Recovery measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMeasurement {
    pub id: Uuid,
    pub backup_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub actual_rto: Option<chrono::Duration>,
    pub actual_rpo: Option<chrono::Duration>,
    pub target_rto: chrono::Duration,
    pub target_rpo: chrono::Duration,
    pub sla_met: bool,
}

/// RTO/RPO monitoring manager.
pub struct RtoMonitor {
    sla_configs: HashMap<Uuid, SlaConfig>,
    measurements: Vec<RecoveryMeasurement>,
}

impl RtoMonitor {
    /// Create a new RTO monitor.
    pub fn new() -> Self {
        Self {
            sla_configs: HashMap::new(),
            measurements: Vec::new(),
        }
    }

    /// Add an SLA configuration.
    pub fn add_sla(&mut self, sla: SlaConfig) {
        self.sla_configs.insert(sla.id, sla);
    }

    /// Remove an SLA configuration.
    pub fn remove_sla(&mut self, sla_id: Uuid) -> Result<()> {
        self.sla_configs
            .remove(&sla_id)
            .ok_or_else(|| BackupError::BackupNotFound(sla_id.to_string()))?;
        Ok(())
    }

    /// Start a recovery measurement.
    pub fn start_recovery_measurement(
        &mut self,
        backup_id: Uuid,
        sla_id: Uuid,
    ) -> Result<Uuid> {
        let sla = self
            .sla_configs
            .get(&sla_id)
            .ok_or_else(|| BackupError::BackupNotFound(sla_id.to_string()))?;

        let measurement_id = Uuid::new_v4();
        let measurement = RecoveryMeasurement {
            id: measurement_id,
            backup_id,
            started_at: chrono::Utc::now(),
            completed_at: None,
            actual_rto: None,
            actual_rpo: None,
            target_rto: chrono::Duration::minutes(sla.rto_minutes as i64),
            target_rpo: chrono::Duration::minutes(sla.rpo_minutes as i64),
            sla_met: false,
        };

        self.measurements.push(measurement);

        Ok(measurement_id)
    }

    /// Complete a recovery measurement.
    pub fn complete_recovery_measurement(
        &mut self,
        measurement_id: Uuid,
        backup_timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let measurement = self
            .measurements
            .iter_mut()
            .find(|m| m.id == measurement_id)
            .ok_or_else(|| BackupError::BackupNotFound(measurement_id.to_string()))?;

        let completed_at = chrono::Utc::now();
        let actual_rto = completed_at - measurement.started_at;
        let actual_rpo = measurement.started_at - backup_timestamp;

        measurement.completed_at = Some(completed_at);
        measurement.actual_rto = Some(actual_rto);
        measurement.actual_rpo = Some(actual_rpo);

        // Check if SLA was met
        measurement.sla_met =
            actual_rto <= measurement.target_rto && actual_rpo <= measurement.target_rpo;

        // Log SLA violations
        if !measurement.sla_met {
            tracing::warn!(
                "SLA violation detected for measurement {}: RTO={:?} (target={:?}), RPO={:?} (target={:?})",
                measurement_id,
                actual_rto,
                measurement.target_rto,
                actual_rpo,
                measurement.target_rpo
            );
        }

        Ok(())
    }

    /// Calculate current RPO based on last backup time.
    pub fn calculate_current_rpo(
        &self,
        last_backup_time: chrono::DateTime<chrono::Utc>,
    ) -> chrono::Duration {
        chrono::Utc::now() - last_backup_time
    }

    /// Check if current RPO meets SLA.
    pub fn check_rpo_sla(
        &self,
        last_backup_time: chrono::DateTime<chrono::Utc>,
        sla_id: Uuid,
    ) -> Result<bool> {
        let sla = self
            .sla_configs
            .get(&sla_id)
            .ok_or_else(|| BackupError::BackupNotFound(sla_id.to_string()))?;

        let current_rpo = self.calculate_current_rpo(last_backup_time);
        let target_rpo = chrono::Duration::minutes(sla.rpo_minutes as i64);

        Ok(current_rpo <= target_rpo)
    }

    /// Get SLA compliance rate.
    pub fn get_sla_compliance_rate(&self) -> f64 {
        if self.measurements.is_empty() {
            return 100.0;
        }

        let compliant = self.measurements.iter().filter(|m| m.sla_met).count();
        (compliant as f64 / self.measurements.len() as f64) * 100.0
    }

    /// Get average RTO.
    pub fn get_average_rto(&self) -> Option<chrono::Duration> {
        let completed: Vec<_> = self
            .measurements
            .iter()
            .filter_map(|m| m.actual_rto)
            .collect();

        if completed.is_empty() {
            return None;
        }

        let total_secs: i64 = completed.iter().map(|d| d.num_seconds()).sum();
        Some(chrono::Duration::seconds(total_secs / completed.len() as i64))
    }

    /// Get average RPO.
    pub fn get_average_rpo(&self) -> Option<chrono::Duration> {
        let completed: Vec<_> = self
            .measurements
            .iter()
            .filter_map(|m| m.actual_rpo)
            .collect();

        if completed.is_empty() {
            return None;
        }

        let total_secs: i64 = completed.iter().map(|d| d.num_seconds()).sum();
        Some(chrono::Duration::seconds(total_secs / completed.len() as i64))
    }

    /// Get measurements for a specific backup.
    pub fn get_measurements_for_backup(&self, backup_id: Uuid) -> Vec<&RecoveryMeasurement> {
        self.measurements
            .iter()
            .filter(|m| m.backup_id == backup_id)
            .collect()
    }

    /// Get recent SLA violations.
    pub fn get_recent_violations(&self, limit: usize) -> Vec<&RecoveryMeasurement> {
        self.measurements
            .iter()
            .filter(|m| !m.sla_met && m.completed_at.is_some())
            .rev()
            .take(limit)
            .collect()
    }

    /// Get statistics.
    pub fn statistics(&self) -> RtoStatistics {
        let total_measurements = self.measurements.len();
        let completed_measurements = self
            .measurements
            .iter()
            .filter(|m| m.completed_at.is_some())
            .count();
        let sla_violations = self.measurements.iter().filter(|m| !m.sla_met).count();

        RtoStatistics {
            total_measurements,
            completed_measurements,
            sla_violations,
            compliance_rate: self.get_sla_compliance_rate(),
            average_rto: self.get_average_rto(),
            average_rpo: self.get_average_rpo(),
            active_slas: self.sla_configs.values().filter(|s| s.enabled).count(),
        }
    }

    /// Generate SLA report.
    pub fn generate_sla_report(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> SlaReport {
        let relevant_measurements: Vec<_> = self
            .measurements
            .iter()
            .filter(|m| {
                m.started_at >= start_time
                    && m.completed_at.map_or(false, |ct| ct <= end_time)
            })
            .collect();

        let total = relevant_measurements.len();
        let met = relevant_measurements.iter().filter(|m| m.sla_met).count();
        let violated = total - met;

        let avg_rto = if !relevant_measurements.is_empty() {
            let total_secs: i64 = relevant_measurements
                .iter()
                .filter_map(|m| m.actual_rto)
                .map(|d| d.num_seconds())
                .sum();
            Some(chrono::Duration::seconds(
                total_secs / relevant_measurements.len() as i64,
            ))
        } else {
            None
        };

        let avg_rpo = if !relevant_measurements.is_empty() {
            let total_secs: i64 = relevant_measurements
                .iter()
                .filter_map(|m| m.actual_rpo)
                .map(|d| d.num_seconds())
                .sum();
            Some(chrono::Duration::seconds(
                total_secs / relevant_measurements.len() as i64,
            ))
        } else {
            None
        };

        SlaReport {
            start_time,
            end_time,
            total_measurements: total,
            sla_met: met,
            sla_violated: violated,
            compliance_rate: if total > 0 {
                (met as f64 / total as f64) * 100.0
            } else {
                100.0
            },
            average_rto: avg_rto,
            average_rpo: avg_rpo,
        }
    }
}

impl Default for RtoMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// RTO/RPO statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtoStatistics {
    pub total_measurements: usize,
    pub completed_measurements: usize,
    pub sla_violations: usize,
    pub compliance_rate: f64,
    pub average_rto: Option<chrono::Duration>,
    pub average_rpo: Option<chrono::Duration>,
    pub active_slas: usize,
}

/// SLA report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaReport {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub total_measurements: usize,
    pub sla_met: usize,
    pub sla_violated: usize,
    pub compliance_rate: f64,
    pub average_rto: Option<chrono::Duration>,
    pub average_rpo: Option<chrono::Duration>,
}
