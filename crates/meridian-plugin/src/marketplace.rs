//! Plugin marketplace integration for discovering and installing plugins.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::error::{PluginError, PluginResult};
use crate::signing::SignatureManager;
use crate::versioning::VersionManifest;

/// Plugin marketplace client.
pub struct MarketplaceClient {
    /// HTTP client.
    client: Client,

    /// Marketplace base URL.
    base_url: Url,

    /// API key for authentication.
    api_key: Option<String>,

    /// Local cache directory.
    cache_dir: PathBuf,

    /// Signature manager for verification.
    signature_manager: SignatureManager,
}

impl MarketplaceClient {
    /// Create a new marketplace client.
    pub fn new(base_url: Url, cache_dir: PathBuf) -> PluginResult<Self> {
        let client = Client::builder()
            .user_agent("Meridian-Plugin-Manager/0.1.5")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| PluginError::MarketplaceError(e.to_string()))?;

        Ok(Self {
            client,
            base_url,
            api_key: None,
            cache_dir,
            signature_manager: SignatureManager::new(true),
        })
    }

    /// Set API key for authentication.
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set signature manager.
    pub fn with_signature_manager(mut self, manager: SignatureManager) -> Self {
        self.signature_manager = manager;
        self
    }

    /// Search for plugins in the marketplace.
    pub async fn search(&self, query: &str) -> PluginResult<Vec<MarketplacePlugin>> {
        let url = self.base_url.join("/api/v1/plugins/search")?;

        let mut request = self.client.get(url).query(&[("q", query)]);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(PluginError::MarketplaceError(format!(
                "Search failed: {}",
                response.status()
            )));
        }

        let search_result: SearchResult = response.json().await?;

        Ok(search_result.plugins)
    }

    /// Get plugin details from the marketplace.
    pub async fn get_plugin(&self, plugin_id: &str) -> PluginResult<MarketplacePlugin> {
        let url = self
            .base_url
            .join(&format!("/api/v1/plugins/{}", plugin_id))?;

        let mut request = self.client.get(url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(PluginError::MarketplaceError(format!(
                "Get plugin failed: {}",
                response.status()
            )));
        }

        let plugin: MarketplacePlugin = response.json().await?;

        Ok(plugin)
    }

    /// Download and install a plugin from the marketplace.
    pub async fn install(
        &self,
        plugin_id: &str,
        version: Option<&str>,
        install_dir: &Path,
    ) -> PluginResult<PathBuf> {
        tracing::info!("Installing plugin '{}' from marketplace", plugin_id);

        // Get plugin details
        let plugin = self.get_plugin(plugin_id).await?;

        // Determine version to install
        let version_to_install = if let Some(v) = version {
            plugin
                .versions
                .iter()
                .find(|pv| pv.version == v)
                .ok_or_else(|| {
                    PluginError::MarketplaceError(format!("Version {} not found", v))
                })?
        } else {
            plugin.versions.first().ok_or_else(|| {
                PluginError::MarketplaceError("No versions available".to_string())
            })?
        };

        // Download the plugin
        let plugin_data = self
            .download_plugin(&version_to_install.download_url)
            .await?;

        // Save to install directory
        fs::create_dir_all(install_dir).await?;

        let plugin_filename = format!("{}-{}.plugin", plugin_id, version_to_install.version);
        let plugin_path = install_dir.join(&plugin_filename);

        let mut file = fs::File::create(&plugin_path).await?;
        file.write_all(&plugin_data).await?;
        file.flush().await?;

        // Download and verify signature if available
        if let Some(signature_url) = &version_to_install.signature_url {
            let signature_data = self.download_plugin(signature_url).await?;

            let sig_path = plugin_path.with_extension("sig");
            let mut sig_file = fs::File::create(&sig_path).await?;
            sig_file.write_all(&signature_data).await?;
            sig_file.flush().await?;

            // Verify signature
            self.signature_manager.verify_plugin(&plugin_path).await?;
        }

        tracing::info!("Plugin '{}' installed successfully", plugin_id);

        Ok(plugin_path)
    }

    /// Download plugin data.
    async fn download_plugin(&self, download_url: &str) -> PluginResult<Vec<u8>> {
        let url = Url::parse(download_url)?;

        let mut request = self.client.get(url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(PluginError::MarketplaceError(format!(
                "Download failed: {}",
                response.status()
            )));
        }

        let data = response.bytes().await?;

        Ok(data.to_vec())
    }

    /// Publish a plugin to the marketplace.
    pub async fn publish(
        &self,
        plugin_path: &Path,
        metadata: PublishMetadata,
    ) -> PluginResult<()> {
        tracing::info!("Publishing plugin to marketplace");

        let url = self.base_url.join("/api/v1/plugins")?;

        let api_key = self.api_key.as_ref().ok_or_else(|| {
            PluginError::MarketplaceError("API key required for publishing".to_string())
        })?;

        // Read plugin file
        let plugin_data = fs::read(plugin_path).await?;

        // Create multipart form
        let form = reqwest::multipart::Form::new()
            .text("name", metadata.name)
            .text("description", metadata.description)
            .text("version", metadata.version)
            .text("author", metadata.author)
            .part(
                "plugin",
                reqwest::multipart::Part::bytes(plugin_data)
                    .file_name(
                        plugin_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    )
                    .mime_str("application/octet-stream")
                    .unwrap(),
            );

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PluginError::MarketplaceError(format!(
                "Publish failed: {}",
                response.status()
            )));
        }

        tracing::info!("Plugin published successfully");

        Ok(())
    }

    /// Get featured plugins.
    pub async fn get_featured(&self) -> PluginResult<Vec<MarketplacePlugin>> {
        let url = self.base_url.join("/api/v1/plugins/featured")?;

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(PluginError::MarketplaceError(format!(
                "Get featured failed: {}",
                response.status()
            )));
        }

        let result: FeaturedResult = response.json().await?;

        Ok(result.plugins)
    }

    /// Get plugin statistics.
    pub async fn get_stats(&self, plugin_id: &str) -> PluginResult<PluginStats> {
        let url = self
            .base_url
            .join(&format!("/api/v1/plugins/{}/stats", plugin_id))?;

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(PluginError::MarketplaceError(format!(
                "Get stats failed: {}",
                response.status()
            )));
        }

        let stats: PluginStats = response.json().await?;

        Ok(stats)
    }

    /// Submit a review for a plugin.
    pub async fn submit_review(
        &self,
        plugin_id: &str,
        review: PluginReview,
    ) -> PluginResult<()> {
        let url = self
            .base_url
            .join(&format!("/api/v1/plugins/{}/reviews", plugin_id))?;

        let api_key = self.api_key.as_ref().ok_or_else(|| {
            PluginError::MarketplaceError("API key required for reviews".to_string())
        })?;

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&review)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PluginError::MarketplaceError(format!(
                "Submit review failed: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

/// Marketplace plugin information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub versions: Vec<PluginVersion>,
    pub rating: f32,
    pub downloads: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Plugin version information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    pub version: String,
    pub download_url: String,
    pub signature_url: Option<String>,
    pub changelog: Option<String>,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

/// Search result from marketplace.
#[derive(Debug, Deserialize)]
struct SearchResult {
    plugins: Vec<MarketplacePlugin>,
    total: usize,
}

/// Featured plugins result.
#[derive(Debug, Deserialize)]
struct FeaturedResult {
    plugins: Vec<MarketplacePlugin>,
}

/// Plugin statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    pub downloads: u64,
    pub daily_downloads: u64,
    pub rating: f32,
    pub review_count: u32,
    pub active_installations: u64,
}

/// Plugin review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub rating: u8,
    pub title: String,
    pub comment: String,
}

/// Metadata for publishing a plugin.
#[derive(Debug, Clone)]
pub struct PublishMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
}

/// Local plugin registry for tracking installed plugins.
pub struct LocalRegistry {
    registry_path: PathBuf,
    plugins: HashMap<String, VersionManifest>,
}

impl LocalRegistry {
    /// Create a new local registry.
    pub fn new(registry_path: PathBuf) -> Self {
        Self {
            registry_path,
            plugins: HashMap::new(),
        }
    }

    /// Load the registry from disk.
    pub async fn load(&mut self) -> PluginResult<()> {
        if self.registry_path.exists() {
            let content = fs::read_to_string(&self.registry_path).await?;
            self.plugins = serde_json::from_str(&content)?;
        }

        Ok(())
    }

    /// Save the registry to disk.
    pub async fn save(&self) -> PluginResult<()> {
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(&self.plugins)?;
        fs::write(&self.registry_path, content).await?;

        Ok(())
    }

    /// Register a plugin installation.
    pub fn register(&mut self, manifest: VersionManifest) {
        self.plugins.insert(manifest.plugin_id.clone(), manifest);
    }

    /// Unregister a plugin.
    pub fn unregister(&mut self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }

    /// Get all installed plugins.
    pub fn list(&self) -> Vec<&VersionManifest> {
        self.plugins.values().collect()
    }

    /// Get a specific plugin manifest.
    pub fn get(&self, plugin_id: &str) -> Option<&VersionManifest> {
        self.plugins.get(plugin_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_metadata() {
        let metadata = PublishMetadata {
            name: "Test Plugin".to_string(),
            description: "A test plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            category: "utility".to_string(),
            tags: vec!["test".to_string()],
        };

        assert_eq!(metadata.name, "Test Plugin");
    }

    #[tokio::test]
    async fn test_local_registry() {
        let temp_dir = tempfile::tempdir().unwrap();
        let registry_path = temp_dir.path().join("registry.json");

        let mut registry = LocalRegistry::new(registry_path);

        let manifest = VersionManifest::new(
            "test-plugin".to_string(),
            semver::Version::new(1, 0, 0),
        );

        registry.register(manifest);
        registry.save().await.unwrap();

        let mut registry2 = LocalRegistry::new(registry.registry_path.clone());
        registry2.load().await.unwrap();

        assert!(registry2.get("test-plugin").is_some());
    }
}
