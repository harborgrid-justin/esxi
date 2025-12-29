//! Selection sharing for collaborative editing

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start position
    pub start: usize,

    /// End position
    pub end: usize,

    /// Direction (forward or backward)
    pub is_forward: bool,
}

impl SelectionRange {
    /// Create new selection range
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            is_forward: end >= start,
        }
    }

    /// Get selection length
    pub fn length(&self) -> usize {
        if self.end >= self.start {
            self.end - self.start
        } else {
            self.start - self.end
        }
    }

    /// Check if collapsed (zero length)
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    /// Check if position is within selection
    pub fn contains(&self, position: usize) -> bool {
        let (min, max) = if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        };
        position >= min && position <= max
    }
}

/// Feature selection (for GIS features)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSelection {
    /// Feature IDs
    pub feature_ids: Vec<String>,

    /// Layer ID
    pub layer_id: Option<String>,
}

impl FeatureSelection {
    /// Create new feature selection
    pub fn new(feature_ids: Vec<String>) -> Self {
        Self {
            feature_ids,
            layer_id: None,
        }
    }

    /// With layer ID
    pub fn with_layer(mut self, layer_id: String) -> Self {
        self.layer_id = Some(layer_id);
        self
    }

    /// Get selection count
    pub fn count(&self) -> usize {
        self.feature_ids.len()
    }

    /// Check if feature is selected
    pub fn contains_feature(&self, feature_id: &str) -> bool {
        self.feature_ids.iter().any(|id| id == feature_id)
    }
}

/// Selection type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SelectionType {
    /// Text/document range selection
    Range(SelectionRange),

    /// Feature selection
    Features(FeatureSelection),

    /// Geographic area selection
    Geographic {
        /// Bounding box [min_lon, min_lat, max_lon, max_lat]
        bbox: [f64; 4],
    },

    /// Custom selection
    Custom(serde_json::Value),
}

/// User selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    /// User ID
    pub user_id: String,

    /// User name
    pub user_name: String,

    /// Selection type
    pub selection: SelectionType,

    /// User color
    pub color: String,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Custom metadata
    pub metadata: serde_json::Value,
}

impl Selection {
    /// Create new selection
    pub fn new(user_id: String, user_name: String, selection: SelectionType) -> Self {
        Self {
            user_id: user_id.clone(),
            user_name,
            selection,
            color: super::generate_user_color(&user_id),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    /// Update selection
    pub fn update_selection(&mut self, selection: SelectionType) {
        self.selection = selection;
        self.updated_at = Utc::now();
    }

    /// Check if selection is stale
    pub fn is_stale(&self, threshold_secs: i64) -> bool {
        (Utc::now() - self.updated_at).num_seconds() > threshold_secs
    }
}

/// Selection manager
pub struct SelectionManager {
    /// Selections by user ID
    selections: Arc<DashMap<String, Selection>>,
}

impl SelectionManager {
    /// Create new selection manager
    pub fn new() -> Self {
        Self {
            selections: Arc::new(DashMap::new()),
        }
    }

    /// Update selection
    pub fn update_selection(&self, selection: Selection) {
        self.selections.insert(selection.user_id.clone(), selection);
    }

    /// Get selection for user
    pub fn get_selection(&self, user_id: &str) -> Option<Selection> {
        self.selections.get(user_id).map(|s| s.clone())
    }

    /// Remove selection
    pub fn remove_selection(&self, user_id: &str) {
        self.selections.remove(user_id);
    }

    /// Get all selections
    pub fn get_all_selections(&self) -> Vec<Selection> {
        self.selections
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get selections for a specific feature
    pub fn get_selections_for_feature(&self, feature_id: &str) -> Vec<Selection> {
        self.selections
            .iter()
            .filter(|entry| {
                if let SelectionType::Features(fs) = &entry.value().selection {
                    fs.contains_feature(feature_id)
                } else {
                    false
                }
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get selections in geographic bounds
    pub fn get_selections_in_bounds(
        &self,
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    ) -> Vec<Selection> {
        self.selections
            .iter()
            .filter(|entry| {
                if let SelectionType::Geographic { bbox } = &entry.value().selection {
                    // Check if bboxes overlap
                    bbox[0] <= max_lon
                        && bbox[2] >= min_lon
                        && bbox[1] <= max_lat
                        && bbox[3] >= min_lat
                } else {
                    false
                }
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove stale selections
    pub fn remove_stale_selections(&self, threshold_secs: i64) {
        let stale_users: Vec<String> = self
            .selections
            .iter()
            .filter(|entry| entry.value().is_stale(threshold_secs))
            .map(|entry| entry.key().clone())
            .collect();

        for user_id in stale_users {
            self.selections.remove(&user_id);
        }
    }

    /// Get selection count
    pub fn count(&self) -> usize {
        self.selections.len()
    }

    /// Clear all selections
    pub fn clear(&self) {
        self.selections.clear();
    }
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_range() {
        let range = SelectionRange::new(10, 20);
        assert_eq!(range.length(), 10);
        assert!(!range.is_collapsed());
        assert!(range.contains(15));
        assert!(!range.contains(5));
        assert!(!range.contains(25));
    }

    #[test]
    fn test_feature_selection() {
        let selection = FeatureSelection::new(vec!["feat1".to_string(), "feat2".to_string()])
            .with_layer("layer1".to_string());

        assert_eq!(selection.count(), 2);
        assert!(selection.contains_feature("feat1"));
        assert!(!selection.contains_feature("feat3"));
    }

    #[test]
    fn test_selection_manager() {
        let manager = SelectionManager::new();
        assert_eq!(manager.count(), 0);

        let selection = Selection::new(
            "user1".to_string(),
            "Alice".to_string(),
            SelectionType::Range(SelectionRange::new(0, 10)),
        );

        manager.update_selection(selection.clone());
        assert_eq!(manager.count(), 1);

        let retrieved = manager.get_selection("user1").unwrap();
        assert_eq!(retrieved.user_id, "user1");

        manager.remove_selection("user1");
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_selections_for_feature() {
        let manager = SelectionManager::new();

        let selection1 = Selection::new(
            "user1".to_string(),
            "Alice".to_string(),
            SelectionType::Features(FeatureSelection::new(vec!["feat1".to_string()])),
        );

        let selection2 = Selection::new(
            "user2".to_string(),
            "Bob".to_string(),
            SelectionType::Features(FeatureSelection::new(vec!["feat2".to_string()])),
        );

        manager.update_selection(selection1);
        manager.update_selection(selection2);

        let selections = manager.get_selections_for_feature("feat1");
        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].user_id, "user1");
    }
}
