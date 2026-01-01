//! Model registry for versioning and management
//!
//! Provides a centralized registry for managing ML models

use super::{ModelInfo, ModelFormat};
use crate::{Error, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Version identifier
    pub id: Uuid,

    /// Version string (e.g., "1.0.0")
    pub version: String,

    /// Version description
    pub description: String,

    /// Model path
    pub path: PathBuf,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Whether this is the active version
    pub is_active: bool,

    /// Performance metrics for this version
    pub metrics: std::collections::HashMap<String, f64>,

    /// Tags
    pub tags: Vec<String>,
}

impl ModelVersion {
    /// Create a new model version
    pub fn new(version: impl Into<String>, path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            version: version.into(),
            description: String::new(),
            path,
            created_at: chrono::Utc::now(),
            is_active: false,
            metrics: std::collections::HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Set as active version
    pub fn set_active(&mut self) {
        self.is_active = true;
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }
}

/// Model metadata in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model identifier
    pub id: Uuid,

    /// Model name
    pub name: String,

    /// Model format
    pub format: ModelFormat,

    /// All versions of this model
    pub versions: Vec<ModelVersion>,

    /// Active version ID
    pub active_version_id: Option<Uuid>,

    /// Model description
    pub description: String,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Model owner/author
    pub owner: String,

    /// Model tags
    pub tags: Vec<String>,
}

impl ModelMetadata {
    /// Create new model metadata
    pub fn new(name: impl Into<String>, format: ModelFormat) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            format,
            versions: Vec::new(),
            active_version_id: None,
            description: String::new(),
            created_at: now,
            updated_at: now,
            owner: "system".to_string(),
            tags: Vec::new(),
        }
    }

    /// Add a new version
    pub fn add_version(&mut self, version: ModelVersion) {
        self.versions.push(version);
        self.updated_at = chrono::Utc::now();
    }

    /// Get the active version
    pub fn active_version(&self) -> Option<&ModelVersion> {
        self.active_version_id.and_then(|id| {
            self.versions.iter().find(|v| v.id == id)
        })
    }

    /// Set active version by ID
    pub fn set_active_version(&mut self, version_id: Uuid) -> Result<()> {
        if !self.versions.iter().any(|v| v.id == version_id) {
            return Err(Error::model_load("Version not found"));
        }

        // Deactivate all versions
        for version in &mut self.versions {
            version.is_active = false;
        }

        // Activate the specified version
        if let Some(version) = self.versions.iter_mut().find(|v| v.id == version_id) {
            version.is_active = true;
            self.active_version_id = Some(version_id);
        }

        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Get version by string
    pub fn get_version(&self, version: &str) -> Option<&ModelVersion> {
        self.versions.iter().find(|v| v.version == version)
    }
}

/// Centralized model registry
pub struct ModelRegistry {
    /// Registry storage
    models: Arc<DashMap<String, ModelMetadata>>,

    /// Registry root path
    root_path: PathBuf,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new(root_path: impl AsRef<Path>) -> Self {
        Self {
            models: Arc::new(DashMap::new()),
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    /// Register a new model
    pub fn register(
        &self,
        name: impl Into<String>,
        format: ModelFormat,
    ) -> Result<Uuid> {
        let name = name.into();

        if self.models.contains_key(&name) {
            return Err(Error::pipeline(format!("Model '{}' already registered", name)));
        }

        let metadata = ModelMetadata::new(name.clone(), format);
        let id = metadata.id;

        self.models.insert(name, metadata);

        Ok(id)
    }

    /// Register a model version
    pub fn register_version(
        &self,
        model_name: &str,
        version: impl Into<String>,
        model_path: PathBuf,
    ) -> Result<Uuid> {
        let mut metadata = self.models.get_mut(model_name)
            .ok_or_else(|| Error::model_load(format!("Model '{}' not found", model_name)))?;

        let model_version = ModelVersion::new(version, model_path);
        let version_id = model_version.id;

        metadata.add_version(model_version);

        // If this is the first version, make it active
        if metadata.versions.len() == 1 {
            metadata.set_active_version(version_id)?;
        }

        Ok(version_id)
    }

    /// Get model metadata
    pub fn get_model(&self, name: &str) -> Option<ModelMetadata> {
        self.models.get(name).map(|entry| entry.clone())
    }

    /// Get active model version
    pub fn get_active_version(&self, name: &str) -> Option<ModelVersion> {
        self.models.get(name).and_then(|metadata| {
            metadata.active_version().cloned()
        })
    }

    /// Set active version for a model
    pub fn set_active_version(&self, model_name: &str, version_id: Uuid) -> Result<()> {
        let mut metadata = self.models.get_mut(model_name)
            .ok_or_else(|| Error::model_load(format!("Model '{}' not found", model_name)))?;

        metadata.set_active_version(version_id)
    }

    /// List all registered models
    pub fn list_models(&self) -> Vec<String> {
        self.models.iter().map(|entry| entry.key().clone()).collect()
    }

    /// List all versions for a model
    pub fn list_versions(&self, model_name: &str) -> Vec<ModelVersion> {
        self.models.get(model_name)
            .map(|metadata| metadata.versions.clone())
            .unwrap_or_default()
    }

    /// Remove a model from the registry
    pub fn unregister(&self, model_name: &str) -> Result<()> {
        self.models.remove(model_name)
            .ok_or_else(|| Error::model_load(format!("Model '{}' not found", model_name)))?;
        Ok(())
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let total_models = self.models.len();
        let total_versions: usize = self.models.iter()
            .map(|entry| entry.versions.len())
            .sum();

        RegistryStats {
            total_models,
            total_versions,
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of models
    pub total_models: usize,

    /// Total number of versions across all models
    pub total_versions: usize,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new("/var/lib/meridian/models")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_version_creation() {
        let version = ModelVersion::new("1.0.0", PathBuf::from("/models/v1"));
        assert_eq!(version.version, "1.0.0");
        assert!(!version.is_active);
    }

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata::new("test-model", ModelFormat::Onnx);
        assert_eq!(metadata.name, "test-model");
        assert_eq!(metadata.format, ModelFormat::Onnx);
        assert!(metadata.versions.is_empty());
    }

    #[test]
    fn test_registry_creation() {
        let registry = ModelRegistry::new("/tmp/models");
        assert_eq!(registry.list_models().len(), 0);
    }

    #[test]
    fn test_register_model() {
        let registry = ModelRegistry::new("/tmp/models");
        let result = registry.register("test-model", ModelFormat::Onnx);
        assert!(result.is_ok());

        let models = registry.list_models();
        assert_eq!(models.len(), 1);
        assert!(models.contains(&"test-model".to_string()));
    }

    #[test]
    fn test_register_duplicate_model() {
        let registry = ModelRegistry::new("/tmp/models");
        registry.register("test-model", ModelFormat::Onnx).unwrap();
        let result = registry.register("test-model", ModelFormat::Onnx);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_version() {
        let registry = ModelRegistry::new("/tmp/models");
        registry.register("test-model", ModelFormat::Onnx).unwrap();

        let result = registry.register_version(
            "test-model",
            "1.0.0",
            PathBuf::from("/models/v1.onnx")
        );
        assert!(result.is_ok());

        let versions = registry.list_versions("test-model");
        assert_eq!(versions.len(), 1);
    }

    #[test]
    fn test_registry_stats() {
        let registry = ModelRegistry::new("/tmp/models");
        registry.register("model1", ModelFormat::Onnx).unwrap();
        registry.register("model2", ModelFormat::Onnx).unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_models, 2);
    }
}
