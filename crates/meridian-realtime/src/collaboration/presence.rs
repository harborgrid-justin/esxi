//! User presence tracking for collaboration

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// User status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// User is online and active
    Online,

    /// User is idle (no activity for a while)
    Idle,

    /// User is away
    Away,

    /// User is busy/do not disturb
    Busy,

    /// User is offline
    Offline,
}

/// User activity type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// Viewing/browsing
    Viewing,

    /// Editing
    Editing,

    /// Commenting
    Commenting,

    /// Drawing
    Drawing,

    /// Measuring
    Measuring,

    /// Analyzing
    Analyzing,

    /// Custom activity
    Custom(String),
}

/// Current user activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Activity type
    pub activity_type: ActivityType,

    /// Target (e.g., feature ID, layer ID)
    pub target: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Started timestamp
    pub started_at: DateTime<Utc>,
}

impl Activity {
    /// Create new activity
    pub fn new(activity_type: ActivityType) -> Self {
        Self {
            activity_type,
            target: None,
            description: None,
            started_at: Utc::now(),
        }
    }

    /// With target
    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    /// With description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    /// User ID
    pub user_id: String,

    /// User name
    pub user_name: String,

    /// User status
    pub status: UserStatus,

    /// Current activity
    pub activity: Option<Activity>,

    /// User location (room, layer, etc.)
    pub location: Option<String>,

    /// User color
    pub color: String,

    /// Avatar URL
    pub avatar_url: Option<String>,

    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,

    /// Session start timestamp
    pub session_started: DateTime<Utc>,

    /// Custom metadata
    pub metadata: serde_json::Value,
}

impl Presence {
    /// Create new presence
    pub fn new(user_id: String, user_name: String) -> Self {
        let now = Utc::now();
        Self {
            user_id: user_id.clone(),
            user_name,
            status: UserStatus::Online,
            activity: None,
            location: None,
            color: super::generate_user_color(&user_id),
            avatar_url: None,
            last_seen: now,
            session_started: now,
            metadata: serde_json::json!({}),
        }
    }

    /// Update status
    pub fn update_status(&mut self, status: UserStatus) {
        self.status = status;
        self.last_seen = Utc::now();
    }

    /// Update activity
    pub fn update_activity(&mut self, activity: Option<Activity>) {
        self.activity = activity;
        self.last_seen = Utc::now();
    }

    /// Update location
    pub fn update_location(&mut self, location: Option<String>) {
        self.location = location;
        self.last_seen = Utc::now();
    }

    /// Touch (update last seen)
    pub fn touch(&mut self) {
        self.last_seen = Utc::now();
    }

    /// Check if user is active (not idle/away/offline)
    pub fn is_active(&self) -> bool {
        matches!(self.status, UserStatus::Online | UserStatus::Busy)
    }

    /// Check if presence is stale
    pub fn is_stale(&self, threshold_secs: i64) -> bool {
        (Utc::now() - self.last_seen).num_seconds() > threshold_secs
    }

    /// Get session duration in seconds
    pub fn session_duration_secs(&self) -> i64 {
        (Utc::now() - self.session_started).num_seconds()
    }
}

/// Presence manager
pub struct PresenceManager {
    /// Presence by user ID
    presences: Arc<DashMap<String, Presence>>,

    /// Idle threshold (seconds)
    idle_threshold: i64,

    /// Offline threshold (seconds)
    offline_threshold: i64,
}

impl PresenceManager {
    /// Create new presence manager
    pub fn new() -> Self {
        Self {
            presences: Arc::new(DashMap::new()),
            idle_threshold: 300,      // 5 minutes
            offline_threshold: 3600,  // 1 hour
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(idle_threshold: i64, offline_threshold: i64) -> Self {
        Self {
            presences: Arc::new(DashMap::new()),
            idle_threshold,
            offline_threshold,
        }
    }

    /// Update presence
    pub fn update_presence(&self, presence: Presence) {
        self.presences.insert(presence.user_id.clone(), presence);
    }

    /// Get presence
    pub fn get_presence(&self, user_id: &str) -> Option<Presence> {
        self.presences.get(user_id).map(|p| p.clone())
    }

    /// Remove presence
    pub fn remove_presence(&self, user_id: &str) -> Option<Presence> {
        self.presences.remove(user_id).map(|(_, p)| p)
    }

    /// Touch presence (update last seen)
    pub fn touch(&self, user_id: &str) {
        if let Some(mut presence) = self.presences.get_mut(user_id) {
            presence.touch();
        }
    }

    /// Update user status
    pub fn update_status(&self, user_id: &str, status: UserStatus) {
        if let Some(mut presence) = self.presences.get_mut(user_id) {
            presence.update_status(status);
        }
    }

    /// Update user activity
    pub fn update_activity(&self, user_id: &str, activity: Option<Activity>) {
        if let Some(mut presence) = self.presences.get_mut(user_id) {
            presence.update_activity(activity);
        }
    }

    /// Update user location
    pub fn update_location(&self, user_id: &str, location: Option<String>) {
        if let Some(mut presence) = self.presences.get_mut(user_id) {
            presence.update_location(location);
        }
    }

    /// Get all presences
    pub fn get_all_presences(&self) -> Vec<Presence> {
        self.presences
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get active users (online or busy)
    pub fn get_active_users(&self) -> Vec<Presence> {
        self.presences
            .iter()
            .filter(|entry| entry.value().is_active())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get users by status
    pub fn get_by_status(&self, status: UserStatus) -> Vec<Presence> {
        self.presences
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get users in location
    pub fn get_in_location(&self, location: &str) -> Vec<Presence> {
        self.presences
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .location
                    .as_ref()
                    .map_or(false, |loc| loc == location)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Update stale presences (set to idle/offline based on thresholds)
    pub fn update_stale_presences(&self) {
        for mut entry in self.presences.iter_mut() {
            let presence = entry.value_mut();
            let inactive_secs = (Utc::now() - presence.last_seen).num_seconds();

            if inactive_secs > self.offline_threshold {
                presence.status = UserStatus::Offline;
            } else if inactive_secs > self.idle_threshold
                && presence.status == UserStatus::Online
            {
                presence.status = UserStatus::Idle;
            }
        }
    }

    /// Remove offline users
    pub fn remove_offline_users(&self) {
        let offline_users: Vec<String> = self
            .presences
            .iter()
            .filter(|entry| entry.value().status == UserStatus::Offline)
            .map(|entry| entry.key().clone())
            .collect();

        for user_id in offline_users {
            self.presences.remove(&user_id);
        }
    }

    /// Get user count
    pub fn count(&self) -> usize {
        self.presences.len()
    }

    /// Get active user count
    pub fn active_count(&self) -> usize {
        self.presences
            .iter()
            .filter(|entry| entry.value().is_active())
            .count()
    }

    /// Clear all presences
    pub fn clear(&self) {
        self.presences.clear();
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_creation() {
        let presence = Presence::new("user1".to_string(), "Alice".to_string());

        assert_eq!(presence.user_id, "user1");
        assert_eq!(presence.user_name, "Alice");
        assert_eq!(presence.status, UserStatus::Online);
        assert!(presence.is_active());
    }

    #[test]
    fn test_activity() {
        let activity = Activity::new(ActivityType::Editing)
            .with_target("feature123".to_string())
            .with_description("Editing road geometry".to_string());

        assert_eq!(activity.target, Some("feature123".to_string()));
        assert!(activity.duration_secs() >= 0);
    }

    #[test]
    fn test_presence_manager() {
        let manager = PresenceManager::new();
        assert_eq!(manager.count(), 0);

        let presence = Presence::new("user1".to_string(), "Alice".to_string());
        manager.update_presence(presence);

        assert_eq!(manager.count(), 1);
        assert_eq!(manager.active_count(), 1);

        manager.update_status("user1", UserStatus::Away);

        let updated = manager.get_presence("user1").unwrap();
        assert_eq!(updated.status, UserStatus::Away);
        assert!(!updated.is_active());
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_presence_location() {
        let manager = PresenceManager::new();

        let mut presence1 = Presence::new("user1".to_string(), "Alice".to_string());
        presence1.update_location(Some("layer1".to_string()));

        let mut presence2 = Presence::new("user2".to_string(), "Bob".to_string());
        presence2.update_location(Some("layer2".to_string()));

        manager.update_presence(presence1);
        manager.update_presence(presence2);

        let in_layer1 = manager.get_in_location("layer1");
        assert_eq!(in_layer1.len(), 1);
        assert_eq!(in_layer1[0].user_id, "user1");
    }
}
