//! Spatial query builders with PostGIS support

use crate::models::BBox;

/// Spatial query builder for PostGIS operations
#[derive(Debug, Clone)]
pub struct SpatialQuery {
    /// Table name
    table: String,
    /// Geometry column name
    geom_column: String,
    /// WHERE clauses
    conditions: Vec<String>,
    /// SRID for transformations
    srid: Option<i32>,
    /// Use spatial index hint
    use_index: bool,
}

impl SpatialQuery {
    /// Create a new spatial query
    pub fn new(table: impl Into<String>, geom_column: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            geom_column: geom_column.into(),
            conditions: Vec::new(),
            srid: None,
            use_index: true,
        }
    }

    /// Set SRID for coordinate transformations
    pub fn with_srid(mut self, srid: i32) -> Self {
        self.srid = Some(srid);
        self
    }

    /// Disable spatial index usage
    pub fn without_index(mut self) -> Self {
        self.use_index = false;
        self
    }

    /// Add a WHERE condition
    pub fn add_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    /// Build ST_Within query
    pub fn st_within(&self, bbox: &BBox) -> String {
        let geom = self.transform_geometry(&bbox.to_wkt(), bbox.srid);
        let condition = format!(
            "ST_Within({}, {})",
            self.geom_column, geom
        );
        self.build_query(&condition)
    }

    /// Build ST_Intersects query
    pub fn st_intersects(&self, wkt: &str, srid: i32) -> String {
        let geom = self.transform_geometry(wkt, srid);
        let condition = format!(
            "ST_Intersects({}, {})",
            self.geom_column, geom
        );
        self.build_query(&condition)
    }

    /// Build ST_Contains query
    pub fn st_contains(&self, wkt: &str, srid: i32) -> String {
        let geom = self.transform_geometry(wkt, srid);
        let condition = format!(
            "ST_Contains({}, {})",
            self.geom_column, geom
        );
        self.build_query(&condition)
    }

    /// Build ST_DWithin query (distance query)
    pub fn st_dwithin(&self, wkt: &str, srid: i32, distance: f64) -> String {
        let geom = self.transform_geometry(wkt, srid);
        let condition = format!(
            "ST_DWithin({}, {}, {})",
            self.geom_column, geom, distance
        );
        self.build_query(&condition)
    }

    /// Build ST_Distance query
    pub fn st_distance(&self, wkt: &str, srid: i32) -> String {
        let geom = self.transform_geometry(wkt, srid);
        format!(
            "SELECT *, ST_Distance({}, {}) as distance FROM {} WHERE TRUE {}",
            self.geom_column,
            geom,
            self.table,
            self.build_conditions()
        )
    }

    /// Build ST_Buffer query
    pub fn st_buffer(&self, distance: f64) -> String {
        format!(
            "SELECT *, ST_Buffer({}, {}) as buffered_geom FROM {} WHERE TRUE {}",
            self.geom_column,
            distance,
            self.table,
            self.build_conditions()
        )
    }

    /// Transform geometry to target SRID
    fn transform_geometry(&self, wkt: &str, source_srid: i32) -> String {
        let geom = format!("ST_GeomFromText('{}', {})", wkt, source_srid);

        if let Some(target_srid) = self.srid {
            if target_srid != source_srid {
                return format!("ST_Transform({}, {})", geom, target_srid);
            }
        }

        geom
    }

    /// Build complete query with conditions
    fn build_query(&self, spatial_condition: &str) -> String {
        let index_hint = if self.use_index {
            format!(" AND {0} && ST_MakeEnvelope(ST_XMin({0}), ST_YMin({0}), ST_XMax({0}), ST_YMax({0}))", self.geom_column)
        } else {
            String::new()
        };

        format!(
            "SELECT * FROM {} WHERE {} {} {}",
            self.table,
            spatial_condition,
            index_hint,
            self.build_conditions()
        )
    }

    /// Build additional conditions
    fn build_conditions(&self) -> String {
        if self.conditions.is_empty() {
            String::new()
        } else {
            format!(" AND {}", self.conditions.join(" AND "))
        }
    }
}

/// Spatial aggregation queries
pub struct SpatialAggregation {
    table: String,
    geom_column: String,
}

impl SpatialAggregation {
    /// Create new aggregation builder
    pub fn new(table: impl Into<String>, geom_column: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            geom_column: geom_column.into(),
        }
    }

    /// Build ST_Union query
    pub fn st_union(&self) -> String {
        format!(
            "SELECT ST_Union({}) as union_geom FROM {}",
            self.geom_column, self.table
        )
    }

    /// Build ST_Collect query
    pub fn st_collect(&self) -> String {
        format!(
            "SELECT ST_Collect({}) as collection FROM {}",
            self.geom_column, self.table
        )
    }

    /// Build convex hull query
    pub fn st_convex_hull(&self) -> String {
        format!(
            "SELECT ST_ConvexHull(ST_Collect({})) as convex_hull FROM {}",
            self.geom_column, self.table
        )
    }

    /// Build extent query
    pub fn st_extent(&self) -> String {
        format!(
            "SELECT ST_Extent({}) as extent FROM {}",
            self.geom_column, self.table
        )
    }

    /// Build centroid query
    pub fn st_centroid(&self) -> String {
        format!(
            "SELECT ST_Centroid(ST_Collect({})) as centroid FROM {}",
            self.geom_column, self.table
        )
    }
}

/// Index optimization hints
pub struct IndexHint {
    /// Force index scan
    pub force_index: bool,
    /// Expected rows
    pub expected_rows: Option<usize>,
}

impl IndexHint {
    /// Create new index hint
    pub fn new() -> Self {
        Self {
            force_index: false,
            expected_rows: None,
        }
    }

    /// Force usage of spatial index
    pub fn force(mut self) -> Self {
        self.force_index = true;
        self
    }

    /// Set expected rows for query planner
    pub fn with_rows(mut self, rows: usize) -> Self {
        self.expected_rows = Some(rows);
        self
    }
}

impl Default for IndexHint {
    fn default() -> Self {
        Self::new()
    }
}

/// Spatial query optimizer
pub struct QueryOptimizer;

impl QueryOptimizer {
    /// Optimize spatial query with bounding box pre-filter
    pub fn optimize_bbox_query(table: &str, geom_column: &str, bbox: &BBox) -> String {
        format!(
            "SELECT * FROM {} WHERE {} && ST_MakeEnvelope({}, {}, {}, {}, {}) \
             AND ST_Intersects({}, ST_MakeEnvelope({}, {}, {}, {}, {}))",
            table,
            geom_column,
            bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y, bbox.srid,
            geom_column,
            bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y, bbox.srid
        )
    }

    /// Generate ANALYZE query for statistics update
    pub fn analyze_table(table: &str) -> String {
        format!("ANALYZE {}", table)
    }

    /// Generate VACUUM query for table maintenance
    pub fn vacuum_table(table: &str) -> String {
        format!("VACUUM ANALYZE {}", table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_query_builder() {
        let query = SpatialQuery::new("features", "geom")
            .with_srid(4326)
            .add_condition("layer_id = 'test'");

        let bbox = BBox::new(-180.0, -90.0, 180.0, 90.0, 4326);
        let sql = query.st_within(&bbox);

        assert!(sql.contains("ST_Within"));
        assert!(sql.contains("features"));
    }

    #[test]
    fn test_aggregation_queries() {
        let agg = SpatialAggregation::new("features", "geom");
        let union_query = agg.st_union();

        assert!(union_query.contains("ST_Union"));
        assert!(union_query.contains("features"));
    }

    #[test]
    fn test_query_optimizer() {
        let bbox = BBox::new(0.0, 0.0, 10.0, 10.0, 4326);
        let optimized = QueryOptimizer::optimize_bbox_query("features", "geom", &bbox);

        assert!(optimized.contains("&&")); // Bounding box operator
        assert!(optimized.contains("ST_Intersects"));
    }
}
