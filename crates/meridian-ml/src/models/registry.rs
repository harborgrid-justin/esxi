//! Model registry and versioning system

use crate::error::{MlError, Result};
use crate::models::{Model, ModelMetadata};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Model version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModelVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl ModelVersion {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse from string (e.g., "1.2.3")
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(MlError::InvalidConfig(format!(
                "Invalid version format: {}",
                s
            )));
        }

        Ok(Self {
            major: parts[0]
                .parse()
                .map_err(|_| MlError::InvalidConfig("Invalid major version".to_string()))?,
            minor: parts[1]
                .parse()
                .map_err(|_| MlError::InvalidConfig("Invalid minor version".to_string()))?,
            patch: parts[2]
                .parse()
                .map_err(|_| MlError::InvalidConfig("Invalid patch version".to_string()))?,
        })
    }

    /// Increment major version
    pub fn increment_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    /// Increment minor version
    pub fn increment_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    /// Increment patch version
    pub fn increment_patch(&mut self) {
        self.patch += 1;
    }
}

impl std::fmt::Display for ModelVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for ModelVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ModelVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => match self.minor.cmp(&other.minor) {
                std::cmp::Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

/// Registered model entry
#[derive(Debug, Clone)]
struct RegisteredModel {
    metadata: ModelMetadata,
    version: ModelVersion,
    path: Option<PathBuf>,
    registered_at: DateTime<Utc>,
}

/// Model registry for managing multiple models
pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<String, Vec<RegisteredModel>>>>,
    base_path: Option<PathBuf>,
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            base_path: None,
        }
    }

    /// Create a new model registry with a base path
    pub fn with_base_path<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            base_path: Some(path.into()),
        }
    }

    /// Register a new model
    pub async fn register(
        &self,
        name: String,
        metadata: ModelMetadata,
        version: Option<ModelVersion>,
    ) -> Result<ModelVersion> {
        let mut models = self.models.write().await;

        let version = version.unwrap_or_else(|| {
            // Auto-increment version
            if let Some(versions) = models.get(&name) {
                if let Some(latest) = versions.iter().map(|m| &m.version).max() {
                    let mut next = latest.clone();
                    next.increment_patch();
                    return next;
                }
            }
            ModelVersion::new(1, 0, 0)
        });

        let entry = RegisteredModel {
            metadata,
            version: version.clone(),
            path: None,
            registered_at: Utc::now(),
        };

        models.entry(name).or_insert_with(Vec::new).push(entry);

        Ok(version)
    }

    /// Get model metadata by name and version
    pub async fn get(
        &self,
        name: &str,
        version: Option<&ModelVersion>,
    ) -> Result<ModelMetadata> {
        let models = self.models.read().await;

        let versions = models
            .get(name)
            .ok_or_else(|| MlError::ModelNotFound(name.to_string()))?;

        let model = if let Some(v) = version {
            versions
                .iter()
                .find(|m| &m.version == v)
                .ok_or_else(|| {
                    MlError::ModelNotFound(format!("{}:{}", name, v))
                })?
        } else {
            // Get latest version
            versions
                .iter()
                .max_by_key(|m| &m.version)
                .ok_or_else(|| MlError::ModelNotFound(name.to_string()))?
        };

        Ok(model.metadata.clone())
    }

    /// List all registered models
    pub async fn list(&self) -> Vec<(String, Vec<ModelVersion>)> {
        let models = self.models.read().await;
        models
            .iter()
            .map(|(name, versions)| {
                let vers = versions.iter().map(|m| m.version.clone()).collect();
                (name.clone(), vers)
            })
            .collect()
    }

    /// Get all versions of a model
    pub async fn versions(&self, name: &str) -> Result<Vec<ModelVersion>> {
        let models = self.models.read().await;
        let versions = models
            .get(name)
            .ok_or_else(|| MlError::ModelNotFound(name.to_string()))?;

        let mut vers: Vec<ModelVersion> = versions.iter().map(|m| m.version.clone()).collect();
        vers.sort();
        vers.reverse(); // Latest first
        Ok(vers)
    }

    /// Get the latest version of a model
    pub async fn latest_version(&self, name: &str) -> Result<ModelVersion> {
        let models = self.models.read().await;
        let versions = models
            .get(name)
            .ok_or_else(|| MlError::ModelNotFound(name.to_string()))?;

        versions
            .iter()
            .map(|m| &m.version)
            .max()
            .cloned()
            .ok_or_else(|| MlError::ModelNotFound(name.to_string()))
    }

    /// Delete a model version
    pub async fn delete(&self, name: &str, version: &ModelVersion) -> Result<()> {
        let mut models = self.models.write().await;

        if let Some(versions) = models.get_mut(name) {
            versions.retain(|m| &m.version != version);
            if versions.is_empty() {
                models.remove(name);
            }
            Ok(())
        } else {
            Err(MlError::ModelNotFound(name.to_string()))
        }
    }

    /// Delete all versions of a model
    pub async fn delete_all(&self, name: &str) -> Result<()> {
        let mut models = self.models.write().await;
        models
            .remove(name)
            .ok_or_else(|| MlError::ModelNotFound(name.to_string()))?;
        Ok(())
    }

    /// Tag a model version
    pub async fn tag(
        &self,
        name: &str,
        version: &ModelVersion,
        tag: String,
    ) -> Result<()> {
        let mut models = self.models.write().await;

        if let Some(versions) = models.get_mut(name) {
            if let Some(model) = versions.iter_mut().find(|m| &m.version == version) {
                model.metadata.add_tag(tag);
                Ok(())
            } else {
                Err(MlError::ModelNotFound(format!("{}:{}", name, version)))
            }
        } else {
            Err(MlError::ModelNotFound(name.to_string()))
        }
    }

    /// Search models by tags
    pub async fn search_by_tags(&self, tags: &[String]) -> Vec<(String, ModelVersion)> {
        let models = self.models.read().await;
        let mut results = Vec::new();

        for (name, versions) in models.iter() {
            for model in versions {
                if tags.iter().all(|tag| model.metadata.tags.contains(tag)) {
                    results.push((name.clone(), model.version.clone()));
                }
            }
        }

        results
    }

    /// Get the total number of registered models
    pub async fn count(&self) -> usize {
        let models = self.models.read().await;
        models.len()
    }

    /// Clear all models from the registry
    pub async fn clear(&self) {
        let mut models = self.models.write().await;
        models.clear();
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_version() {
        let v1 = ModelVersion::new(1, 2, 3);
        assert_eq!(v1.to_string(), "1.2.3");

        let v2 = ModelVersion::from_string("2.0.1").unwrap();
        assert_eq!(v2.major, 2);
        assert_eq!(v2.minor, 0);
        assert_eq!(v2.patch, 1);

        assert!(v2 > v1);
    }

    #[test]
    fn test_version_increment() {
        let mut v = ModelVersion::new(1, 2, 3);
        v.increment_patch();
        assert_eq!(v.to_string(), "1.2.4");

        v.increment_minor();
        assert_eq!(v.to_string(), "1.3.0");

        v.increment_major();
        assert_eq!(v.to_string(), "2.0.0");
    }

    #[tokio::test]
    async fn test_registry() {
        let registry = ModelRegistry::new();
        let metadata = ModelMetadata::new("test".to_string(), "classifier".to_string(), 10, 2);

        let version = registry
            .register("test_model".to_string(), metadata, None)
            .await
            .unwrap();

        assert_eq!(version, ModelVersion::new(1, 0, 0));

        let retrieved = registry.get("test_model", None).await.unwrap();
        assert_eq!(retrieved.name, "test");
    }
}
