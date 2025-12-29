//! Plugin isolation and resource limiting for security.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::ResourceLimits;
use crate::error::{PluginError, PluginResult};

/// Resource monitor and limiter for plugins.
#[derive(Clone)]
pub struct ResourceMonitor {
    /// Resource usage tracking.
    usage: Arc<RwLock<HashMap<String, ResourceUsage>>>,

    /// Resource limits per plugin.
    limits: Arc<RwLock<HashMap<String, ResourceLimits>>>,

    /// Global limits.
    global_limits: Arc<RwLock<GlobalResourceLimits>>,
}

impl ResourceMonitor {
    /// Create a new resource monitor.
    pub fn new(global_limits: GlobalResourceLimits) -> Self {
        Self {
            usage: Arc::new(RwLock::new(HashMap::new())),
            limits: Arc::new(RwLock::new(HashMap::new())),
            global_limits: Arc::new(RwLock::new(global_limits)),
        }
    }

    /// Set resource limits for a plugin.
    pub fn set_limits(&self, plugin_id: &str, limits: ResourceLimits) {
        self.limits
            .write()
            .insert(plugin_id.to_string(), limits);
    }

    /// Get resource limits for a plugin.
    pub fn get_limits(&self, plugin_id: &str) -> Option<ResourceLimits> {
        self.limits.read().get(plugin_id).cloned()
    }

    /// Initialize tracking for a plugin.
    pub fn start_tracking(&self, plugin_id: &str) {
        self.usage
            .write()
            .insert(plugin_id.to_string(), ResourceUsage::new());
    }

    /// Stop tracking for a plugin.
    pub fn stop_tracking(&self, plugin_id: &str) {
        self.usage.write().remove(plugin_id);
    }

    /// Record memory allocation.
    pub fn record_memory_alloc(&self, plugin_id: &str, bytes: u64) -> PluginResult<()> {
        let mut usage = self.usage.write();
        let plugin_usage = usage
            .entry(plugin_id.to_string())
            .or_insert_with(ResourceUsage::new);

        plugin_usage.memory_bytes += bytes;

        // Check limit
        if let Some(limits) = self.limits.read().get(plugin_id) {
            if limits.max_memory_bytes > 0
                && plugin_usage.memory_bytes > limits.max_memory_bytes
            {
                return Err(PluginError::ResourceLimitExceeded {
                    id: plugin_id.to_string(),
                    resource: format!("memory ({} bytes)", plugin_usage.memory_bytes),
                });
            }
        }

        // Check global limit
        let total_memory: u64 = usage.values().map(|u| u.memory_bytes).sum();
        let global_limits = self.global_limits.read();

        if global_limits.max_total_memory_bytes > 0
            && total_memory > global_limits.max_total_memory_bytes
        {
            return Err(PluginError::ResourceLimitExceeded {
                id: "global".to_string(),
                resource: format!("total memory ({} bytes)", total_memory),
            });
        }

        Ok(())
    }

    /// Record memory deallocation.
    pub fn record_memory_dealloc(&self, plugin_id: &str, bytes: u64) {
        let mut usage = self.usage.write();
        if let Some(plugin_usage) = usage.get_mut(plugin_id) {
            plugin_usage.memory_bytes = plugin_usage.memory_bytes.saturating_sub(bytes);
        }
    }

    /// Record CPU time usage.
    pub fn record_cpu_time(&self, plugin_id: &str, duration: Duration) -> PluginResult<()> {
        let mut usage = self.usage.write();
        let plugin_usage = usage
            .entry(plugin_id.to_string())
            .or_insert_with(ResourceUsage::new);

        plugin_usage.cpu_time += duration;

        // Check limit
        if let Some(limits) = self.limits.read().get(plugin_id) {
            if limits.max_cpu_time_ms > 0
                && plugin_usage.cpu_time.as_millis() as u64 > limits.max_cpu_time_ms
            {
                return Err(PluginError::ResourceLimitExceeded {
                    id: plugin_id.to_string(),
                    resource: format!(
                        "CPU time ({} ms)",
                        plugin_usage.cpu_time.as_millis()
                    ),
                });
            }
        }

        Ok(())
    }

    /// Increment thread count.
    pub fn increment_threads(&self, plugin_id: &str) -> PluginResult<()> {
        let mut usage = self.usage.write();
        let plugin_usage = usage
            .entry(plugin_id.to_string())
            .or_insert_with(ResourceUsage::new);

        plugin_usage.thread_count += 1;

        // Check limit
        if let Some(limits) = self.limits.read().get(plugin_id) {
            if limits.max_threads > 0 && plugin_usage.thread_count > limits.max_threads {
                return Err(PluginError::ResourceLimitExceeded {
                    id: plugin_id.to_string(),
                    resource: format!("threads ({})", plugin_usage.thread_count),
                });
            }
        }

        Ok(())
    }

    /// Decrement thread count.
    pub fn decrement_threads(&self, plugin_id: &str) {
        let mut usage = self.usage.write();
        if let Some(plugin_usage) = usage.get_mut(plugin_id) {
            plugin_usage.thread_count = plugin_usage.thread_count.saturating_sub(1);
        }
    }

    /// Record disk usage.
    pub fn record_disk_usage(&self, plugin_id: &str, bytes: u64) -> PluginResult<()> {
        let mut usage = self.usage.write();
        let plugin_usage = usage
            .entry(plugin_id.to_string())
            .or_insert_with(ResourceUsage::new);

        plugin_usage.disk_bytes = bytes;

        // Check limit
        if let Some(limits) = self.limits.read().get(plugin_id) {
            if limits.max_disk_bytes > 0 && plugin_usage.disk_bytes > limits.max_disk_bytes {
                return Err(PluginError::ResourceLimitExceeded {
                    id: plugin_id.to_string(),
                    resource: format!("disk ({} bytes)", plugin_usage.disk_bytes),
                });
            }
        }

        Ok(())
    }

    /// Record network traffic.
    pub fn record_network_traffic(&self, plugin_id: &str, bytes: u64) -> PluginResult<()> {
        let mut usage = self.usage.write();
        let plugin_usage = usage
            .entry(plugin_id.to_string())
            .or_insert_with(ResourceUsage::new);

        plugin_usage.network_bytes += bytes;

        // Check rate limit
        if let Some(limits) = self.limits.read().get(plugin_id) {
            if limits.max_network_bytes_per_sec > 0 {
                let elapsed = plugin_usage.start_time.elapsed();
                let rate = plugin_usage.network_bytes as f64 / elapsed.as_secs_f64();

                if rate > limits.max_network_bytes_per_sec as f64 {
                    return Err(PluginError::ResourceLimitExceeded {
                        id: plugin_id.to_string(),
                        resource: format!("network rate ({} bytes/s)", rate as u64),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get resource usage for a plugin.
    pub fn get_usage(&self, plugin_id: &str) -> Option<ResourceUsage> {
        self.usage.read().get(plugin_id).cloned()
    }

    /// Get all resource usage.
    pub fn get_all_usage(&self) -> HashMap<String, ResourceUsage> {
        self.usage.read().clone()
    }

    /// Reset resource usage for a plugin.
    pub fn reset_usage(&self, plugin_id: &str) {
        let mut usage = self.usage.write();
        if let Some(plugin_usage) = usage.get_mut(plugin_id) {
            *plugin_usage = ResourceUsage::new();
        }
    }

    /// Check if a plugin is within limits.
    pub fn check_limits(&self, plugin_id: &str) -> PluginResult<()> {
        let usage = self.usage.read();
        let plugin_usage = usage.get(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        let limits = self.limits.read();
        let plugin_limits = limits.get(plugin_id).ok_or_else(|| {
            PluginError::Generic(format!("No limits set for plugin '{}'", plugin_id))
        })?;

        // Check all limits
        if plugin_limits.max_memory_bytes > 0
            && plugin_usage.memory_bytes > plugin_limits.max_memory_bytes
        {
            return Err(PluginError::ResourceLimitExceeded {
                id: plugin_id.to_string(),
                resource: "memory".to_string(),
            });
        }

        if plugin_limits.max_cpu_time_ms > 0
            && plugin_usage.cpu_time.as_millis() as u64 > plugin_limits.max_cpu_time_ms
        {
            return Err(PluginError::ResourceLimitExceeded {
                id: plugin_id.to_string(),
                resource: "CPU time".to_string(),
            });
        }

        if plugin_limits.max_threads > 0 && plugin_usage.thread_count > plugin_limits.max_threads {
            return Err(PluginError::ResourceLimitExceeded {
                id: plugin_id.to_string(),
                resource: "threads".to_string(),
            });
        }

        if plugin_limits.max_disk_bytes > 0
            && plugin_usage.disk_bytes > plugin_limits.max_disk_bytes
        {
            return Err(PluginError::ResourceLimitExceeded {
                id: plugin_id.to_string(),
                resource: "disk".to_string(),
            });
        }

        Ok(())
    }
}

/// Resource usage tracking.
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Memory usage in bytes.
    pub memory_bytes: u64,

    /// CPU time used.
    pub cpu_time: Duration,

    /// Number of threads.
    pub thread_count: usize,

    /// Disk usage in bytes.
    pub disk_bytes: u64,

    /// Network traffic in bytes.
    pub network_bytes: u64,

    /// Start time for rate calculations.
    pub start_time: Instant,
}

impl ResourceUsage {
    /// Create new resource usage tracker.
    pub fn new() -> Self {
        Self {
            memory_bytes: 0,
            cpu_time: Duration::ZERO,
            thread_count: 0,
            disk_bytes: 0,
            network_bytes: 0,
            start_time: Instant::now(),
        }
    }

    /// Get memory usage in MB.
    pub fn memory_mb(&self) -> f64 {
        self.memory_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get CPU time in seconds.
    pub fn cpu_seconds(&self) -> f64 {
        self.cpu_time.as_secs_f64()
    }

    /// Get network rate in bytes/sec.
    pub fn network_rate(&self) -> f64 {
        let elapsed = self.start_time.elapsed();
        if elapsed.as_secs() == 0 {
            0.0
        } else {
            self.network_bytes as f64 / elapsed.as_secs_f64()
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self::new()
    }
}

/// Global resource limits across all plugins.
#[derive(Debug, Clone)]
pub struct GlobalResourceLimits {
    /// Maximum total memory for all plugins.
    pub max_total_memory_bytes: u64,

    /// Maximum number of total threads.
    pub max_total_threads: usize,

    /// Maximum total disk usage.
    pub max_total_disk_bytes: u64,

    /// Maximum total network bandwidth.
    pub max_total_network_bytes_per_sec: u64,
}

impl Default for GlobalResourceLimits {
    fn default() -> Self {
        Self {
            max_total_memory_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            max_total_threads: 100,
            max_total_disk_bytes: 1024 * 1024 * 1024, // 1 GB
            max_total_network_bytes_per_sec: 10 * 1024 * 1024, // 10 MB/s
        }
    }
}

/// Sandbox environment for plugin execution.
pub struct Sandbox {
    /// Plugin ID.
    plugin_id: String,

    /// Resource monitor.
    monitor: ResourceMonitor,

    /// Allowed capabilities.
    capabilities: Vec<Capability>,

    /// File system restrictions.
    fs_restrictions: FileSystemRestrictions,
}

impl Sandbox {
    /// Create a new sandbox.
    pub fn new(
        plugin_id: String,
        monitor: ResourceMonitor,
        capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            plugin_id,
            monitor,
            capabilities,
            fs_restrictions: FileSystemRestrictions::default(),
        }
    }

    /// Check if a capability is allowed.
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Check file system access.
    pub fn check_fs_access(&self, path: &std::path::Path) -> PluginResult<()> {
        // Check if path is in allowed list
        let path_str = path.to_string_lossy();

        for allowed in &self.fs_restrictions.allowed_paths {
            if path_str.starts_with(allowed) {
                return Ok(());
            }
        }

        // Check if path is in denied list
        for denied in &self.fs_restrictions.denied_paths {
            if path_str.starts_with(denied) {
                return Err(PluginError::PermissionDenied {
                    id: self.plugin_id.clone(),
                    action: format!("access path: {}", path_str),
                });
            }
        }

        // Default deny if allow list is specified
        if !self.fs_restrictions.allowed_paths.is_empty() {
            return Err(PluginError::PermissionDenied {
                id: self.plugin_id.clone(),
                action: format!("access path not in allow list: {}", path_str),
            });
        }

        Ok(())
    }

    /// Set file system restrictions.
    pub fn set_fs_restrictions(&mut self, restrictions: FileSystemRestrictions) {
        self.fs_restrictions = restrictions;
    }
}

/// Plugin capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// File system read access.
    FileSystemRead,

    /// File system write access.
    FileSystemWrite,

    /// Network access.
    Network,

    /// Process spawning.
    ProcessSpawn,

    /// Native code execution.
    NativeExecution,

    /// Database access.
    Database,

    /// Inter-plugin communication.
    IPC,

    /// UI rendering.
    UIRender,

    /// Custom capability.
    Custom(String),
}

/// File system access restrictions.
#[derive(Debug, Clone, Default)]
pub struct FileSystemRestrictions {
    /// Allowed paths.
    pub allowed_paths: Vec<String>,

    /// Denied paths.
    pub denied_paths: Vec<String>,

    /// Maximum file size for reads/writes.
    pub max_file_size_bytes: u64,
}

impl FileSystemRestrictions {
    /// Create new restrictions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow a path.
    pub fn allow_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Deny a path.
    pub fn deny_path(mut self, path: impl Into<String>) -> Self {
        self.denied_paths.push(path.into());
        self
    }

    /// Set max file size.
    pub fn max_file_size(mut self, bytes: u64) -> Self {
        self.max_file_size_bytes = bytes;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_tracking() {
        let monitor = ResourceMonitor::new(GlobalResourceLimits::default());

        let limits = ResourceLimits {
            max_memory_bytes: 1024 * 1024, // 1 MB
            max_cpu_time_ms: 1000,
            max_threads: 4,
            max_disk_bytes: 10 * 1024 * 1024,
            max_network_bytes_per_sec: 1024 * 1024,
        };

        monitor.set_limits("test-plugin", limits);
        monitor.start_tracking("test-plugin");

        // Should succeed
        assert!(monitor
            .record_memory_alloc("test-plugin", 512 * 1024)
            .is_ok());

        // Should fail (exceeds limit)
        assert!(monitor
            .record_memory_alloc("test-plugin", 1024 * 1024)
            .is_err());
    }

    #[test]
    fn test_sandbox_capabilities() {
        let monitor = ResourceMonitor::new(GlobalResourceLimits::default());

        let sandbox = Sandbox::new(
            "test-plugin".to_string(),
            monitor,
            vec![Capability::FileSystemRead, Capability::Network],
        );

        assert!(sandbox.has_capability(&Capability::FileSystemRead));
        assert!(sandbox.has_capability(&Capability::Network));
        assert!(!sandbox.has_capability(&Capability::FileSystemWrite));
    }

    #[test]
    fn test_fs_restrictions() {
        let monitor = ResourceMonitor::new(GlobalResourceLimits::default());

        let mut sandbox = Sandbox::new(
            "test-plugin".to_string(),
            monitor,
            vec![Capability::FileSystemRead],
        );

        let restrictions = FileSystemRestrictions::new()
            .allow_path("/tmp/plugins")
            .deny_path("/etc");

        sandbox.set_fs_restrictions(restrictions);

        assert!(sandbox
            .check_fs_access(std::path::Path::new("/tmp/plugins/data"))
            .is_ok());
        assert!(sandbox
            .check_fs_access(std::path::Path::new("/etc/passwd"))
            .is_err());
    }
}
