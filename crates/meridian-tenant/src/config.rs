//! Per-tenant configuration management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};

/// Configuration scope for tenant settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    /// System-wide defaults
    System,
    /// Tenant-specific configuration
    Tenant,
    /// User-specific configuration (within tenant)
    User,
}

/// Configuration value with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    pub key: String,
    pub value: serde_json::Value,
    pub value_type: ConfigValueType,
    pub scope: ConfigScope,
    pub is_secret: bool,
    pub is_locked: bool,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ConfigValidation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigValueType {
    String,
    Number,
    Boolean,
    Json,
    Array,
}

/// Validation rules for configuration values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

/// Tenant-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub tenant_id: Uuid,
    pub settings: HashMap<String, ConfigValue>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TenantConfig {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            settings: HashMap::new(),
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Gets a configuration value.
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.settings.get(key)
    }

    /// Gets a typed configuration value.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.value.as_str().map(String::from))
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.value.as_bool())
    }

    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.value.as_f64())
    }

    pub fn get_json(&self, key: &str) -> Option<&serde_json::Value> {
        self.get(key).map(|v| &v.value)
    }

    /// Sets a configuration value.
    pub fn set(&mut self, key: impl Into<String>, value: ConfigValue) -> TenantResult<()> {
        let key = key.into();

        // Check if locked
        if let Some(existing) = self.settings.get(&key) {
            if existing.is_locked {
                return Err(TenantError::ConfigError(
                    format!("Configuration key '{}' is locked", key)
                ));
            }
        }

        // Validate value
        if let Some(validation) = &value.validation {
            self.validate_value(&value.value, validation)?;
        }

        self.settings.insert(key, value);
        self.updated_at = Utc::now();
        self.version += 1;

        Ok(())
    }

    /// Removes a configuration value.
    pub fn remove(&mut self, key: &str) -> TenantResult<()> {
        if let Some(existing) = self.settings.get(key) {
            if existing.is_locked {
                return Err(TenantError::ConfigError(
                    format!("Configuration key '{}' is locked", key)
                ));
            }
        }

        self.settings.remove(key);
        self.updated_at = Utc::now();
        self.version += 1;

        Ok(())
    }

    fn validate_value(&self, value: &serde_json::Value, validation: &ConfigValidation) -> TenantResult<()> {
        // Validate string length
        if let Some(s) = value.as_str() {
            if let Some(min) = validation.min_length {
                if s.len() < min {
                    return Err(TenantError::ValidationError(
                        format!("Value length must be at least {}", min)
                    ));
                }
            }
            if let Some(max) = validation.max_length {
                if s.len() > max {
                    return Err(TenantError::ValidationError(
                        format!("Value length must be at most {}", max)
                    ));
                }
            }
        }

        // Validate numeric range
        if let Some(n) = value.as_f64() {
            if let Some(min) = validation.min_value {
                if n < min {
                    return Err(TenantError::ValidationError(
                        format!("Value must be at least {}", min)
                    ));
                }
            }
            if let Some(max) = validation.max_value {
                if n > max {
                    return Err(TenantError::ValidationError(
                        format!("Value must be at most {}", max)
                    ));
                }
            }
        }

        // Validate allowed values
        if let Some(allowed) = &validation.allowed_values {
            if !allowed.contains(value) {
                return Err(TenantError::ValidationError(
                    "Value is not in the list of allowed values".to_string()
                ));
            }
        }

        Ok(())
    }
}

/// Configuration manager for all tenants.
pub struct ConfigManager {
    configs: HashMap<Uuid, TenantConfig>,
    default_config: HashMap<String, ConfigValue>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let mut default_config = HashMap::new();

        // Add default system configurations
        default_config.insert(
            "max_map_layers".to_string(),
            ConfigValue {
                key: "max_map_layers".to_string(),
                value: serde_json::json!(10),
                value_type: ConfigValueType::Number,
                scope: ConfigScope::System,
                is_secret: false,
                is_locked: false,
                description: Some("Maximum number of map layers".to_string()),
                default_value: Some(serde_json::json!(10)),
                validation: Some(ConfigValidation {
                    min_length: None,
                    max_length: None,
                    min_value: Some(1.0),
                    max_value: Some(100.0),
                    pattern: None,
                    allowed_values: None,
                }),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        default_config.insert(
            "enable_public_sharing".to_string(),
            ConfigValue {
                key: "enable_public_sharing".to_string(),
                value: serde_json::json!(true),
                value_type: ConfigValueType::Boolean,
                scope: ConfigScope::System,
                is_secret: false,
                is_locked: false,
                description: Some("Enable public map sharing".to_string()),
                default_value: Some(serde_json::json!(true)),
                validation: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        default_config.insert(
            "tile_server_url".to_string(),
            ConfigValue {
                key: "tile_server_url".to_string(),
                value: serde_json::json!("https://tiles.example.com"),
                value_type: ConfigValueType::String,
                scope: ConfigScope::System,
                is_secret: false,
                is_locked: false,
                description: Some("Map tile server URL".to_string()),
                default_value: Some(serde_json::json!("https://tiles.example.com")),
                validation: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        Self {
            configs: HashMap::new(),
            default_config,
        }
    }

    /// Gets tenant configuration, creating if it doesn't exist.
    pub fn get_or_create(&mut self, tenant_id: Uuid) -> &mut TenantConfig {
        self.configs
            .entry(tenant_id)
            .or_insert_with(|| TenantConfig::new(tenant_id))
    }

    /// Gets a configuration value for a tenant.
    pub fn get_value(&self, tenant_id: Uuid, key: &str) -> Option<&serde_json::Value> {
        // First check tenant-specific config
        if let Some(config) = self.configs.get(&tenant_id) {
            if let Some(value) = config.get(key) {
                return Some(&value.value);
            }
        }

        // Fall back to default config
        self.default_config.get(key).map(|v| &v.value)
    }

    /// Sets a configuration value for a tenant.
    pub fn set_value(
        &mut self,
        tenant_id: Uuid,
        key: impl Into<String>,
        value: ConfigValue,
    ) -> TenantResult<()> {
        let config = self.get_or_create(tenant_id);
        config.set(key, value)
    }

    /// Removes a configuration value for a tenant.
    pub fn remove_value(&mut self, tenant_id: Uuid, key: &str) -> TenantResult<()> {
        if let Some(config) = self.configs.get_mut(&tenant_id) {
            config.remove(key)?;
        }
        Ok(())
    }

    /// Gets all configuration for a tenant (merged with defaults).
    pub fn get_merged_config(&self, tenant_id: Uuid) -> HashMap<String, serde_json::Value> {
        let mut merged = HashMap::new();

        // Start with defaults
        for (key, value) in &self.default_config {
            merged.insert(key.clone(), value.value.clone());
        }

        // Override with tenant-specific values
        if let Some(config) = self.configs.get(&tenant_id) {
            for (key, value) in &config.settings {
                merged.insert(key.clone(), value.value.clone());
            }
        }

        merged
    }

    /// Exports tenant configuration as JSON.
    pub fn export_config(&self, tenant_id: Uuid) -> TenantResult<String> {
        let config = self.configs.get(&tenant_id).ok_or_else(|| {
            TenantError::TenantNotFound(tenant_id.to_string())
        })?;

        serde_json::to_string_pretty(config)
            .map_err(|e| TenantError::ConfigError(e.to_string()))
    }

    /// Imports tenant configuration from JSON.
    pub fn import_config(&mut self, tenant_id: Uuid, json: &str) -> TenantResult<()> {
        let config: TenantConfig = serde_json::from_str(json)
            .map_err(|e| TenantError::ConfigError(e.to_string()))?;

        self.configs.insert(tenant_id, config);
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration template for quick tenant setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub settings: HashMap<String, ConfigValue>,
    pub tags: Vec<String>,
}

impl ConfigTemplate {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            settings: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Applies this template to a tenant configuration.
    pub fn apply_to(&self, config: &mut TenantConfig) -> TenantResult<()> {
        for (key, value) in &self.settings {
            config.set(key.clone(), value.clone())?;
        }
        Ok(())
    }

    /// Creates a basic GIS template.
    pub fn basic_gis_template() -> Self {
        let mut template = Self::new("Basic GIS");
        template.description = Some("Basic configuration for GIS applications".to_string());

        template.settings.insert(
            "max_zoom_level".to_string(),
            ConfigValue {
                key: "max_zoom_level".to_string(),
                value: serde_json::json!(18),
                value_type: ConfigValueType::Number,
                scope: ConfigScope::Tenant,
                is_secret: false,
                is_locked: false,
                description: Some("Maximum zoom level for maps".to_string()),
                default_value: Some(serde_json::json!(18)),
                validation: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        template.tags.push("gis".to_string());
        template.tags.push("basic".to_string());

        template
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_config() {
        let mut config = TenantConfig::new(Uuid::new_v4());

        let value = ConfigValue {
            key: "test_key".to_string(),
            value: serde_json::json!("test_value"),
            value_type: ConfigValueType::String,
            scope: ConfigScope::Tenant,
            is_secret: false,
            is_locked: false,
            description: None,
            default_value: None,
            validation: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(config.set("test_key", value).is_ok());
        assert_eq!(config.get_string("test_key"), Some("test_value".to_string()));
    }

    #[test]
    fn test_config_manager() {
        let mut manager = ConfigManager::new();
        let tenant_id = Uuid::new_v4();

        let value = ConfigValue {
            key: "custom_setting".to_string(),
            value: serde_json::json!(42),
            value_type: ConfigValueType::Number,
            scope: ConfigScope::Tenant,
            is_secret: false,
            is_locked: false,
            description: None,
            default_value: None,
            validation: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(manager.set_value(tenant_id, "custom_setting", value).is_ok());

        let retrieved = manager.get_value(tenant_id, "custom_setting");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_f64(), Some(42.0));
    }

    #[test]
    fn test_config_template() {
        let template = ConfigTemplate::basic_gis_template();
        let mut config = TenantConfig::new(Uuid::new_v4());

        assert!(template.apply_to(&mut config).is_ok());
        assert!(config.get("max_zoom_level").is_some());
    }
}
