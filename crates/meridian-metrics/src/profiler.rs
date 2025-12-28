//! Performance profiling with flamegraph support.

use crate::error::{MetricsError, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use pprof::ProfilerGuard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerConfig {
    /// Enable profiling
    pub enabled: bool,

    /// Output directory for profiles
    pub output_dir: PathBuf,

    /// Sampling frequency in Hz
    pub frequency: i32,

    /// Enable flamegraph generation
    pub enable_flamegraph: bool,

    /// Automatically profile on startup
    pub auto_profile: bool,

    /// Auto profile duration in seconds
    pub auto_profile_duration_secs: u64,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            output_dir: PathBuf::from("/tmp/meridian-profiles"),
            frequency: 100,
            enable_flamegraph: true,
            auto_profile: false,
            auto_profile_duration_secs: 60,
        }
    }
}

/// Profile session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSession {
    /// Session ID
    pub id: String,

    /// Session name
    pub name: String,

    /// Start time
    pub start_time: DateTime<Utc>,

    /// End time
    pub end_time: Option<DateTime<Utc>>,

    /// Duration in seconds
    pub duration_secs: Option<f64>,

    /// Output file path
    pub output_path: Option<PathBuf>,

    /// Flamegraph path
    pub flamegraph_path: Option<PathBuf>,

    /// Session status
    pub status: ProfileStatus,
}

/// Profile session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    /// Profile is running
    Running,
    /// Profile completed successfully
    Completed,
    /// Profile failed
    Failed,
    /// Profile was cancelled
    Cancelled,
}

/// Performance profiler
pub struct Profiler {
    config: ProfilerConfig,
    active_guard: Arc<RwLock<Option<ProfilerGuard<'static>>>>,
    current_session: Arc<RwLock<Option<ProfileSession>>>,
    sessions: Arc<RwLock<Vec<ProfileSession>>>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new(config: ProfilerConfig) -> Result<Self> {
        // Create output directory if it doesn't exist
        if !config.output_dir.exists() {
            std::fs::create_dir_all(&config.output_dir).map_err(|e| {
                MetricsError::profiling(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        info!("Profiler initialized with output dir: {:?}", config.output_dir);

        Ok(Self {
            config,
            active_guard: Arc::new(RwLock::new(None)),
            current_session: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create with default configuration
    pub fn default() -> Result<Self> {
        Self::new(ProfilerConfig::default())
    }

    /// Start a profiling session
    pub fn start_profile<S: Into<String>>(&self, name: S) -> Result<String> {
        if !self.config.enabled {
            return Err(MetricsError::profiling("Profiler is disabled"));
        }

        let mut guard_lock = self.active_guard.write();
        if guard_lock.is_some() {
            return Err(MetricsError::profiling("A profile is already running"));
        }

        let guard = ProfilerGuard::new(self.config.frequency)
            .map_err(|e| MetricsError::profiling(format!("Failed to start profiler: {}", e)))?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = ProfileSession {
            id: session_id.clone(),
            name: name.into(),
            start_time: Utc::now(),
            end_time: None,
            duration_secs: None,
            output_path: None,
            flamegraph_path: None,
            status: ProfileStatus::Running,
        };

        *guard_lock = Some(guard);
        *self.current_session.write() = Some(session.clone());
        self.sessions.write().push(session);

        info!("Started profiling session: {}", session_id);

        Ok(session_id)
    }

    /// Stop the current profiling session
    pub fn stop_profile(&self) -> Result<ProfileSession> {
        let mut guard_lock = self.active_guard.write();
        let guard = guard_lock
            .take()
            .ok_or_else(|| MetricsError::profiling("No active profile"))?;

        let mut session_lock = self.current_session.write();
        let mut session = session_lock
            .take()
            .ok_or_else(|| MetricsError::profiling("No active session"))?;

        session.end_time = Some(Utc::now());
        session.duration_secs = Some(
            session
                .end_time
                .unwrap()
                .signed_duration_since(session.start_time)
                .num_milliseconds() as f64
                / 1000.0,
        );

        // Build the report
        let report = guard
            .report()
            .build()
            .map_err(|e| MetricsError::profiling(format!("Failed to build report: {}", e)))?;

        // Generate output file path
        let timestamp = session.start_time.format("%Y%m%d_%H%M%S");
        let profile_filename = format!("profile_{}_{}.pb", session.name, timestamp);
        let profile_path = self.config.output_dir.join(&profile_filename);

        // Write protobuf profile
        let file = File::create(&profile_path).map_err(|e| {
            MetricsError::profiling(format!("Failed to create profile file: {}", e))
        })?;

        report.pprof().map_err(|e| {
            MetricsError::profiling(format!("Failed to serialize profile: {}", e))
        })?;

        session.output_path = Some(profile_path.clone());

        // Generate flamegraph if enabled
        if self.config.enable_flamegraph {
            let flamegraph_filename = format!("flamegraph_{}_{}.svg", session.name, timestamp);
            let flamegraph_path = self.config.output_dir.join(&flamegraph_filename);

            let flamegraph_file = File::create(&flamegraph_path).map_err(|e| {
                MetricsError::profiling(format!("Failed to create flamegraph file: {}", e))
            })?;

            report.flamegraph(flamegraph_file).map_err(|e| {
                MetricsError::profiling(format!("Failed to generate flamegraph: {}", e))
            })?;

            session.flamegraph_path = Some(flamegraph_path);
            info!("Flamegraph generated: {:?}", session.flamegraph_path);
        }

        session.status = ProfileStatus::Completed;

        // Update session in history
        let mut sessions = self.sessions.write();
        if let Some(last) = sessions.last_mut() {
            *last = session.clone();
        }

        info!(
            "Profiling session completed: {} (duration: {:.2}s)",
            session.id,
            session.duration_secs.unwrap_or(0.0)
        );

        Ok(session)
    }

    /// Profile a function execution
    pub async fn profile_async<F, Fut, T>(&self, name: &str, f: F) -> Result<(T, ProfileSession)>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let session_id = self.start_profile(name)?;

        let result = f().await;

        let session = self.stop_profile()?;

        Ok((result, session))
    }

    /// Profile a synchronous function
    pub fn profile_sync<F, T>(&self, name: &str, f: F) -> Result<(T, ProfileSession)>
    where
        F: FnOnce() -> T,
    {
        let session_id = self.start_profile(name)?;

        let result = f();

        let session = self.stop_profile()?;

        Ok((result, session))
    }

    /// Cancel the current profiling session
    pub fn cancel_profile(&self) -> Result<()> {
        let mut guard_lock = self.active_guard.write();
        guard_lock.take();

        let mut session_lock = self.current_session.write();
        if let Some(mut session) = session_lock.take() {
            session.status = ProfileStatus::Cancelled;
            session.end_time = Some(Utc::now());

            let mut sessions = self.sessions.write();
            if let Some(last) = sessions.last_mut() {
                *last = session;
            }

            info!("Profiling session cancelled");
        }

        Ok(())
    }

    /// Get the current profiling session
    pub fn current_session(&self) -> Option<ProfileSession> {
        self.current_session.read().clone()
    }

    /// Get all profiling sessions
    pub fn sessions(&self) -> Vec<ProfileSession> {
        self.sessions.read().clone()
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<ProfileSession> {
        self.sessions
            .read()
            .iter()
            .find(|s| s.id == id)
            .cloned()
    }

    /// Clear session history
    pub fn clear_sessions(&self) {
        self.sessions.write().clear();
        info!("Profiling session history cleared");
    }

    /// Check if a profile is currently running
    pub fn is_profiling(&self) -> bool {
        self.active_guard.read().is_some()
    }
}

/// Scoped profiler that automatically stops on drop
pub struct ScopedProfile {
    profiler: Arc<Profiler>,
    session_id: Option<String>,
}

impl ScopedProfile {
    /// Create a new scoped profile
    pub fn new(profiler: Arc<Profiler>, name: &str) -> Result<Self> {
        let session_id = profiler.start_profile(name)?;

        Ok(Self {
            profiler,
            session_id: Some(session_id),
        })
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Manually finish the profile
    pub fn finish(mut self) -> Result<ProfileSession> {
        self.session_id.take(); // Prevent drop from stopping again
        self.profiler.stop_profile()
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        if self.session_id.is_some() {
            if let Err(e) = self.profiler.stop_profile() {
                warn!("Failed to stop scoped profile: {}", e);
            }
        }
    }
}

/// Timing utilities for manual performance measurement
pub struct Timer {
    name: String,
    start: Instant,
    samples: Arc<RwLock<Vec<Duration>>>,
}

impl Timer {
    /// Create a new timer
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            samples: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start a new measurement
    pub fn start(&mut self) {
        self.start = Instant::now();
    }

    /// Record the elapsed time
    pub fn record(&self) -> Duration {
        let elapsed = self.start.elapsed();
        self.samples.write().push(elapsed);
        elapsed
    }

    /// Get all samples
    pub fn samples(&self) -> Vec<Duration> {
        self.samples.read().clone()
    }

    /// Get statistics
    pub fn stats(&self) -> TimerStats {
        let samples = self.samples.read();

        if samples.is_empty() {
            return TimerStats::default();
        }

        let mut durations: Vec<f64> = samples.iter().map(|d| d.as_secs_f64()).collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let sum: f64 = durations.iter().sum();
        let count = durations.len();
        let mean = sum / count as f64;
        let min = durations[0];
        let max = durations[count - 1];

        let p50_idx = (count as f64 * 0.50) as usize;
        let p95_idx = (count as f64 * 0.95) as usize;
        let p99_idx = (count as f64 * 0.99) as usize;

        TimerStats {
            count,
            mean_ms: mean * 1000.0,
            min_ms: min * 1000.0,
            max_ms: max * 1000.0,
            p50_ms: durations[p50_idx.min(count - 1)] * 1000.0,
            p95_ms: durations[p95_idx.min(count - 1)] * 1000.0,
            p99_ms: durations[p99_idx.min(count - 1)] * 1000.0,
        }
    }

    /// Clear all samples
    pub fn clear(&self) {
        self.samples.write().clear();
    }
}

/// Timer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStats {
    pub count: usize,
    pub mean_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

impl Default for TimerStats {
    fn default() -> Self {
        Self {
            count: 0,
            mean_ms: 0.0,
            min_ms: 0.0,
            max_ms: 0.0,
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler_session() {
        let profiler = Profiler::default().unwrap();

        let session_id = profiler.start_profile("test_profile").unwrap();
        assert!(profiler.is_profiling());

        thread::sleep(Duration::from_millis(100));

        let session = profiler.stop_profile().unwrap();
        assert_eq!(session.status, ProfileStatus::Completed);
        assert!(session.duration_secs.unwrap() > 0.0);
        assert!(!profiler.is_profiling());
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::new("test_timer");

        for _ in 0..10 {
            timer.start();
            thread::sleep(Duration::from_millis(10));
            timer.record();
        }

        let stats = timer.stats();
        assert_eq!(stats.count, 10);
        assert!(stats.mean_ms >= 10.0);
    }

    #[test]
    fn test_cancel_profile() {
        let profiler = Profiler::default().unwrap();

        profiler.start_profile("test_cancel").unwrap();
        assert!(profiler.is_profiling());

        profiler.cancel_profile().unwrap();
        assert!(!profiler.is_profiling());

        let sessions = profiler.sessions();
        assert_eq!(sessions.last().unwrap().status, ProfileStatus::Cancelled);
    }
}
