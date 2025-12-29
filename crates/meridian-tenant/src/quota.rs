//! Resource quotas and usage limits for multi-tenant systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};
use crate::tenant::TenantTier;

/// Resource types that can be limited by quotas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// API requests per time period
    ApiRequests,
    /// Database storage in bytes
    StorageBytes,
    /// Number of users
    Users,
    /// Number of concurrent connections
    Connections,
    /// Bandwidth in bytes
    BandwidthBytes,
    /// Number of GIS layers
    GisLayers,
    /// Number of map tiles
    MapTiles,
    /// Number of spatial queries
    SpatialQueries,
    /// Number of exports
    Exports,
    /// Custom resource type
    Custom(u32),
}

/// Quota definition for a resource type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    pub resource_type: ResourceType,
    pub limit: u64,
    pub period: Option<QuotaPeriod>,
    pub hard_limit: bool,
    pub warning_threshold: Option<u64>,
}

/// Time period for quota limits.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuotaPeriod {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl QuotaPeriod {
    pub fn to_seconds(&self) -> u64 {
        match self {
            QuotaPeriod::Second => 1,
            QuotaPeriod::Minute => 60,
            QuotaPeriod::Hour => 3600,
            QuotaPeriod::Day => 86400,
            QuotaPeriod::Week => 604800,
            QuotaPeriod::Month => 2592000, // 30 days
            QuotaPeriod::Year => 31536000,
        }
    }
}

/// Current usage of a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub tenant_id: Uuid,
    pub resource_type: ResourceType,
    pub current: u64,
    pub limit: u64,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl ResourceUsage {
    /// Calculates the percentage of quota used.
    pub fn usage_percentage(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        (self.current as f64 / self.limit as f64) * 100.0
    }

    /// Checks if usage exceeds the quota.
    pub fn is_exceeded(&self) -> bool {
        self.current >= self.limit
    }

    /// Checks if usage is approaching the limit.
    pub fn is_near_limit(&self, threshold_percentage: f64) -> bool {
        self.usage_percentage() >= threshold_percentage
    }
}

/// Quota configuration for a tenant tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierQuotas {
    pub tier: TenantTier,
    pub quotas: HashMap<ResourceType, Quota>,
}

impl TierQuotas {
    /// Creates default quotas for a tier.
    pub fn default_for_tier(tier: TenantTier) -> Self {
        let mut quotas = HashMap::new();

        match tier {
            TenantTier::Free => {
                quotas.insert(ResourceType::ApiRequests, Quota {
                    resource_type: ResourceType::ApiRequests,
                    limit: 1000,
                    period: Some(QuotaPeriod::Day),
                    hard_limit: true,
                    warning_threshold: Some(800),
                });
                quotas.insert(ResourceType::StorageBytes, Quota {
                    resource_type: ResourceType::StorageBytes,
                    limit: 1024 * 1024 * 100, // 100 MB
                    period: None,
                    hard_limit: true,
                    warning_threshold: Some(1024 * 1024 * 80),
                });
                quotas.insert(ResourceType::Users, Quota {
                    resource_type: ResourceType::Users,
                    limit: 1,
                    period: None,
                    hard_limit: true,
                    warning_threshold: None,
                });
                quotas.insert(ResourceType::GisLayers, Quota {
                    resource_type: ResourceType::GisLayers,
                    limit: 5,
                    period: None,
                    hard_limit: true,
                    warning_threshold: Some(4),
                });
            }
            TenantTier::Starter => {
                quotas.insert(ResourceType::ApiRequests, Quota {
                    resource_type: ResourceType::ApiRequests,
                    limit: 10000,
                    period: Some(QuotaPeriod::Day),
                    hard_limit: true,
                    warning_threshold: Some(8000),
                });
                quotas.insert(ResourceType::StorageBytes, Quota {
                    resource_type: ResourceType::StorageBytes,
                    limit: 1024 * 1024 * 1024, // 1 GB
                    period: None,
                    hard_limit: true,
                    warning_threshold: Some(1024 * 1024 * 800),
                });
                quotas.insert(ResourceType::Users, Quota {
                    resource_type: ResourceType::Users,
                    limit: 5,
                    period: None,
                    hard_limit: true,
                    warning_threshold: Some(4),
                });
                quotas.insert(ResourceType::GisLayers, Quota {
                    resource_type: ResourceType::GisLayers,
                    limit: 25,
                    period: None,
                    hard_limit: true,
                    warning_threshold: Some(20),
                });
            }
            TenantTier::Professional => {
                quotas.insert(ResourceType::ApiRequests, Quota {
                    resource_type: ResourceType::ApiRequests,
                    limit: 100000,
                    period: Some(QuotaPeriod::Day),
                    hard_limit: false,
                    warning_threshold: Some(80000),
                });
                quotas.insert(ResourceType::StorageBytes, Quota {
                    resource_type: ResourceType::StorageBytes,
                    limit: 1024 * 1024 * 1024 * 10, // 10 GB
                    period: None,
                    hard_limit: false,
                    warning_threshold: Some(1024 * 1024 * 1024 * 8),
                });
                quotas.insert(ResourceType::Users, Quota {
                    resource_type: ResourceType::Users,
                    limit: 25,
                    period: None,
                    hard_limit: false,
                    warning_threshold: Some(20),
                });
                quotas.insert(ResourceType::GisLayers, Quota {
                    resource_type: ResourceType::GisLayers,
                    limit: 100,
                    period: None,
                    hard_limit: false,
                    warning_threshold: Some(80),
                });
            }
            TenantTier::Enterprise | TenantTier::Custom => {
                quotas.insert(ResourceType::ApiRequests, Quota {
                    resource_type: ResourceType::ApiRequests,
                    limit: u64::MAX,
                    period: Some(QuotaPeriod::Day),
                    hard_limit: false,
                    warning_threshold: None,
                });
                quotas.insert(ResourceType::StorageBytes, Quota {
                    resource_type: ResourceType::StorageBytes,
                    limit: u64::MAX,
                    period: None,
                    hard_limit: false,
                    warning_threshold: None,
                });
                quotas.insert(ResourceType::Users, Quota {
                    resource_type: ResourceType::Users,
                    limit: u64::MAX,
                    period: None,
                    hard_limit: false,
                    warning_threshold: None,
                });
                quotas.insert(ResourceType::GisLayers, Quota {
                    resource_type: ResourceType::GisLayers,
                    limit: u64::MAX,
                    period: None,
                    hard_limit: false,
                    warning_threshold: None,
                });
            }
        }

        Self { tier, quotas }
    }

    /// Gets quota for a specific resource type.
    pub fn get_quota(&self, resource_type: &ResourceType) -> Option<&Quota> {
        self.quotas.get(resource_type)
    }
}

/// Quota manager for tracking and enforcing resource limits.
pub struct QuotaManager {
    tier_quotas: HashMap<TenantTier, TierQuotas>,
    usage_store: HashMap<(Uuid, ResourceType), ResourceUsage>,
}

impl QuotaManager {
    pub fn new() -> Self {
        let mut tier_quotas = HashMap::new();

        for tier in [
            TenantTier::Free,
            TenantTier::Starter,
            TenantTier::Professional,
            TenantTier::Enterprise,
        ] {
            tier_quotas.insert(tier, TierQuotas::default_for_tier(tier));
        }

        Self {
            tier_quotas,
            usage_store: HashMap::new(),
        }
    }

    /// Records resource usage for a tenant.
    pub fn record_usage(
        &mut self,
        tenant_id: Uuid,
        tier: TenantTier,
        resource_type: ResourceType,
        amount: u64,
    ) -> TenantResult<()> {
        let quota = self.tier_quotas
            .get(&tier)
            .and_then(|tq| tq.get_quota(&resource_type))
            .ok_or_else(|| TenantError::ConfigError(
                format!("No quota defined for resource type: {:?}", resource_type)
            ))?;

        let key = (tenant_id, resource_type);
        let usage = self.usage_store.entry(key).or_insert_with(|| {
            ResourceUsage {
                tenant_id,
                resource_type,
                current: 0,
                limit: quota.limit,
                period_start: quota.period.map(|_| Utc::now()),
                period_end: quota.period.map(|p| Utc::now() + chrono::Duration::seconds(p.to_seconds() as i64)),
                last_updated: Utc::now(),
            }
        });

        // Check if period has expired
        if let Some(period_end) = usage.period_end {
            if Utc::now() > period_end {
                // Reset usage for new period
                usage.current = 0;
                usage.period_start = Some(Utc::now());
                if let Some(period) = quota.period {
                    usage.period_end = Some(Utc::now() + chrono::Duration::seconds(period.to_seconds() as i64));
                }
            }
        }

        usage.current += amount;
        usage.last_updated = Utc::now();

        // Check if quota exceeded
        if quota.hard_limit && usage.is_exceeded() {
            return Err(TenantError::QuotaExceeded {
                resource: format!("{:?}", resource_type),
                current: usage.current,
                limit: usage.limit,
            });
        }

        Ok(())
    }

    /// Checks if a tenant can consume a certain amount of a resource.
    pub fn check_quota(
        &self,
        tenant_id: Uuid,
        tier: TenantTier,
        resource_type: ResourceType,
        amount: u64,
    ) -> TenantResult<bool> {
        let quota = self.tier_quotas
            .get(&tier)
            .and_then(|tq| tq.get_quota(&resource_type))
            .ok_or_else(|| TenantError::ConfigError(
                format!("No quota defined for resource type: {:?}", resource_type)
            ))?;

        let key = (tenant_id, resource_type);
        if let Some(usage) = self.usage_store.get(&key) {
            if quota.hard_limit && (usage.current + amount) > usage.limit {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Gets current usage for a tenant and resource type.
    pub fn get_usage(&self, tenant_id: Uuid, resource_type: ResourceType) -> Option<&ResourceUsage> {
        self.usage_store.get(&(tenant_id, resource_type))
    }

    /// Gets all usage for a tenant.
    pub fn get_tenant_usage(&self, tenant_id: Uuid) -> Vec<&ResourceUsage> {
        self.usage_store
            .iter()
            .filter(|((tid, _), _)| *tid == tenant_id)
            .map(|(_, usage)| usage)
            .collect()
    }

    /// Resets usage for a tenant and resource type.
    pub fn reset_usage(&mut self, tenant_id: Uuid, resource_type: ResourceType) {
        self.usage_store.remove(&(tenant_id, resource_type));
    }
}

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Quota violation event for monitoring and alerting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaViolation {
    pub tenant_id: Uuid,
    pub resource_type: ResourceType,
    pub current_usage: u64,
    pub quota_limit: u64,
    pub violation_time: DateTime<Utc>,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_quotas() {
        let quotas = TierQuotas::default_for_tier(TenantTier::Free);
        assert!(quotas.get_quota(&ResourceType::ApiRequests).is_some());
        assert_eq!(quotas.tier, TenantTier::Free);
    }

    #[test]
    fn test_quota_manager() {
        let mut manager = QuotaManager::new();
        let tenant_id = Uuid::new_v4();

        // Record usage
        let result = manager.record_usage(
            tenant_id,
            TenantTier::Free,
            ResourceType::ApiRequests,
            100,
        );
        assert!(result.is_ok());

        // Check usage
        let usage = manager.get_usage(tenant_id, ResourceType::ApiRequests);
        assert!(usage.is_some());
        assert_eq!(usage.unwrap().current, 100);
    }

    #[test]
    fn test_quota_exceeded() {
        let mut manager = QuotaManager::new();
        let tenant_id = Uuid::new_v4();

        // Try to exceed quota (Free tier has 1000 API requests/day)
        let result = manager.record_usage(
            tenant_id,
            TenantTier::Free,
            ResourceType::ApiRequests,
            1001,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_usage_percentage() {
        let usage = ResourceUsage {
            tenant_id: Uuid::new_v4(),
            resource_type: ResourceType::StorageBytes,
            current: 50,
            limit: 100,
            period_start: None,
            period_end: None,
            last_updated: Utc::now(),
        };

        assert_eq!(usage.usage_percentage(), 50.0);
        assert!(!usage.is_exceeded());
        assert!(usage.is_near_limit(40.0));
    }
}
