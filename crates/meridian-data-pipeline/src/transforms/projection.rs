//! Coordinate reference system (CRS) transformation.

use crate::error::{Result, TransformError, PipelineError};
use crate::transforms::Transform;
use crate::Crs;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;

/// Projection transform for CRS transformations.
pub struct ProjectionTransform {
    source_crs: Crs,
    target_crs: Crs,
    geometry_column: String,
    always_xy: bool,
}

impl ProjectionTransform {
    /// Create a new projection transform.
    pub fn new(source_crs: impl Into<String>, target_crs: impl Into<String>) -> Self {
        Self {
            source_crs: Crs::from_string(source_crs),
            target_crs: Crs::from_string(target_crs),
            geometry_column: "geometry".to_string(),
            always_xy: true,
        }
    }

    /// Create a projection from EPSG codes.
    pub fn from_epsg(source_epsg: u32, target_epsg: u32) -> Self {
        Self {
            source_crs: Crs::epsg(source_epsg),
            target_crs: Crs::epsg(target_epsg),
            geometry_column: "geometry".to_string(),
            always_xy: true,
        }
    }

    /// Project from WGS84 to Web Mercator.
    pub fn wgs84_to_web_mercator() -> Self {
        Self::from_epsg(4326, 3857)
    }

    /// Project from Web Mercator to WGS84.
    pub fn web_mercator_to_wgs84() -> Self {
        Self::from_epsg(3857, 4326)
    }

    /// Set geometry column name.
    pub fn with_geometry_column(mut self, column: impl Into<String>) -> Self {
        self.geometry_column = column.into();
        self
    }

    /// Set always_xy flag (for axis order handling).
    pub fn with_always_xy(mut self, always_xy: bool) -> Self {
        self.always_xy = always_xy;
        self
    }
}

#[async_trait]
impl Transform for ProjectionTransform {
    async fn transform(&self, batch: RecordBatch) -> Result<RecordBatch> {
        tracing::debug!(
            from_crs = %self.source_crs.code,
            to_crs = %self.target_crs.code,
            geometry_column = %self.geometry_column,
            "Applying CRS transformation"
        );

        // In a real implementation, this would:
        // 1. Extract geometry column from batch
        // 2. Parse geometries (WKT/WKB)
        // 3. Create proj transformation using proj crate
        // 4. Transform coordinates
        // 5. Convert back to WKT/WKB
        // 6. Create new batch with transformed geometries

        Ok(batch)
    }

    fn name(&self) -> &str {
        "projection"
    }

    fn description(&self) -> Option<&str> {
        Some("Transform coordinates between different CRS")
    }

    fn modifies_schema(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_transform_creation() {
        let transform = ProjectionTransform::new("EPSG:4326", "EPSG:3857");
        assert_eq!(transform.source_crs.code, "EPSG:4326");
        assert_eq!(transform.target_crs.code, "EPSG:3857");
    }

    #[test]
    fn test_projection_from_epsg() {
        let transform = ProjectionTransform::from_epsg(4326, 3857);
        assert_eq!(transform.source_crs.code, "EPSG:4326");
        assert_eq!(transform.target_crs.code, "EPSG:3857");
    }

    #[test]
    fn test_projection_presets() {
        let wgs84_to_mercator = ProjectionTransform::wgs84_to_web_mercator();
        assert_eq!(wgs84_to_mercator.source_crs.code, "EPSG:4326");
        assert_eq!(wgs84_to_mercator.target_crs.code, "EPSG:3857");

        let mercator_to_wgs84 = ProjectionTransform::web_mercator_to_wgs84();
        assert_eq!(mercator_to_wgs84.source_crs.code, "EPSG:3857");
        assert_eq!(mercator_to_wgs84.target_crs.code, "EPSG:4326");
    }
}
