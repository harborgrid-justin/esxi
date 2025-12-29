//! Cursor sharing for real-time collaboration

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Geographic cursor position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPosition {
    /// Longitude
    pub lon: f64,

    /// Latitude
    pub lat: f64,

    /// Altitude (optional)
    pub alt: Option<f64>,
}

impl GeoPosition {
    /// Create new geo position
    pub fn new(lon: f64, lat: f64) -> Self {
        Self { lon, lat, alt: None }
    }

    /// With altitude
    pub fn with_altitude(mut self, alt: f64) -> Self {
        self.alt = Some(alt);
        self
    }
}

/// Document cursor position (for text/feature editing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPosition {
    /// Feature ID or layer ID
    pub feature_id: Option<String>,

    /// Property path (e.g., "properties.name")
    pub property_path: Option<String>,

    /// Offset within text
    pub offset: usize,
}

/// Cursor position (can be geographic or document-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CursorPosition {
    /// Geographic position on map
    Geographic(GeoPosition),

    /// Document/text position
    Document(DocumentPosition),

    /// Custom position
    Custom(serde_json::Value),
}

/// User cursor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    /// User ID
    pub user_id: String,

    /// User name
    pub user_name: String,

    /// Cursor position
    pub position: CursorPosition,

    /// User color
    pub color: String,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Custom metadata
    pub metadata: serde_json::Value,
}

impl Cursor {
    /// Create new cursor
    pub fn new(user_id: String, user_name: String, position: CursorPosition) -> Self {
        Self {
            user_id: user_id.clone(),
            user_name,
            position,
            color: super::generate_user_color(&user_id),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    /// Update position
    pub fn update_position(&mut self, position: CursorPosition) {
        self.position = position;
        self.updated_at = Utc::now();
    }

    /// Check if cursor is stale (not updated recently)
    pub fn is_stale(&self, threshold_secs: i64) -> bool {
        (Utc::now() - self.updated_at).num_seconds() > threshold_secs
    }
}

/// Cursor manager for tracking all user cursors
pub struct CursorManager {
    /// Cursors by user ID
    cursors: Arc<DashMap<String, Cursor>>,
}

impl CursorManager {
    /// Create new cursor manager
    pub fn new() -> Self {
        Self {
            cursors: Arc::new(DashMap::new()),
        }
    }

    /// Update cursor position
    pub fn update_cursor(&self, cursor: Cursor) {
        self.cursors.insert(cursor.user_id.clone(), cursor);
    }

    /// Get cursor for user
    pub fn get_cursor(&self, user_id: &str) -> Option<Cursor> {
        self.cursors.get(user_id).map(|c| c.clone())
    }

    /// Remove cursor
    pub fn remove_cursor(&self, user_id: &str) {
        self.cursors.remove(user_id);
    }

    /// Get all cursors
    pub fn get_all_cursors(&self) -> Vec<Cursor> {
        self.cursors.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get cursors in geographic bounds
    pub fn get_cursors_in_bounds(
        &self,
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    ) -> Vec<Cursor> {
        self.cursors
            .iter()
            .filter(|entry| {
                if let CursorPosition::Geographic(pos) = &entry.value().position {
                    pos.lon >= min_lon
                        && pos.lon <= max_lon
                        && pos.lat >= min_lat
                        && pos.lat <= max_lat
                } else {
                    false
                }
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove stale cursors
    pub fn remove_stale_cursors(&self, threshold_secs: i64) {
        let stale_users: Vec<String> = self
            .cursors
            .iter()
            .filter(|entry| entry.value().is_stale(threshold_secs))
            .map(|entry| entry.key().clone())
            .collect();

        for user_id in stale_users {
            self.cursors.remove(&user_id);
        }
    }

    /// Get cursor count
    pub fn count(&self) -> usize {
        self.cursors.len()
    }

    /// Clear all cursors
    pub fn clear(&self) {
        self.cursors.clear();
    }
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_position() {
        let pos = GeoPosition::new(-122.4194, 37.7749);
        assert_eq!(pos.lon, -122.4194);
        assert_eq!(pos.lat, 37.7749);
        assert_eq!(pos.alt, None);

        let pos_with_alt = pos.with_altitude(100.0);
        assert_eq!(pos_with_alt.alt, Some(100.0));
    }

    #[test]
    fn test_cursor_creation() {
        let pos = CursorPosition::Geographic(GeoPosition::new(-122.4194, 37.7749));
        let cursor = Cursor::new("user1".to_string(), "Alice".to_string(), pos);

        assert_eq!(cursor.user_id, "user1");
        assert_eq!(cursor.user_name, "Alice");
        assert!(!cursor.is_stale(60));
    }

    #[test]
    fn test_cursor_manager() {
        let manager = CursorManager::new();
        assert_eq!(manager.count(), 0);

        let pos = CursorPosition::Geographic(GeoPosition::new(-122.4194, 37.7749));
        let cursor = Cursor::new("user1".to_string(), "Alice".to_string(), pos);

        manager.update_cursor(cursor.clone());
        assert_eq!(manager.count(), 1);

        let retrieved = manager.get_cursor("user1").unwrap();
        assert_eq!(retrieved.user_id, "user1");

        manager.remove_cursor("user1");
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_cursors_in_bounds() {
        let manager = CursorManager::new();

        // Add cursors at different positions
        let cursor1 = Cursor::new(
            "user1".to_string(),
            "Alice".to_string(),
            CursorPosition::Geographic(GeoPosition::new(-122.4194, 37.7749)),
        );
        let cursor2 = Cursor::new(
            "user2".to_string(),
            "Bob".to_string(),
            CursorPosition::Geographic(GeoPosition::new(-74.0060, 40.7128)),
        );

        manager.update_cursor(cursor1);
        manager.update_cursor(cursor2);

        // Query bounds around San Francisco
        let in_bounds = manager.get_cursors_in_bounds(-123.0, 37.0, -122.0, 38.0);
        assert_eq!(in_bounds.len(), 1);
        assert_eq!(in_bounds[0].user_id, "user1");
    }
}
