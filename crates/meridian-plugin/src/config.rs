//! Plugin configuration management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{PluginError, PluginResult};

/// Plugin configuration manager.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config_dir: PathBuf,
    configs: HashMap<String, PluginConfigData>,
}

impl ConfigManager {
    /// Create a new configuration manager.
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            configs: HashMap::new(),
        }
    }

    /// Load configuration for a plugin.
    pub async fn load(&mut self, plugin_id: &str) -> PluginResult<PluginConfigData> {
        let config_path = self.config_path(plugin_id);

        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let config: PluginConfigData = toml::from_str(&content)?;
            self.configs.insert(plugin_id.to_string(), config.clone());
            Ok(config)
        } else {
            // Return default configuration
            let config = PluginConfigData::default();
            self.configs.insert(plugin_id.to_string(), config.clone());
            Ok(config)
        }
    }

    /// Save configuration for a plugin.
    pub async fn save(&mut self, plugin_id: &str, config: PluginConfigData) -> PluginResult<()> {
        let config_path = self.config_path(plugin_id);

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(&config)
            .map_err(|e| PluginError::ConfigError {
                id: plugin_id.to_string(),
                reason: e.to_string(),
            })?;

        let mut file = fs::File::create(&config_path).await?;
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        self.configs.insert(plugin_id.to_string(), config);
        Ok(())
    }

    /// Get configuration for a plugin.
    pub fn get(&self, plugin_id: &str) -> Option<&PluginConfigData> {
        self.configs.get(plugin_id)
    }

    /// Get a specific configuration value.
    pub fn get_value(&self, plugin_id: &str, key: &str) -> Option<serde_json::Value> {
        self.configs
            .get(plugin_id)
            .and_then(|config| config.settings.get(key).cloned())
    }

    /// Set a specific configuration value.
    pub fn set_value(
        &mut self,
        plugin_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> PluginResult<()> {
        let config = self.configs.get_mut(plugin_id).ok_or_else(|| {
            PluginError::ConfigError {
                id: plugin_id.to_string(),
                reason: "Configuration not loaded".to_string(),
            }
        })?;

        config.settings.insert(key, value);
        Ok(())
    }

    /// Delete configuration for a plugin.
    pub async fn delete(&mut self, plugin_id: &str) -> PluginResult<()> {
        let config_path = self.config_path(plugin_id);

        if config_path.exists() {
            fs::remove_file(&config_path).await?;
        }

        self.configs.remove(plugin_id);
        Ok(())
    }

    /// Validate configuration against a schema.
    pub fn validate(
        &self,
        plugin_id: &str,
        schema: &ConfigSchema,
    ) -> PluginResult<()> {
        let config = self.configs.get(plugin_id).ok_or_else(|| {
            PluginError::ConfigError {
                id: plugin_id.to_string(),
                reason: "Configuration not loaded".to_string(),
            }
        })?;

        // Check required fields
        for field in &schema.required_fields {
            if !config.settings.contains_key(field) {
                return Err(PluginError::ConfigError {
                    id: plugin_id.to_string(),
                    reason: format!("Required field '{}' is missing", field),
                });
            }
        }

        // Validate field types
        for (key, value) in &config.settings {
            if let Some(expected_type) = schema.field_types.get(key) {
                if !validate_type(value, expected_type) {
                    return Err(PluginError::ConfigError {
                        id: plugin_id.to_string(),
                        reason: format!(
                            "Field '{}' has invalid type, expected {:?}",
                            key, expected_type
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get the configuration file path for a plugin.
    fn config_path(&self, plugin_id: &str) -> PathBuf {
        self.config_dir.join(format!("{}.toml", plugin_id))
    }
}

/// Plugin configuration data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigData {
    /// Configuration version.
    pub version: String,

    /// Whether the plugin is enabled.
    pub enabled: bool,

    /// Plugin settings.
    pub settings: HashMap<String, serde_json::Value>,

    /// Resource limits.
    pub limits: ResourceLimits,

    /// Plugin-specific metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for PluginConfigData {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            enabled: true,
            settings: HashMap::new(),
            limits: ResourceLimits::default(),
            metadata: HashMap::new(),
        }
    }
}

/// Resource limits for a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (0 = unlimited).
    pub max_memory_bytes: u64,

    /// Maximum CPU time in milliseconds (0 = unlimited).
    pub max_cpu_time_ms: u64,

    /// Maximum number of threads (0 = unlimited).
    pub max_threads: usize,

    /// Maximum disk usage in bytes (0 = unlimited).
    pub max_disk_bytes: u64,

    /// Maximum network bandwidth in bytes/sec (0 = unlimited).
    pub max_network_bytes_per_sec: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_cpu_time_ms: 10000,               // 10 seconds
            max_threads: 4,
            max_disk_bytes: 100 * 1024 * 1024, // 100 MB
            max_network_bytes_per_sec: 1024 * 1024, // 1 MB/s
        }
    }
}

/// Configuration schema for validation.
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    /// Required configuration fields.
    pub required_fields: Vec<String>,

    /// Expected types for fields.
    pub field_types: HashMap<String, ConfigFieldType>,

    /// Field descriptions.
    pub descriptions: HashMap<String, String>,
}

impl ConfigSchema {
    /// Create a new configuration schema.
    pub fn new() -> Self {
        Self {
            required_fields: Vec::new(),
            field_types: HashMap::new(),
            descriptions: HashMap::new(),
        }
    }

    /// Add a required field.
    pub fn require(mut self, field: impl Into<String>) -> Self {
        self.required_fields.push(field.into());
        self
    }

    /// Add a field type constraint.
    pub fn field_type(
        mut self,
        field: impl Into<String>,
        field_type: ConfigFieldType,
    ) -> Self {
        self.field_types.insert(field.into(), field_type);
        self
    }

    /// Add a field description.
    pub fn describe(
        mut self,
        field: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.descriptions.insert(field.into(), description.into());
        self
    }
}

impl Default for ConfigSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration field types.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// Validate a value against a field type.
fn validate_type(value: &serde_json::Value, expected: &ConfigFieldType) -> bool {
    match expected {
        ConfigFieldType::String => value.is_string(),
        ConfigFieldType::Number => value.is_number(),
        ConfigFieldType::Boolean => value.is_boolean(),
        ConfigFieldType::Array => value.is_array(),
        ConfigFieldType::Object => value.is_object(),
    }
}

/// Configuration builder for creating plugin configurations.
pub struct ConfigBuilder {
    config: PluginConfigData,
}

impl ConfigBuilder {
    /// Create a new configuration builder.
    pub fn new() -> Self {
        Self {
            config: PluginConfigData::default(),
        }
    }

    /// Set enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Add a setting.
    pub fn setting(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.settings.insert(key.into(), value);
        self
    }

    /// Set resource limits.
    pub fn limits(mut self, limits: ResourceLimits) -> Self {
        self.config.limits = limits;
        self
    }

    /// Add metadata.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the configuration.
    pub fn build(self) -> PluginConfigData {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut manager = ConfigManager::new(temp_dir.path().to_path_buf());

        let config = ConfigBuilder::new()
            .enabled(true)
            .setting("key1", serde_json::json!("value1"))
            .build();

        manager.save("test-plugin", config.clone()).await.unwrap();

        let loaded = manager.load("test-plugin").await.unwrap();
        assert_eq!(loaded.enabled, config.enabled);
        assert_eq!(
            loaded.settings.get("key1"),
            config.settings.get("key1")
        );
    }

    #[test]
    fn test_config_validation() {
        let mut manager = ConfigManager::new(PathBuf::from("/tmp"));

        let config = ConfigBuilder::new()
            .setting("required_field", serde_json::json!("value"))
            .setting("number_field", serde_json::json!(42))
            .build();

        manager.configs.insert("test".to_string(), config);

        let schema = ConfigSchema::new()
            .require("required_field")
            .field_type("number_field", ConfigFieldType::Number);

        assert!(manager.validate("test", &schema).is_ok());

        // Test missing required field
        let schema_missing = ConfigSchema::new().require("missing_field");
        assert!(manager.validate("test", &schema_missing).is_err());
    }
}
