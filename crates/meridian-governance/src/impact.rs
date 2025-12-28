//! Impact analysis for schema changes and data modifications

use crate::error::{GovernanceError, Result};
use crate::lineage::{DataNode, LineageTracker};
use crate::schema::{CompatibilityIssue, SchemaRegistry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Impact analysis engine
#[derive(Debug, Clone)]
pub struct ImpactAnalyzer {
    /// Reference to lineage tracker
    lineage: LineageTracker,
    /// Reference to schema registry
    schema_registry: SchemaRegistry,
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysisResult {
    /// Entity being analyzed
    pub target_entity: String,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
    /// Overall impact score (0.0 = no impact, 1.0 = critical impact)
    pub impact_score: f64,
    /// Impact level
    pub impact_level: ImpactLevel,
    /// Affected upstream entities
    pub upstream_impact: Vec<ImpactedEntity>,
    /// Affected downstream entities
    pub downstream_impact: Vec<ImpactedEntity>,
    /// Total affected entities
    pub total_affected: usize,
    /// Recommended actions
    pub recommendations: Vec<String>,
    /// Risks identified
    pub risks: Vec<Risk>,
}

/// Impact level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Impacted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactedEntity {
    /// Entity identifier
    pub entity_id: String,
    /// Entity type
    pub entity_type: String,
    /// Impact type
    pub impact_type: ImpactType,
    /// Impact severity
    pub severity: ImpactLevel,
    /// Distance from source (number of hops)
    pub distance: usize,
    /// Estimated affected records
    pub affected_records: Option<u64>,
}

/// Type of impact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImpactType {
    /// Schema change impact
    SchemaChange,
    /// Data modification impact
    DataModification,
    /// Access control change
    AccessControl,
    /// Deletion impact
    Deletion,
    /// Performance impact
    Performance,
    /// Compliance impact
    Compliance,
    /// Custom impact type
    Custom(String),
}

/// Risk identified in impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    /// Risk identifier
    pub id: String,
    /// Risk category
    pub category: RiskCategory,
    /// Risk severity
    pub severity: ImpactLevel,
    /// Risk description
    pub description: String,
    /// Mitigation recommendations
    pub mitigation: Vec<String>,
}

/// Risk category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskCategory {
    /// Data loss risk
    DataLoss,
    /// Breaking change risk
    BreakingChange,
    /// Performance degradation
    Performance,
    /// Compliance violation
    Compliance,
    /// Security risk
    Security,
    /// Availability risk
    Availability,
}

/// Schema change impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChangeImpact {
    /// Subject name
    pub subject: String,
    /// Old version
    pub old_version: u32,
    /// New version
    pub new_version: u32,
    /// Compatibility issues
    pub compatibility_issues: Vec<CompatibilityIssue>,
    /// Affected downstream entities
    pub affected_entities: Vec<String>,
    /// Breaking changes detected
    pub breaking_changes: bool,
    /// Recommended migration steps
    pub migration_steps: Vec<String>,
}

impl ImpactAnalyzer {
    /// Create a new impact analyzer
    pub fn new(lineage: LineageTracker, schema_registry: SchemaRegistry) -> Self {
        Self {
            lineage,
            schema_registry,
        }
    }

    /// Analyze impact of changing/deleting an entity
    pub fn analyze_entity_change(&self, entity_id: &str) -> Result<ImpactAnalysisResult> {
        // Get upstream and downstream dependencies
        let upstream = self
            .lineage
            .get_upstream(entity_id)
            .unwrap_or_else(|_| Vec::new());
        let downstream = self
            .lineage
            .get_downstream(entity_id)
            .unwrap_or_else(|_| Vec::new());

        // Calculate impact for each affected entity
        let upstream_impact = self.calculate_entity_impacts(&upstream, ImpactType::DataModification);
        let downstream_impact =
            self.calculate_entity_impacts(&downstream, ImpactType::DataModification);

        let total_affected = upstream_impact.len() + downstream_impact.len();

        // Calculate overall impact score
        let impact_score = self.calculate_impact_score(&upstream_impact, &downstream_impact);
        let impact_level = Self::score_to_level(impact_score);

        // Identify risks
        let risks = self.identify_risks(entity_id, &upstream_impact, &downstream_impact);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&impact_level, &risks);

        Ok(ImpactAnalysisResult {
            target_entity: entity_id.to_string(),
            analyzed_at: Utc::now(),
            impact_score,
            impact_level,
            upstream_impact,
            downstream_impact,
            total_affected,
            recommendations,
            risks,
        })
    }

    /// Analyze impact of a schema change
    pub fn analyze_schema_change(
        &self,
        subject: &str,
        old_version: u32,
        new_version: u32,
    ) -> Result<SchemaChangeImpact> {
        let old_schema = self.schema_registry.get_version(subject, old_version)?;
        let new_schema = self.schema_registry.get_version(subject, new_version)?;

        let compatibility = self
            .schema_registry
            .check_compatibility(&old_schema.schema, &new_schema.schema)?;

        let breaking_changes = compatibility
            .issues
            .iter()
            .any(|i| i.severity == crate::schema::IssueSeverity::Breaking);

        // Get affected downstream entities
        let affected_entities = self
            .lineage
            .get_downstream(subject)
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|node| node.entity_id)
            .collect();

        let migration_steps = if breaking_changes {
            self.generate_migration_steps(&compatibility.issues)
        } else {
            vec!["No breaking changes detected. Safe to deploy.".to_string()]
        };

        Ok(SchemaChangeImpact {
            subject: subject.to_string(),
            old_version,
            new_version,
            compatibility_issues: compatibility.issues,
            affected_entities,
            breaking_changes,
            migration_steps,
        })
    }

    /// Calculate impact for multiple entities
    fn calculate_entity_impacts(
        &self,
        entities: &[DataNode],
        impact_type: ImpactType,
    ) -> Vec<ImpactedEntity> {
        entities
            .iter()
            .enumerate()
            .map(|(index, node)| ImpactedEntity {
                entity_id: node.entity_id.clone(),
                entity_type: format!("{:?}", node.entity_type),
                impact_type: impact_type.clone(),
                severity: if index < 5 {
                    ImpactLevel::High
                } else if index < 15 {
                    ImpactLevel::Medium
                } else {
                    ImpactLevel::Low
                },
                distance: index + 1,
                affected_records: None,
            })
            .collect()
    }

    /// Calculate overall impact score
    fn calculate_impact_score(
        &self,
        upstream: &[ImpactedEntity],
        downstream: &[ImpactedEntity],
    ) -> f64 {
        let total_entities = (upstream.len() + downstream.len()) as f64;

        if total_entities == 0.0 {
            return 0.0;
        }

        // Weight downstream more heavily than upstream
        let downstream_weight = 0.7;
        let upstream_weight = 0.3;

        let downstream_score = downstream.len() as f64 * downstream_weight;
        let upstream_score = upstream.len() as f64 * upstream_weight;

        // Normalize to 0-1 range (assume max 50 entities = score of 1.0)
        ((downstream_score + upstream_score) / 50.0).min(1.0)
    }

    /// Convert impact score to impact level
    fn score_to_level(score: f64) -> ImpactLevel {
        if score >= 0.8 {
            ImpactLevel::Critical
        } else if score >= 0.6 {
            ImpactLevel::High
        } else if score >= 0.3 {
            ImpactLevel::Medium
        } else if score > 0.0 {
            ImpactLevel::Low
        } else {
            ImpactLevel::None
        }
    }

    /// Identify risks based on impact analysis
    fn identify_risks(
        &self,
        _entity_id: &str,
        upstream: &[ImpactedEntity],
        downstream: &[ImpactedEntity],
    ) -> Vec<Risk> {
        let mut risks = Vec::new();

        // Check for high downstream impact
        if downstream.len() > 10 {
            risks.push(Risk {
                id: "high_downstream_impact".to_string(),
                category: RiskCategory::BreakingChange,
                severity: ImpactLevel::High,
                description: format!(
                    "{} downstream entities will be affected by this change",
                    downstream.len()
                ),
                mitigation: vec![
                    "Notify all downstream consumers".to_string(),
                    "Plan a gradual rollout".to_string(),
                    "Implement backward compatibility".to_string(),
                ],
            });
        }

        // Check for critical upstream dependencies
        if upstream.len() > 5 {
            risks.push(Risk {
                id: "upstream_dependencies".to_string(),
                category: RiskCategory::Availability,
                severity: ImpactLevel::Medium,
                description: format!(
                    "Entity depends on {} upstream sources",
                    upstream.len()
                ),
                mitigation: vec![
                    "Ensure upstream sources are stable".to_string(),
                    "Implement error handling for upstream failures".to_string(),
                ],
            });
        }

        risks
    }

    /// Generate recommendations based on impact level
    fn generate_recommendations(&self, level: &ImpactLevel, risks: &[Risk]) -> Vec<String> {
        let mut recommendations = Vec::new();

        match level {
            ImpactLevel::Critical => {
                recommendations.push(
                    "CRITICAL: Extensive testing required before deployment".to_string(),
                );
                recommendations.push("Notify all stakeholders immediately".to_string());
                recommendations.push("Plan for rollback procedures".to_string());
                recommendations.push("Consider phased rollout strategy".to_string());
            }
            ImpactLevel::High => {
                recommendations.push("HIGH: Comprehensive testing recommended".to_string());
                recommendations.push("Notify affected teams".to_string());
                recommendations.push("Monitor closely after deployment".to_string());
            }
            ImpactLevel::Medium => {
                recommendations.push("MEDIUM: Standard testing procedures apply".to_string());
                recommendations.push("Review with team lead".to_string());
            }
            ImpactLevel::Low => {
                recommendations.push("LOW: Minimal impact expected".to_string());
                recommendations.push("Standard deployment process".to_string());
            }
            ImpactLevel::None => {
                recommendations.push("No impact detected. Safe to proceed.".to_string());
            }
        }

        // Add risk-specific recommendations
        for risk in risks {
            for mitigation in &risk.mitigation {
                if !recommendations.contains(mitigation) {
                    recommendations.push(mitigation.clone());
                }
            }
        }

        recommendations
    }

    /// Generate migration steps for schema changes
    fn generate_migration_steps(&self, issues: &[CompatibilityIssue]) -> Vec<String> {
        let mut steps = vec!["1. Review all compatibility issues".to_string()];

        let has_breaking = issues
            .iter()
            .any(|i| i.severity == crate::schema::IssueSeverity::Breaking);

        if has_breaking {
            steps.push("2. Create migration plan for breaking changes".to_string());
            steps.push("3. Update all downstream consumers".to_string());
            steps.push("4. Test with sample data".to_string());
            steps.push("5. Deploy to staging environment".to_string());
            steps.push("6. Validate all integrations".to_string());
            steps.push("7. Schedule production deployment window".to_string());
            steps.push("8. Deploy with monitoring enabled".to_string());
            steps.push("9. Verify all downstream systems".to_string());
        } else {
            steps.push("2. Test changes in staging".to_string());
            steps.push("3. Deploy to production".to_string());
            steps.push("4. Monitor for issues".to_string());
        }

        steps
    }

    /// Get lineage tracker reference
    pub fn lineage(&self) -> &LineageTracker {
        &self.lineage
    }

    /// Get schema registry reference
    pub fn schema_registry(&self) -> &SchemaRegistry {
        &self.schema_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_level_ordering() {
        assert!(ImpactLevel::Critical > ImpactLevel::High);
        assert!(ImpactLevel::High > ImpactLevel::Medium);
        assert!(ImpactLevel::Medium > ImpactLevel::Low);
        assert!(ImpactLevel::Low > ImpactLevel::None);
    }

    #[test]
    fn test_score_to_level() {
        assert_eq!(ImpactAnalyzer::score_to_level(0.0), ImpactLevel::None);
        assert_eq!(ImpactAnalyzer::score_to_level(0.2), ImpactLevel::Low);
        assert_eq!(ImpactAnalyzer::score_to_level(0.5), ImpactLevel::Medium);
        assert_eq!(ImpactAnalyzer::score_to_level(0.7), ImpactLevel::High);
        assert_eq!(ImpactAnalyzer::score_to_level(0.9), ImpactLevel::Critical);
    }
}
