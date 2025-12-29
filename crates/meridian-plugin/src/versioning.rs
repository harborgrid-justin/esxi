//! Plugin versioning and compatibility checking.

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::error::{PluginError, PluginResult};
use crate::traits::PluginMetadata;

/// Version compatibility checker.
#[derive(Debug, Clone)]
pub struct VersionChecker {
    platform_version: Version,
}

impl VersionChecker {
    /// Create a new version checker with the current platform version.
    pub fn new(platform_version: Version) -> Self {
        Self { platform_version }
    }

    /// Check if a plugin is compatible with the current platform version.
    pub fn is_compatible(&self, metadata: &PluginMetadata) -> PluginResult<()> {
        // Check minimum version requirement
        if self.platform_version < metadata.min_platform_version {
            return Err(PluginError::VersionIncompatible(format!(
                "Plugin '{}' requires platform version {} or higher, but current version is {}",
                metadata.id, metadata.min_platform_version, self.platform_version
            )));
        }

        // Check maximum version requirement if specified
        if let Some(max_version) = &metadata.max_platform_version {
            if self.platform_version > *max_version {
                return Err(PluginError::VersionIncompatible(format!(
                    "Plugin '{}' supports platform version up to {}, but current version is {}",
                    metadata.id, max_version, self.platform_version
                )));
            }
        }

        Ok(())
    }

    /// Check if two plugin versions are compatible.
    pub fn is_version_compatible(
        required: &VersionReq,
        available: &Version,
    ) -> bool {
        required.matches(available)
    }

    /// Get the platform version.
    pub fn platform_version(&self) -> &Version {
        &self.platform_version
    }
}

/// Plugin version manifest for tracking installed versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    /// Plugin ID.
    pub plugin_id: String,

    /// Installed version.
    pub version: Version,

    /// Installation timestamp.
    pub installed_at: chrono::DateTime<chrono::Utc>,

    /// Update history.
    pub update_history: Vec<VersionUpdate>,
}

/// Version update record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionUpdate {
    /// Previous version.
    pub from_version: Version,

    /// New version.
    pub to_version: Version,

    /// Update timestamp.
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Update notes.
    pub notes: Option<String>,
}

impl VersionManifest {
    /// Create a new version manifest.
    pub fn new(plugin_id: String, version: Version) -> Self {
        Self {
            plugin_id,
            version,
            installed_at: chrono::Utc::now(),
            update_history: Vec::new(),
        }
    }

    /// Record a version update.
    pub fn record_update(&mut self, to_version: Version, notes: Option<String>) {
        let update = VersionUpdate {
            from_version: self.version.clone(),
            to_version: to_version.clone(),
            updated_at: chrono::Utc::now(),
            notes,
        };

        self.update_history.push(update);
        self.version = to_version;
    }

    /// Check if an update is available.
    pub fn is_update_available(&self, latest_version: &Version) -> bool {
        latest_version > &self.version
    }

    /// Get update count.
    pub fn update_count(&self) -> usize {
        self.update_history.len()
    }
}

/// Semantic version comparison utilities.
pub mod semver_utils {
    use semver::Version;

    /// Check if a version is a breaking change from another.
    pub fn is_breaking_change(from: &Version, to: &Version) -> bool {
        to.major > from.major
    }

    /// Check if a version is a feature update.
    pub fn is_feature_update(from: &Version, to: &Version) -> bool {
        to.major == from.major && to.minor > from.minor
    }

    /// Check if a version is a patch update.
    pub fn is_patch_update(from: &Version, to: &Version) -> bool {
        to.major == from.major && to.minor == from.minor && to.patch > from.patch
    }

    /// Get the update type as a string.
    pub fn update_type(from: &Version, to: &Version) -> &'static str {
        if is_breaking_change(from, to) {
            "major"
        } else if is_feature_update(from, to) {
            "minor"
        } else if is_patch_update(from, to) {
            "patch"
        } else {
            "unknown"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        let platform_version = Version::new(1, 2, 0);
        let checker = VersionChecker::new(platform_version);

        let mut metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: Version::new(1, 0, 0),
            description: "A test plugin".to_string(),
            authors: vec!["Test Author".to_string()],
            license: None,
            homepage: None,
            dependencies: vec![],
            min_platform_version: Version::new(1, 0, 0),
            max_platform_version: None,
            capabilities: vec![],
            tags: vec![],
        };

        // Should be compatible
        assert!(checker.is_compatible(&metadata).is_ok());

        // Set min version too high
        metadata.min_platform_version = Version::new(2, 0, 0);
        assert!(checker.is_compatible(&metadata).is_err());

        // Set max version too low
        metadata.min_platform_version = Version::new(1, 0, 0);
        metadata.max_platform_version = Some(Version::new(1, 1, 0));
        assert!(checker.is_compatible(&metadata).is_err());
    }

    #[test]
    fn test_version_manifest() {
        let mut manifest = VersionManifest::new(
            "test-plugin".to_string(),
            Version::new(1, 0, 0),
        );

        assert_eq!(manifest.update_count(), 0);

        manifest.record_update(Version::new(1, 1, 0), Some("Feature update".to_string()));
        assert_eq!(manifest.update_count(), 1);
        assert_eq!(manifest.version, Version::new(1, 1, 0));

        assert!(manifest.is_update_available(&Version::new(1, 2, 0)));
        assert!(!manifest.is_update_available(&Version::new(1, 0, 0)));
    }

    #[test]
    fn test_semver_utils() {
        use super::semver_utils::*;

        let v1 = Version::new(1, 0, 0);
        let v2_major = Version::new(2, 0, 0);
        let v1_minor = Version::new(1, 1, 0);
        let v1_patch = Version::new(1, 0, 1);

        assert!(is_breaking_change(&v1, &v2_major));
        assert!(!is_breaking_change(&v1, &v1_minor));

        assert!(is_feature_update(&v1, &v1_minor));
        assert!(!is_feature_update(&v1, &v2_major));

        assert!(is_patch_update(&v1, &v1_patch));
        assert!(!is_patch_update(&v1, &v1_minor));

        assert_eq!(update_type(&v1, &v2_major), "major");
        assert_eq!(update_type(&v1, &v1_minor), "minor");
        assert_eq!(update_type(&v1, &v1_patch), "patch");
    }
}
