//! Spatial join transforms.

use crate::error::Result;
use crate::transforms::Transform;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;

/// Spatial join predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialJoinPredicate {
    /// Geometries intersect.
    Intersects,
    /// Left geometry contains right.
    Contains,
    /// Left geometry is within right.
    Within,
    /// Geometries touch.
    Touches,
    /// Geometries cross.
    Crosses,
    /// Geometries overlap.
    Overlaps,
    /// Within distance.
    WithinDistance,
}

/// Join type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    /// Inner join.
    Inner,
    /// Left join.
    Left,
    /// Right join.
    Right,
}

/// Spatial join transform.
pub struct SpatialJoinTransform {
    predicate: SpatialJoinPredicate,
    join_type: JoinType,
    left_geometry_column: String,
    right_geometry_column: String,
    distance: Option<f64>,
}

impl SpatialJoinTransform {
    /// Create a new spatial join.
    pub fn new(predicate: SpatialJoinPredicate) -> Self {
        Self {
            predicate,
            join_type: JoinType::Inner,
            left_geometry_column: "geometry".to_string(),
            right_geometry_column: "geometry".to_string(),
            distance: None,
        }
    }

    /// Create intersects join.
    pub fn intersects() -> Self {
        Self::new(SpatialJoinPredicate::Intersects)
    }

    /// Create contains join.
    pub fn contains() -> Self {
        Self::new(SpatialJoinPredicate::Contains)
    }

    /// Create within join.
    pub fn within() -> Self {
        Self::new(SpatialJoinPredicate::Within)
    }

    /// Create distance join.
    pub fn within_distance(distance: f64) -> Self {
        Self {
            predicate: SpatialJoinPredicate::WithinDistance,
            distance: Some(distance),
            ..Self::new(SpatialJoinPredicate::WithinDistance)
        }
    }

    /// Set join type.
    pub fn with_join_type(mut self, join_type: JoinType) -> Self {
        self.join_type = join_type;
        self
    }

    /// Set left geometry column.
    pub fn with_left_geometry(mut self, column: impl Into<String>) -> Self {
        self.left_geometry_column = column.into();
        self
    }

    /// Set right geometry column.
    pub fn with_right_geometry(mut self, column: impl Into<String>) -> Self {
        self.right_geometry_column = column.into();
        self
    }
}

#[async_trait]
impl Transform for SpatialJoinTransform {
    async fn transform(&self, batch: RecordBatch) -> Result<RecordBatch> {
        tracing::debug!(
            predicate = ?self.predicate,
            join_type = ?self.join_type,
            "Applying spatial join transformation"
        );

        // In a real implementation, this would:
        // 1. Build spatial index for efficient lookup
        // 2. For each geometry in left batch, find matching geometries in right
        // 3. Apply spatial predicate
        // 4. Join the records based on join type
        // 5. Create new batch with joined results

        Ok(batch)
    }

    fn name(&self) -> &str {
        "spatial_join"
    }

    fn description(&self) -> Option<&str> {
        Some("Join records based on spatial relationship")
    }

    fn modifies_schema(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_join_creation() {
        let join = SpatialJoinTransform::intersects();
        assert_eq!(join.predicate, SpatialJoinPredicate::Intersects);
    }

    #[test]
    fn test_spatial_join_distance() {
        let join = SpatialJoinTransform::within_distance(100.0);
        assert_eq!(join.predicate, SpatialJoinPredicate::WithinDistance);
        assert_eq!(join.distance, Some(100.0));
    }
}
