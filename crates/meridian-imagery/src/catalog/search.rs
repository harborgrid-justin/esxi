//! Imagery search and filtering

use crate::error::{ImageryError, Result};
use crate::catalog::{CatalogEntry, StacItem};
use serde::{Deserialize, Serialize};

/// Imagery search interface
pub struct ImagerySearch {
    entries: Vec<CatalogEntry>,
}

impl ImagerySearch {
    /// Create a new search interface
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a catalog entry
    pub fn add_entry(&mut self, entry: CatalogEntry) {
        self.entries.push(entry);
    }

    /// Search with criteria
    pub fn search(&self, criteria: &SearchCriteria) -> SearchResult {
        let mut results = Vec::new();

        for entry in &self.entries {
            if Self::matches_criteria(entry, criteria) {
                results.push(entry.clone());
            }
        }

        SearchResult {
            items: results,
            total: self.entries.len(),
        }
    }

    /// Check if entry matches search criteria
    fn matches_criteria(entry: &CatalogEntry, criteria: &SearchCriteria) -> bool {
        // Check bounding box
        if let Some(bbox) = &criteria.bbox {
            if !Self::bbox_intersects(&entry.bbox, bbox) {
                return false;
            }
        }

        // Check datetime range
        if let Some(start) = &criteria.start_date {
            if entry.datetime < *start {
                return false;
            }
        }

        if let Some(end) = &criteria.end_date {
            if entry.datetime > *end {
                return false;
            }
        }

        // Check cloud cover
        if let Some(max_cloud) = criteria.max_cloud_cover {
            if let Some(cloud) = entry.cloud_cover {
                if cloud > max_cloud {
                    return false;
                }
            }
        }

        // Check platform
        if let Some(ref platform) = criteria.platform {
            if entry.platform != *platform {
                return false;
            }
        }

        true
    }

    /// Check if two bounding boxes intersect
    fn bbox_intersects(bbox1: &[f64; 4], bbox2: &[f64; 4]) -> bool {
        // bbox format: [min_lon, min_lat, max_lon, max_lat]
        !(bbox1[2] < bbox2[0] ||  // bbox1 is left of bbox2
          bbox1[0] > bbox2[2] ||  // bbox1 is right of bbox2
          bbox1[3] < bbox2[1] ||  // bbox1 is below bbox2
          bbox1[1] > bbox2[3])    // bbox1 is above bbox2
    }
}

impl Default for ImagerySearch {
    fn default() -> Self {
        Self::new()
    }
}

/// Search criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchCriteria {
    /// Bounding box [min_lon, min_lat, max_lon, max_lat]
    pub bbox: Option<[f64; 4]>,
    /// Start date
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    /// End date
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Maximum cloud cover percentage
    pub max_cloud_cover: Option<f32>,
    /// Platform filter
    pub platform: Option<String>,
    /// Maximum results
    pub limit: Option<usize>,
}

impl SearchCriteria {
    /// Create new search criteria
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bounding box
    pub fn with_bbox(mut self, min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Self {
        self.bbox = Some([min_lon, min_lat, max_lon, max_lat]);
        self
    }

    /// Set date range
    pub fn with_date_range(
        mut self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    /// Set maximum cloud cover
    pub fn with_max_cloud_cover(mut self, max_cloud: f32) -> Self {
        self.max_cloud_cover = Some(max_cloud);
        self
    }

    /// Set platform filter
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Matching items
    pub items: Vec<CatalogEntry>,
    /// Total items in catalog
    pub total: usize,
}

impl SearchResult {
    /// Get number of results
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Check if results are empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Sort by date (newest first)
    pub fn sort_by_date(&mut self) {
        self.items.sort_by(|a, b| b.datetime.cmp(&a.datetime));
    }

    /// Sort by cloud cover (lowest first)
    pub fn sort_by_cloud_cover(&mut self) {
        self.items.sort_by(|a, b| {
            let a_cloud = a.cloud_cover.unwrap_or(100.0);
            let b_cloud = b.cloud_cover.unwrap_or(100.0);
            a_cloud.partial_cmp(&b_cloud).unwrap()
        });
    }

    /// Apply limit
    pub fn limit(mut self, n: usize) -> Self {
        self.items.truncate(n);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_criteria() {
        let criteria = SearchCriteria::new()
            .with_bbox(-180.0, -90.0, 180.0, 90.0)
            .with_max_cloud_cover(20.0)
            .with_platform("Sentinel-2");

        assert!(criteria.bbox.is_some());
        assert_eq!(criteria.max_cloud_cover, Some(20.0));
        assert_eq!(criteria.platform, Some("Sentinel-2".to_string()));
    }

    #[test]
    fn test_bbox_intersection() {
        let bbox1 = [0.0, 0.0, 10.0, 10.0];
        let bbox2 = [5.0, 5.0, 15.0, 15.0];
        let bbox3 = [20.0, 20.0, 30.0, 30.0];

        assert!(ImagerySearch::bbox_intersects(&bbox1, &bbox2));
        assert!(!ImagerySearch::bbox_intersects(&bbox1, &bbox3));
    }
}
