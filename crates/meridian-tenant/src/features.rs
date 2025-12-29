//! Tenant feature flags and capability management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};
use crate::tenant::{Tenant, TenantTier};

/// Feature flag definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled_by_default: bool,
    pub tier_requirements: Option<HashSet<TenantTier>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FeatureFlag {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            description: None,
            enabled_by_default: false,
            tier_requirements: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn enabled_by_default(mut self) -> Self {
        self.enabled_by_default = true;
        self
    }

    pub fn require_tier(mut self, tier: TenantTier) -> Self {
        self.tier_requirements
            .get_or_insert_with(HashSet::new)
            .insert(tier);
        self
    }

    pub fn is_available_for_tier(&self, tier: TenantTier) -> bool {
        if let Some(required_tiers) = &self.tier_requirements {
            required_tiers.contains(&tier)
        } else {
            true
        }
    }
}

/// Tenant-specific feature override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantFeature {
    pub tenant_id: Uuid,
    pub feature_key: String,
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
    pub enabled_at: Option<DateTime<Utc>>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Feature rollout strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RolloutStrategy {
    /// Feature is available to all tenants
    All,
    /// Feature is available to percentage of tenants
    Percentage,
    /// Feature is available to specific tenants only
    Whitelist,
    /// Feature is available to all except blacklisted tenants
    Blacklist,
    /// Gradual rollout based on tenant creation date
    Gradual,
}

/// Feature rollout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRollout {
    pub feature_key: String,
    pub strategy: RolloutStrategy,
    pub percentage: Option<f64>,
    pub whitelist: HashSet<Uuid>,
    pub blacklist: HashSet<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl FeatureRollout {
    pub fn new(feature_key: impl Into<String>) -> Self {
        Self {
            feature_key: feature_key.into(),
            strategy: RolloutStrategy::All,
            percentage: None,
            whitelist: HashSet::new(),
            blacklist: HashSet::new(),
            start_date: None,
            end_date: None,
        }
    }

    pub fn percentage_rollout(mut self, percentage: f64) -> Self {
        self.strategy = RolloutStrategy::Percentage;
        self.percentage = Some(percentage.clamp(0.0, 100.0));
        self
    }

    pub fn whitelist_tenant(mut self, tenant_id: Uuid) -> Self {
        self.strategy = RolloutStrategy::Whitelist;
        self.whitelist.insert(tenant_id);
        self
    }

    pub fn is_enabled_for_tenant(&self, tenant_id: Uuid) -> bool {
        match self.strategy {
            RolloutStrategy::All => true,
            RolloutStrategy::Percentage => {
                if let Some(percentage) = self.percentage {
                    // Use tenant_id hash for deterministic percentage
                    let hash = self.hash_tenant_id(tenant_id);
                    let tenant_percentage = (hash % 100) as f64;
                    tenant_percentage < percentage
                } else {
                    false
                }
            }
            RolloutStrategy::Whitelist => self.whitelist.contains(&tenant_id),
            RolloutStrategy::Blacklist => !self.blacklist.contains(&tenant_id),
            RolloutStrategy::Gradual => {
                // Implement gradual rollout logic
                true
            }
        }
    }

    fn hash_tenant_id(&self, tenant_id: Uuid) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        tenant_id.hash(&mut hasher);
        hasher.finish()
    }
}

/// Feature manager for handling tenant feature flags.
pub struct FeatureManager {
    features: HashMap<String, FeatureFlag>,
    tenant_features: HashMap<Uuid, HashMap<String, TenantFeature>>,
    rollouts: HashMap<String, FeatureRollout>,
}

impl FeatureManager {
    pub fn new() -> Self {
        let mut manager = Self {
            features: HashMap::new(),
            tenant_features: HashMap::new(),
            rollouts: HashMap::new(),
        };

        // Register default features
        manager.register_default_features();

        manager
    }

    fn register_default_features(&mut self) {
        // GIS Features
        self.register_feature(
            FeatureFlag::new("advanced_mapping", "Advanced Mapping")
                .with_description("Advanced GIS mapping capabilities")
                .require_tier(TenantTier::Professional)
                .require_tier(TenantTier::Enterprise),
        );

        self.register_feature(
            FeatureFlag::new("3d_visualization", "3D Visualization")
                .with_description("3D map visualization")
                .require_tier(TenantTier::Enterprise),
        );

        self.register_feature(
            FeatureFlag::new("custom_branding", "Custom Branding")
                .with_description("White-label branding capabilities")
                .require_tier(TenantTier::Professional)
                .require_tier(TenantTier::Enterprise),
        );

        self.register_feature(
            FeatureFlag::new("api_access", "API Access")
                .with_description("Programmatic API access")
                .require_tier(TenantTier::Professional)
                .require_tier(TenantTier::Enterprise),
        );

        self.register_feature(
            FeatureFlag::new("real_time_collaboration", "Real-time Collaboration")
                .with_description("Real-time collaborative editing")
                .require_tier(TenantTier::Enterprise),
        );

        self.register_feature(
            FeatureFlag::new("export_formats", "Advanced Export Formats")
                .with_description("Export to KML, GeoJSON, Shapefile, etc.")
                .enabled_by_default(),
        );
    }

    /// Registers a new feature flag.
    pub fn register_feature(&mut self, feature: FeatureFlag) {
        self.features.insert(feature.key.clone(), feature);
    }

    /// Checks if a feature is enabled for a tenant.
    pub fn is_enabled(&self, tenant: &Tenant, feature_key: &str) -> bool {
        // Check tenant-specific override first
        if let Some(tenant_features) = self.tenant_features.get(&tenant.id) {
            if let Some(tenant_feature) = tenant_features.get(feature_key) {
                // Check if override is expired
                if let Some(expires_at) = tenant_feature.expires_at {
                    if Utc::now() > expires_at {
                        return false;
                    }
                }
                return tenant_feature.enabled;
            }
        }

        // Check rollout strategy
        if let Some(rollout) = self.rollouts.get(feature_key) {
            if !rollout.is_enabled_for_tenant(tenant.id) {
                return false;
            }
        }

        // Check feature definition
        if let Some(feature) = self.features.get(feature_key) {
            if !feature.is_available_for_tier(tenant.tier) {
                return false;
            }
            return feature.enabled_by_default;
        }

        false
    }

    /// Enables a feature for a specific tenant.
    pub fn enable_feature(
        &mut self,
        tenant_id: Uuid,
        feature_key: impl Into<String>,
    ) -> TenantResult<()> {
        let feature_key = feature_key.into();

        if !self.features.contains_key(&feature_key) {
            return Err(TenantError::FeatureNotAvailable(feature_key));
        }

        let tenant_feature = TenantFeature {
            tenant_id,
            feature_key: feature_key.clone(),
            enabled: true,
            config: None,
            enabled_at: Some(Utc::now()),
            disabled_at: None,
            expires_at: None,
        };

        self.tenant_features
            .entry(tenant_id)
            .or_insert_with(HashMap::new)
            .insert(feature_key, tenant_feature);

        Ok(())
    }

    /// Disables a feature for a specific tenant.
    pub fn disable_feature(
        &mut self,
        tenant_id: Uuid,
        feature_key: impl Into<String>,
    ) -> TenantResult<()> {
        let feature_key = feature_key.into();

        let tenant_feature = TenantFeature {
            tenant_id,
            feature_key: feature_key.clone(),
            enabled: false,
            config: None,
            enabled_at: None,
            disabled_at: Some(Utc::now()),
            expires_at: None,
        };

        self.tenant_features
            .entry(tenant_id)
            .or_insert_with(HashMap::new)
            .insert(feature_key, tenant_feature);

        Ok(())
    }

    /// Gets all enabled features for a tenant.
    pub fn get_enabled_features(&self, tenant: &Tenant) -> Vec<String> {
        self.features
            .keys()
            .filter(|key| self.is_enabled(tenant, key))
            .cloned()
            .collect()
    }

    /// Gets all available features for a tenant tier.
    pub fn get_tier_features(&self, tier: TenantTier) -> Vec<&FeatureFlag> {
        self.features
            .values()
            .filter(|feature| feature.is_available_for_tier(tier))
            .collect()
    }

    /// Sets a feature rollout strategy.
    pub fn set_rollout(&mut self, rollout: FeatureRollout) {
        self.rollouts.insert(rollout.feature_key.clone(), rollout);
    }

    /// Gets feature configuration for a tenant.
    pub fn get_feature_config(&self, tenant_id: Uuid, feature_key: &str) -> Option<&serde_json::Value> {
        self.tenant_features
            .get(&tenant_id)
            .and_then(|features| features.get(feature_key))
            .and_then(|feature| feature.config.as_ref())
    }

    /// Sets feature configuration for a tenant.
    pub fn set_feature_config(
        &mut self,
        tenant_id: Uuid,
        feature_key: impl Into<String>,
        config: serde_json::Value,
    ) -> TenantResult<()> {
        let feature_key = feature_key.into();

        let tenant_features = self
            .tenant_features
            .entry(tenant_id)
            .or_insert_with(HashMap::new);

        if let Some(feature) = tenant_features.get_mut(&feature_key) {
            feature.config = Some(config);
        } else {
            let tenant_feature = TenantFeature {
                tenant_id,
                feature_key: feature_key.clone(),
                enabled: true,
                config: Some(config),
                enabled_at: Some(Utc::now()),
                disabled_at: None,
                expires_at: None,
            };
            tenant_features.insert(feature_key, tenant_feature);
        }

        Ok(())
    }
}

impl Default for FeatureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag() {
        let feature = FeatureFlag::new("test_feature", "Test Feature")
            .with_description("A test feature")
            .require_tier(TenantTier::Professional);

        assert!(!feature.is_available_for_tier(TenantTier::Free));
        assert!(feature.is_available_for_tier(TenantTier::Professional));
    }

    #[test]
    fn test_feature_manager() {
        let mut manager = FeatureManager::new();
        let tenant = crate::tenant::Tenant::new("test", "Test", "test@example.com");

        // Feature not available for free tier
        assert!(!manager.is_enabled(&tenant, "advanced_mapping"));

        // Enable feature override
        manager.enable_feature(tenant.id, "advanced_mapping").unwrap();
        assert!(manager.is_enabled(&tenant, "advanced_mapping"));
    }

    #[test]
    fn test_rollout_percentage() {
        let rollout = FeatureRollout::new("test_feature").percentage_rollout(50.0);

        let tenant_id = Uuid::new_v4();
        let enabled = rollout.is_enabled_for_tenant(tenant_id);

        // Should be deterministic based on tenant ID
        assert_eq!(enabled, rollout.is_enabled_for_tenant(tenant_id));
    }

    #[test]
    fn test_rollout_whitelist() {
        let tenant_id = Uuid::new_v4();
        let rollout = FeatureRollout::new("test_feature").whitelist_tenant(tenant_id);

        assert!(rollout.is_enabled_for_tenant(tenant_id));
        assert!(!rollout.is_enabled_for_tenant(Uuid::new_v4()));
    }
}
