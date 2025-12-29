//! Real-time annotations for collaborative GIS work

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Annotation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    /// Comment/note
    Comment,

    /// Question
    Question,

    /// Issue/problem
    Issue,

    /// Suggestion
    Suggestion,

    /// Marker/pin
    Marker,

    /// Drawing/sketch
    Drawing,

    /// Measurement
    Measurement,

    /// Custom type
    Custom(String),
}

/// Annotation geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnnotationGeometry {
    /// Point annotation
    Point {
        /// Longitude
        lon: f64,
        /// Latitude
        lat: f64,
    },

    /// Line annotation
    LineString {
        /// Coordinates
        coordinates: Vec<[f64; 2]>,
    },

    /// Polygon annotation
    Polygon {
        /// Exterior ring
        exterior: Vec<[f64; 2]>,
        /// Interior rings (holes)
        interiors: Vec<Vec<[f64; 2]>>,
    },

    /// Bounding box
    BoundingBox {
        /// [min_lon, min_lat, max_lon, max_lat]
        bbox: [f64; 4],
    },
}

impl AnnotationGeometry {
    /// Create point annotation
    pub fn point(lon: f64, lat: f64) -> Self {
        Self::Point { lon, lat }
    }

    /// Create line annotation
    pub fn line(coordinates: Vec<[f64; 2]>) -> Self {
        Self::LineString { coordinates }
    }

    /// Create polygon annotation
    pub fn polygon(exterior: Vec<[f64; 2]>) -> Self {
        Self::Polygon {
            exterior,
            interiors: vec![],
        }
    }

    /// Create bounding box annotation
    pub fn bbox(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Self {
        Self::BoundingBox {
            bbox: [min_lon, min_lat, max_lon, max_lat],
        }
    }
}

/// Annotation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationStatus {
    /// Open/active
    Open,

    /// In progress
    InProgress,

    /// Resolved
    Resolved,

    /// Closed
    Closed,

    /// Archived
    Archived,
}

/// Annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// Annotation ID
    pub id: String,

    /// Annotation type
    pub annotation_type: AnnotationType,

    /// Author user ID
    pub author_id: String,

    /// Author name
    pub author_name: String,

    /// Geometry
    pub geometry: AnnotationGeometry,

    /// Content/text
    pub content: String,

    /// Status
    pub status: AnnotationStatus,

    /// Tags
    pub tags: Vec<String>,

    /// Related feature ID
    pub feature_id: Option<String>,

    /// Layer ID
    pub layer_id: Option<String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Resolved by user ID
    pub resolved_by: Option<String>,

    /// Resolved timestamp
    pub resolved_at: Option<DateTime<Utc>>,

    /// Custom metadata
    pub metadata: serde_json::Value,
}

impl Annotation {
    /// Create new annotation
    pub fn new(
        author_id: String,
        author_name: String,
        annotation_type: AnnotationType,
        geometry: AnnotationGeometry,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            annotation_type,
            author_id,
            author_name,
            geometry,
            content,
            status: AnnotationStatus::Open,
            tags: vec![],
            feature_id: None,
            layer_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            resolved_by: None,
            resolved_at: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Update content
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    /// Update status
    pub fn update_status(&mut self, status: AnnotationStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Resolve annotation
    pub fn resolve(&mut self, resolver_id: String) {
        self.status = AnnotationStatus::Resolved;
        self.resolved_by = Some(resolver_id);
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Add tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove tag
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = Utc::now();
    }

    /// Check if has tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Annotation manager
pub struct AnnotationManager {
    /// Annotations by ID
    annotations: Arc<DashMap<String, Annotation>>,
}

impl AnnotationManager {
    /// Create new annotation manager
    pub fn new() -> Self {
        Self {
            annotations: Arc::new(DashMap::new()),
        }
    }

    /// Add annotation
    pub fn add_annotation(&self, annotation: Annotation) -> String {
        let id = annotation.id.clone();
        self.annotations.insert(id.clone(), annotation);
        id
    }

    /// Get annotation
    pub fn get_annotation(&self, id: &str) -> Option<Annotation> {
        self.annotations.get(id).map(|a| a.clone())
    }

    /// Update annotation
    pub fn update_annotation(&self, annotation: Annotation) {
        self.annotations.insert(annotation.id.clone(), annotation);
    }

    /// Remove annotation
    pub fn remove_annotation(&self, id: &str) -> Option<Annotation> {
        self.annotations.remove(id).map(|(_, a)| a)
    }

    /// Get all annotations
    pub fn get_all_annotations(&self) -> Vec<Annotation> {
        self.annotations
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get annotations by author
    pub fn get_by_author(&self, author_id: &str) -> Vec<Annotation> {
        self.annotations
            .iter()
            .filter(|entry| entry.value().author_id == author_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get annotations by status
    pub fn get_by_status(&self, status: AnnotationStatus) -> Vec<Annotation> {
        self.annotations
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get annotations by type
    pub fn get_by_type(&self, annotation_type: AnnotationType) -> Vec<Annotation> {
        self.annotations
            .iter()
            .filter(|entry| entry.value().annotation_type == annotation_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get annotations by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<Annotation> {
        self.annotations
            .iter()
            .filter(|entry| entry.value().has_tag(tag))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get annotations for feature
    pub fn get_for_feature(&self, feature_id: &str) -> Vec<Annotation> {
        self.annotations
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .feature_id
                    .as_ref()
                    .map_or(false, |id| id == feature_id)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get annotations in geographic bounds
    pub fn get_in_bounds(
        &self,
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    ) -> Vec<Annotation> {
        self.annotations
            .iter()
            .filter(|entry| {
                match &entry.value().geometry {
                    AnnotationGeometry::Point { lon, lat } => {
                        *lon >= min_lon && *lon <= max_lon && *lat >= min_lat && *lat <= max_lat
                    }
                    AnnotationGeometry::BoundingBox { bbox } => {
                        // Check if bboxes overlap
                        bbox[0] <= max_lon
                            && bbox[2] >= min_lon
                            && bbox[1] <= max_lat
                            && bbox[3] >= min_lat
                    }
                    AnnotationGeometry::LineString { coordinates } => {
                        // Check if any point is in bounds
                        coordinates.iter().any(|[lon, lat]| {
                            *lon >= min_lon
                                && *lon <= max_lon
                                && *lat >= min_lat
                                && *lat <= max_lat
                        })
                    }
                    AnnotationGeometry::Polygon { exterior, .. } => {
                        // Check if any exterior point is in bounds
                        exterior.iter().any(|[lon, lat]| {
                            *lon >= min_lon
                                && *lon <= max_lon
                                && *lat >= min_lat
                                && *lat <= max_lat
                        })
                    }
                }
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get annotation count
    pub fn count(&self) -> usize {
        self.annotations.len()
    }

    /// Clear all annotations
    pub fn clear(&self) {
        self.annotations.clear();
    }
}

impl Default for AnnotationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_creation() {
        let annotation = Annotation::new(
            "user1".to_string(),
            "Alice".to_string(),
            AnnotationType::Comment,
            AnnotationGeometry::point(-122.4194, 37.7749),
            "This is a comment".to_string(),
        );

        assert_eq!(annotation.author_id, "user1");
        assert_eq!(annotation.content, "This is a comment");
        assert_eq!(annotation.status, AnnotationStatus::Open);
    }

    #[test]
    fn test_annotation_tags() {
        let mut annotation = Annotation::new(
            "user1".to_string(),
            "Alice".to_string(),
            AnnotationType::Issue,
            AnnotationGeometry::point(-122.4194, 37.7749),
            "Issue found".to_string(),
        );

        annotation.add_tag("urgent".to_string());
        annotation.add_tag("bug".to_string());

        assert_eq!(annotation.tags.len(), 2);
        assert!(annotation.has_tag("urgent"));
        assert!(annotation.has_tag("bug"));

        annotation.remove_tag("urgent");
        assert_eq!(annotation.tags.len(), 1);
        assert!(!annotation.has_tag("urgent"));
    }

    #[test]
    fn test_annotation_manager() {
        let manager = AnnotationManager::new();

        let annotation = Annotation::new(
            "user1".to_string(),
            "Alice".to_string(),
            AnnotationType::Comment,
            AnnotationGeometry::point(-122.4194, 37.7749),
            "Comment".to_string(),
        );

        let id = manager.add_annotation(annotation.clone());
        assert_eq!(manager.count(), 1);

        let retrieved = manager.get_annotation(&id).unwrap();
        assert_eq!(retrieved.author_id, "user1");

        manager.remove_annotation(&id);
        assert_eq!(manager.count(), 0);
    }
}
