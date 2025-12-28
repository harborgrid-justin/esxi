//! Spatial indexing using R-trees for fast spatial queries.
//!
//! This module provides an R-tree based spatial index that enables efficient
//! spatial queries like nearest neighbor search, intersection queries, and
//! containment queries.
//!
//! # Examples
//!
//! ```ignore
//! use meridian_core::spatial_index::SpatialIndex;
//! use meridian_core::geometry::Point;
//! use meridian_core::crs::Crs;
//!
//! let mut index = SpatialIndex::new();
//!
//! // Insert points
//! index.insert(Point::new(0.0, 0.0, Crs::wgs84()));
//! index.insert(Point::new(10.0, 10.0, Crs::wgs84()));
//!
//! // Query nearest neighbor
//! let query_point = Point::new(1.0, 1.0, Crs::wgs84());
//! if let Some(nearest) = index.nearest_neighbor(&query_point) {
//!     println!("Found nearest: {:?}", nearest);
//! }
//! ```

use crate::bbox::BoundingBox;
use crate::geometry::Geometry;
use crate::traits::Bounded;
use rstar::{Envelope, RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A spatial index wrapper that can store any geometry with spatial extent.
///
/// This struct wraps geometries with an ID to enable retrieval after spatial queries.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexedGeometry {
    /// Unique identifier for the geometry
    pub id: u64,
    /// The geometry being indexed
    pub geometry: Geometry,
    /// Cached bounding box for performance
    #[serde(skip)]
    bbox: Option<BoundingBox>,
}

impl IndexedGeometry {
    /// Creates a new indexed geometry.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier
    /// * `geometry` - The geometry to index
    pub fn new(id: u64, geometry: Geometry) -> Self {
        let bbox = geometry.bounds();
        Self {
            id,
            geometry,
            bbox: Some(bbox),
        }
    }

    /// Returns the bounding box, computing it if not cached.
    fn get_bbox(&self) -> BoundingBox {
        self.bbox.unwrap_or_else(|| self.geometry.bounds())
    }
}

impl RTreeObject for IndexedGeometry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let bbox = self.get_bbox();
        AABB::from_corners([bbox.min_x, bbox.min_y], [bbox.max_x, bbox.max_y])
    }
}

impl fmt::Debug for IndexedGeometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexedGeometry")
            .field("id", &self.id)
            .field("bbox", &self.get_bbox())
            .finish()
    }
}

/// A spatial index for fast geometric queries.
///
/// Uses an R-tree data structure for efficient spatial operations including:
/// - Nearest neighbor search
/// - Intersection queries
/// - Containment queries
/// - Range queries
#[derive(Clone)]
pub struct SpatialIndex {
    /// The underlying R-tree
    tree: RTree<IndexedGeometry>,
    /// Counter for auto-generating IDs
    next_id: u64,
}

impl SpatialIndex {
    /// Creates a new empty spatial index.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use meridian_core::spatial_index::SpatialIndex;
    ///
    /// let index = SpatialIndex::new();
    /// ```
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
            next_id: 0,
        }
    }

    /// Creates a spatial index from a collection of geometries.
    ///
    /// IDs are automatically assigned starting from 0.
    ///
    /// # Arguments
    ///
    /// * `geometries` - Iterator of geometries to index
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use meridian_core::spatial_index::SpatialIndex;
    /// use meridian_core::geometry::{Point, Geometry};
    /// use meridian_core::crs::Crs;
    ///
    /// let points = vec![
    ///     Geometry::Point(Point::new(0.0, 0.0, Crs::wgs84())),
    ///     Geometry::Point(Point::new(10.0, 10.0, Crs::wgs84())),
    /// ];
    ///
    /// let index = SpatialIndex::from_geometries(points);
    /// ```
    pub fn from_geometries<I>(geometries: I) -> Self
    where
        I: IntoIterator<Item = Geometry>,
    {
        let indexed: Vec<_> = geometries
            .into_iter()
            .enumerate()
            .map(|(i, geom)| IndexedGeometry::new(i as u64, geom))
            .collect();

        let next_id = indexed.len() as u64;

        Self {
            tree: RTree::bulk_load(indexed),
            next_id,
        }
    }

    /// Inserts a geometry into the index with an auto-generated ID.
    ///
    /// # Arguments
    ///
    /// * `geometry` - The geometry to insert
    ///
    /// # Returns
    ///
    /// The ID assigned to the geometry
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut index = SpatialIndex::new();
    /// let id = index.insert(Geometry::Point(point));
    /// ```
    pub fn insert(&mut self, geometry: Geometry) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tree.insert(IndexedGeometry::new(id, geometry));
        id
    }

    /// Inserts a geometry with a specific ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to assign
    /// * `geometry` - The geometry to insert
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut index = SpatialIndex::new();
    /// index.insert_with_id(42, Geometry::Point(point));
    /// ```
    pub fn insert_with_id(&mut self, id: u64, geometry: Geometry) {
        self.tree.insert(IndexedGeometry::new(id, geometry));
        self.next_id = self.next_id.max(id + 1);
    }

    /// Removes a geometry by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the geometry to remove
    ///
    /// # Returns
    ///
    /// `true` if a geometry was removed, `false` if no geometry with that ID exists
    pub fn remove(&mut self, id: u64) -> bool {
        // Find and remove the geometry
        let indexed = self.tree.iter().find(|g| g.id == id).cloned();
        if let Some(indexed) = indexed {
            self.tree.remove(&indexed);
            true
        } else {
            false
        }
    }

    /// Returns the number of geometries in the index.
    pub fn len(&self) -> usize {
        self.tree.size()
    }

    /// Checks if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }

    /// Finds the nearest neighbor to a point.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point
    ///
    /// # Returns
    ///
    /// The nearest geometry, or None if the index is empty
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let query_point = [10.5, 20.5];
    /// if let Some(nearest) = index.nearest_neighbor_point(&query_point) {
    ///     println!("Found nearest geometry: ID {}", nearest.id);
    /// }
    /// ```
    pub fn nearest_neighbor_point(&self, point: &[f64; 2]) -> Option<&IndexedGeometry> {
        // Find the nearest geometry by checking all geometries
        // This is not optimal but works without PointDistance trait
        self.tree
            .iter()
            .min_by(|a, b| {
                let a_center = a.envelope().center();
                let b_center = b.envelope().center();
                let dist_a = ((a_center[0] - point[0]).powi(2) + (a_center[1] - point[1]).powi(2)).sqrt();
                let dist_b = ((b_center[0] - point[0]).powi(2) + (b_center[1] - point[1]).powi(2)).sqrt();
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Finds the k nearest neighbors to a point.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point
    /// * `k` - Number of neighbors to find
    ///
    /// # Returns
    ///
    /// Vector of the k nearest geometries (may be less than k if index has fewer items)
    pub fn k_nearest_neighbors(&self, point: &[f64; 2], k: usize) -> Vec<&IndexedGeometry> {
        // Sort all geometries by distance and take k
        let mut geometries: Vec<_> = self.tree
            .iter()
            .map(|geom| {
                let center = geom.envelope().center();
                let dist = ((center[0] - point[0]).powi(2) + (center[1] - point[1]).powi(2)).sqrt();
                (geom, dist)
            })
            .collect();

        geometries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        geometries.into_iter().take(k).map(|(geom, _)| geom).collect()
    }

    /// Queries all geometries that intersect with a bounding box.
    ///
    /// # Arguments
    ///
    /// * `bbox` - The query bounding box
    ///
    /// # Returns
    ///
    /// Iterator over geometries that intersect the bounding box
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use meridian_core::bbox::BoundingBox;
    ///
    /// let query_bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    /// for geometry in index.query_bbox(&query_bbox) {
    ///     println!("Found intersecting geometry: ID {}", geometry.id);
    /// }
    /// ```
    pub fn query_bbox(&self, bbox: &BoundingBox) -> impl Iterator<Item = &IndexedGeometry> {
        let aabb = AABB::from_corners([bbox.min_x, bbox.min_y], [bbox.max_x, bbox.max_y]);
        self.tree.locate_in_envelope_intersecting(&aabb)
    }

    /// Queries all geometries within a distance of a point.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point [x, y]
    /// * `distance` - The search radius
    ///
    /// # Returns
    ///
    /// Iterator over geometries within the specified distance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let query_point = [10.0, 20.0];
    /// let radius = 5.0;
    ///
    /// for geometry in index.query_within_distance(&query_point, radius) {
    ///     println!("Found geometry within radius: ID {}", geometry.id);
    /// }
    /// ```
    pub fn query_within_distance(
        &self,
        point: &[f64; 2],
        distance: f64,
    ) -> Vec<&IndexedGeometry> {
        // Filter geometries by distance
        self.tree
            .iter()
            .filter(|geom| {
                let center = geom.envelope().center();
                let dist = ((center[0] - point[0]).powi(2) + (center[1] - point[1]).powi(2)).sqrt();
                dist <= distance
            })
            .collect()
    }

    /// Gets a geometry by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the geometry to retrieve
    ///
    /// # Returns
    ///
    /// The geometry if found, None otherwise
    pub fn get(&self, id: u64) -> Option<&IndexedGeometry> {
        self.tree.iter().find(|g| g.id == id)
    }

    /// Returns an iterator over all geometries in the index.
    pub fn iter(&self) -> impl Iterator<Item = &IndexedGeometry> {
        self.tree.iter()
    }

    /// Clears all geometries from the index.
    pub fn clear(&mut self) {
        self.tree = RTree::new();
        self.next_id = 0;
    }

    /// Returns statistics about the index.
    pub fn stats(&self) -> IndexStats {
        let mut total_area = 0.0;
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for indexed in self.tree.iter() {
            let bbox = indexed.get_bbox();
            total_area += bbox.area();
            min_x = min_x.min(bbox.min_x);
            min_y = min_y.min(bbox.min_y);
            max_x = max_x.max(bbox.max_x);
            max_y = max_y.max(bbox.max_y);
        }

        let extent = if self.is_empty() {
            None
        } else {
            Some(BoundingBox::new(min_x, min_y, max_x, max_y))
        };

        IndexStats {
            count: self.len(),
            total_area,
            extent,
        }
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SpatialIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpatialIndex")
            .field("count", &self.len())
            .field("next_id", &self.next_id)
            .finish()
    }
}

/// Statistics about a spatial index.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexStats {
    /// Number of geometries in the index
    pub count: usize,
    /// Total area of all bounding boxes
    pub total_area: f64,
    /// Overall extent of all geometries
    pub extent: Option<BoundingBox>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::Crs;
    use crate::geometry::Point;

    #[test]
    fn test_spatial_index_insert() {
        let mut index = SpatialIndex::new();
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let id = index.insert(Geometry::Point(point));

        assert_eq!(index.len(), 1);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_spatial_index_remove() {
        let mut index = SpatialIndex::new();
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let id = index.insert(Geometry::Point(point));

        assert!(index.remove(id));
        assert_eq!(index.len(), 0);
        assert!(!index.remove(id));
    }

    #[test]
    fn test_nearest_neighbor() {
        let mut index = SpatialIndex::new();
        index.insert(Geometry::Point(Point::new(0.0, 0.0, Crs::wgs84())));
        index.insert(Geometry::Point(Point::new(10.0, 10.0, Crs::wgs84())));

        let nearest = index.nearest_neighbor_point(&[1.0, 1.0]).unwrap();
        assert_eq!(nearest.id, 0); // First point is closer
    }

    #[test]
    fn test_query_bbox() {
        let mut index = SpatialIndex::new();
        index.insert(Geometry::Point(Point::new(5.0, 5.0, Crs::wgs84())));
        index.insert(Geometry::Point(Point::new(15.0, 15.0, Crs::wgs84())));

        let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let results: Vec<_> = index.query_bbox(&bbox).collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 0);
    }

    #[test]
    fn test_k_nearest_neighbors() {
        let mut index = SpatialIndex::new();
        index.insert(Geometry::Point(Point::new(0.0, 0.0, Crs::wgs84())));
        index.insert(Geometry::Point(Point::new(5.0, 5.0, Crs::wgs84())));
        index.insert(Geometry::Point(Point::new(10.0, 10.0, Crs::wgs84())));

        let nearest = index.k_nearest_neighbors(&[0.0, 0.0], 2);
        assert_eq!(nearest.len(), 2);
        assert_eq!(nearest[0].id, 0);
        assert_eq!(nearest[1].id, 1);
    }

    #[test]
    fn test_index_stats() {
        let mut index = SpatialIndex::new();
        index.insert(Geometry::Point(Point::new(0.0, 0.0, Crs::wgs84())));
        index.insert(Geometry::Point(Point::new(10.0, 10.0, Crs::wgs84())));

        let stats = index.stats();
        assert_eq!(stats.count, 2);
        assert!(stats.extent.is_some());

        let extent = stats.extent.unwrap();
        assert_eq!(extent.min_x, 0.0);
        assert_eq!(extent.max_x, 10.0);
    }
}
