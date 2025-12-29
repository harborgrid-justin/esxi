//! Pipeline configuration module.
//!
//! Provides declarative pipeline configuration using YAML, JSON, or TOML.

pub mod yaml;

pub use yaml::YamlPipelineConfig;

use crate::error::{ConfigError, PipelineError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Pipeline configuration version.
pub const CONFIG_VERSION: &str = "1.0";

/// Generic pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfigFile {
    /// Configuration version.
    pub version: String,
    /// Pipeline configuration.
    pub pipeline: PipelineDefinition,
}

/// Pipeline definition in config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    /// Pipeline name.
    pub name: String,
    /// Pipeline version.
    #[serde(default = "default_version")]
    pub version: String,
    /// Description.
    pub description: Option<String>,
    /// Execution mode (batch, streaming, micro-batch).
    #[serde(default)]
    pub execution_mode: String,
    /// Parallelism level.
    #[serde(default = "default_parallelism")]
    pub parallelism: usize,
    /// Enable checkpointing.
    #[serde(default)]
    pub checkpointing: bool,
    /// Checkpoint directory.
    pub checkpoint_dir: Option<String>,
    /// Batch size for micro-batch mode.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Data source configuration.
    pub source: Option<SourceConfig>,
    /// Transform configurations.
    #[serde(default)]
    pub transforms: Vec<TransformConfig>,
    /// Data sink configuration.
    pub sink: Option<SinkConfig>,
}

/// Data source configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Source type (file, database, api, stream).
    pub r#type: String,
    /// Additional source-specific configuration.
    #[serde(flatten)]
    pub config: serde_json::Value,
}

/// Transform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// Transform type (projection, filter, aggregate, etc.).
    pub r#type: String,
    /// Additional transform-specific configuration.
    #[serde(flatten)]
    pub config: serde_json::Value,
}

/// Data sink configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkConfig {
    /// Sink type (file, database, vector_tiles).
    pub r#type: String,
    /// Additional sink-specific configuration.
    #[serde(flatten)]
    pub config: serde_json::Value,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_parallelism() -> usize {
    4
}

fn default_batch_size() -> usize {
    1000
}

impl PipelineConfigFile {
    /// Load configuration from YAML file.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            PipelineError::Config(ConfigError::MissingConfig(format!(
                "Cannot read config file: {}",
                e
            )))
        })?;

        let config: Self = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from YAML string.
    pub fn from_yaml_str(yaml: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(yaml)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            PipelineError::Config(ConfigError::MissingConfig(format!(
                "Cannot read config file: {}",
                e
            )))
        })?;

        let config: Self = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from JSON string.
    pub fn from_json_str(json: &str) -> Result<Self> {
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to YAML file.
    pub fn to_yaml_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path.as_ref(), yaml).map_err(|e| {
            PipelineError::Config(ConfigError::MissingConfig(format!(
                "Cannot write config file: {}",
                e
            )))
        })?;
        Ok(())
    }

    /// Convert to YAML string.
    pub fn to_yaml_string(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    /// Convert to JSON string.
    pub fn to_json_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Validate configuration.
    pub fn validate(&self) -> Result<()> {
        // Check version compatibility
        if self.version != CONFIG_VERSION {
            return Err(PipelineError::Config(ConfigError::VersionMismatch {
                expected: CONFIG_VERSION.to_string(),
                found: self.version.clone(),
            }));
        }

        // Validate pipeline name
        if self.pipeline.name.is_empty() {
            return Err(PipelineError::Config(ConfigError::MissingConfig(
                "Pipeline name is required".to_string(),
            )));
        }

        // Validate execution mode
        let valid_modes = ["batch", "streaming", "micro-batch"];
        if !self.pipeline.execution_mode.is_empty()
            && !valid_modes.contains(&self.pipeline.execution_mode.as_str())
        {
            return Err(PipelineError::Config(ConfigError::InvalidValue {
                key: "execution_mode".to_string(),
                value: self.pipeline.execution_mode.clone(),
            }));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_yaml() {
        let yaml = r#"
version: "1.0"
pipeline:
  name: test-pipeline
  version: "1.0.0"
  execution_mode: batch
  parallelism: 4
  source:
    type: file
    format: geojson
    path: input.geojson
  sink:
    type: file
    format: geojson
    path: output.geojson
"#;

        let config = PipelineConfigFile::from_yaml_str(yaml).unwrap();
        assert_eq!(config.pipeline.name, "test-pipeline");
        assert_eq!(config.pipeline.parallelism, 4);
    }

    #[test]
    fn test_config_validation() {
        let yaml = r#"
version: "1.0"
pipeline:
  name: test-pipeline
  execution_mode: invalid-mode
"#;

        let result = PipelineConfigFile::from_yaml_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_version_mismatch() {
        let yaml = r#"
version: "2.0"
pipeline:
  name: test-pipeline
"#;

        let result = PipelineConfigFile::from_yaml_str(yaml);
        assert!(result.is_err());
    }
}
