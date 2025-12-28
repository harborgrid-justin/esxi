//! Viewport tracking for spatial subscriptions and dynamic updates.

use crate::channel::ChannelManager;
use crate::error::Result;
use crate::messages::{ClientId, LayerId, StreamMessage, ViewportUpdateMessage};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Spatial bounds represented as [min_lon, min_lat, max_lon, max_lat].
pub type Bounds = [f64; 4];

/// Spatial point represented as [lon, lat].
pub type Point = [f64; 2];

/// Viewport information for a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// Client identifier
    pub client_id: ClientId,
    /// Viewport bounds
    pub bounds: Bounds,
    /// Zoom level
    pub zoom: f64,
    /// Center point
    pub center: Point,
    /// Active layer subscriptions
    pub layers: Vec<LayerId>,
    /// Last update timestamp
    pub last_update: u64,
}

impl Viewport {
    /// Create a new viewport.
    pub fn new(client_id: ClientId, bounds: Bounds, zoom: f64) -> Self {
        let center = [
            (bounds[0] + bounds[2]) / 2.0,
            (bounds[1] + bounds[3]) / 2.0,
        ];
        Self {
            client_id,
            bounds,
            zoom,
            center,
            layers: Vec::new(),
            last_update: crate::messages::current_timestamp(),
        }
    }

    /// Update viewport bounds.
    pub fn update_bounds(&mut self, bounds: Bounds, zoom: f64) {
        self.bounds = bounds;
        self.zoom = zoom;
        self.center = [
            (bounds[0] + bounds[2]) / 2.0,
            (bounds[1] + bounds[3]) / 2.0,
        ];
        self.last_update = crate::messages::current_timestamp();
    }

    /// Check if a point is within the viewport.
    pub fn contains_point(&self, point: Point) -> bool {
        point[0] >= self.bounds[0]
            && point[0] <= self.bounds[2]
            && point[1] >= self.bounds[1]
            && point[1] <= self.bounds[3]
    }

    /// Check if bounds overlap with the viewport.
    pub fn overlaps(&self, bounds: Bounds) -> bool {
        !(bounds[2] < self.bounds[0]
            || bounds[0] > self.bounds[2]
            || bounds[3] < self.bounds[1]
            || bounds[1] > self.bounds[3])
    }

    /// Calculate area of the viewport.
    pub fn area(&self) -> f64 {
        let width = self.bounds[2] - self.bounds[0];
        let height = self.bounds[3] - self.bounds[1];
        width * height
    }

    /// Expand viewport by a factor.
    pub fn expand(&self, factor: f64) -> Bounds {
        let width = (self.bounds[2] - self.bounds[0]) * factor;
        let height = (self.bounds[3] - self.bounds[1]) * factor;
        [
            self.center[0] - width / 2.0,
            self.center[1] - height / 2.0,
            self.center[0] + width / 2.0,
            self.center[1] + height / 2.0,
        ]
    }
}

/// Spatial index for efficient viewport queries.
struct SpatialIndex {
    /// Grid size for spatial partitioning
    grid_size: f64,
    /// Features indexed by grid cell
    cells: DashMap<(i32, i32), Vec<String>>,
}

impl SpatialIndex {
    /// Create a new spatial index.
    fn new(grid_size: f64) -> Self {
        Self {
            grid_size,
            cells: DashMap::new(),
        }
    }

    /// Get grid cell for a point.
    fn get_cell(&self, point: Point) -> (i32, i32) {
        (
            (point[0] / self.grid_size).floor() as i32,
            (point[1] / self.grid_size).floor() as i32,
        )
    }

    /// Get all cells that overlap with bounds.
    fn get_cells_for_bounds(&self, bounds: Bounds) -> Vec<(i32, i32)> {
        let min_cell = self.get_cell([bounds[0], bounds[1]]);
        let max_cell = self.get_cell([bounds[2], bounds[3]]);

        let mut cells = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                cells.push((x, y));
            }
        }
        cells
    }

    /// Insert a feature at a point.
    fn insert(&self, feature_id: String, point: Point) {
        let cell = self.get_cell(point);
        self.cells
            .entry(cell)
            .or_insert_with(Vec::new)
            .push(feature_id);
    }

    /// Query features within bounds.
    fn query(&self, bounds: Bounds) -> Vec<String> {
        let cells = self.get_cells_for_bounds(bounds);
        let mut results = Vec::new();

        for cell in cells {
            if let Some(features) = self.cells.get(&cell) {
                results.extend(features.clone());
            }
        }

        results
    }

    /// Remove a feature.
    fn remove(&self, feature_id: &str, point: Point) {
        let cell = self.get_cell(point);
        if let Some(mut features) = self.cells.get_mut(&cell) {
            features.retain(|f| f != feature_id);
        }
    }
}

/// Viewport manager for tracking client viewports and spatial subscriptions.
#[derive(Clone)]
pub struct ViewportManager {
    /// Active viewports by client
    viewports: Arc<DashMap<ClientId, Viewport>>,
    /// Spatial index for features
    spatial_index: Arc<SpatialIndex>,
    /// Channel manager for subscriptions
    channels: Arc<ChannelManager>,
}

impl ViewportManager {
    /// Create a new viewport manager.
    pub fn new(channels: Arc<ChannelManager>) -> Self {
        Self::with_grid_size(channels, 1.0) // 1 degree grid by default
    }

    /// Create a new viewport manager with custom grid size.
    pub fn with_grid_size(channels: Arc<ChannelManager>, grid_size: f64) -> Self {
        Self {
            viewports: Arc::new(DashMap::new()),
            spatial_index: Arc::new(SpatialIndex::new(grid_size)),
            channels,
        }
    }

    /// Update client viewport.
    pub fn update_viewport(&self, message: ViewportUpdateMessage) -> Result<()> {
        let client_id = message.client_id;
        let bounds = message.bounds;
        let zoom = message.zoom;

        // Get or create viewport
        let mut viewport = self
            .viewports
            .entry(client_id)
            .or_insert_with(|| Viewport::new(client_id, bounds, zoom));

        // Store old bounds for comparison
        let old_bounds = viewport.bounds;

        // Update viewport
        viewport.update_bounds(bounds, zoom);

        // Update layer subscriptions if provided
        if let Some(layers) = message.layers {
            viewport.layers = layers;
        }

        debug!(
            "Updated viewport for client {}: zoom={}, area={}",
            client_id,
            viewport.zoom,
            viewport.area()
        );

        // If viewport changed significantly, update subscriptions
        if !bounds_similar(old_bounds, bounds) {
            self.update_spatial_subscriptions(client_id, &viewport)?;
        }

        Ok(())
    }

    /// Update spatial subscriptions based on viewport.
    fn update_spatial_subscriptions(&self, client_id: ClientId, viewport: &Viewport) -> Result<()> {
        // Create viewport-specific channel
        let channel_id = format!("viewport:{}", client_id);

        // Unsubscribe from old viewport channel if exists
        let _ = self.channels.unsubscribe(client_id, &channel_id);

        // Subscribe to new viewport channel
        self.channels.create_channel(channel_id.clone())?;
        let _ = self.channels.subscribe(client_id, channel_id);

        info!(
            "Updated spatial subscriptions for client {} (zoom: {}, layers: {})",
            client_id,
            viewport.zoom,
            viewport.layers.len()
        );

        Ok(())
    }

    /// Get viewport for a client.
    pub fn get_viewport(&self, client_id: &ClientId) -> Option<Viewport> {
        self.viewports.get(client_id).map(|v| v.clone())
    }

    /// Remove viewport for a client.
    pub fn remove_viewport(&self, client_id: &ClientId) -> Result<()> {
        self.viewports.remove(client_id);
        let channel_id = format!("viewport:{}", client_id);
        let _ = self.channels.delete_channel(&channel_id);
        Ok(())
    }

    /// Find all clients viewing a specific point.
    pub fn find_viewers_at_point(&self, point: Point) -> Vec<ClientId> {
        self.viewports
            .iter()
            .filter(|entry| entry.value().contains_point(point))
            .map(|entry| *entry.key())
            .collect()
    }

    /// Find all clients whose viewport overlaps with bounds.
    pub fn find_viewers_in_bounds(&self, bounds: Bounds) -> Vec<ClientId> {
        self.viewports
            .iter()
            .filter(|entry| entry.value().overlaps(bounds))
            .map(|entry| *entry.key())
            .collect()
    }

    /// Find all clients viewing a specific layer.
    pub fn find_viewers_of_layer(&self, layer_id: &LayerId) -> Vec<ClientId> {
        self.viewports
            .iter()
            .filter(|entry| entry.value().layers.contains(layer_id))
            .map(|entry| *entry.key())
            .collect()
    }

    /// Broadcast message to all viewers of a point.
    pub fn broadcast_to_point(&self, point: Point, message: StreamMessage) -> Result<usize> {
        let viewers = self.find_viewers_at_point(point);
        let mut count = 0;

        for client_id in viewers {
            let channel_id = format!("viewport:{}", client_id);
            if self.channels.publish(&channel_id, message.clone()).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Broadcast message to all viewers in bounds.
    pub fn broadcast_to_bounds(&self, bounds: Bounds, message: StreamMessage) -> Result<usize> {
        let viewers = self.find_viewers_in_bounds(bounds);
        let mut count = 0;

        for client_id in viewers {
            let channel_id = format!("viewport:{}", client_id);
            if self.channels.publish(&channel_id, message.clone()).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Broadcast message to all viewers of a layer.
    pub fn broadcast_to_layer(&self, layer_id: &LayerId, message: StreamMessage) -> Result<usize> {
        let viewers = self.find_viewers_of_layer(layer_id);
        let mut count = 0;

        for client_id in viewers {
            let channel_id = format!("viewport:{}", client_id);
            if self.channels.publish(&channel_id, message.clone()).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Index a feature at a specific location.
    pub fn index_feature(&self, feature_id: String, point: Point) {
        self.spatial_index.insert(feature_id, point);
    }

    /// Remove a feature from the index.
    pub fn remove_feature(&self, feature_id: &str, point: Point) {
        self.spatial_index.remove(feature_id, point);
    }

    /// Query features within bounds.
    pub fn query_features(&self, bounds: Bounds) -> Vec<String> {
        self.spatial_index.query(bounds)
    }

    /// Get the number of active viewports.
    pub fn viewport_count(&self) -> usize {
        self.viewports.len()
    }

    /// Get statistics about viewports.
    pub fn get_stats(&self) -> ViewportStats {
        let mut total_area = 0.0;
        let mut min_zoom = f64::MAX;
        let mut max_zoom = f64::MIN;
        let count = self.viewports.len();

        for entry in self.viewports.iter() {
            let viewport = entry.value();
            total_area += viewport.area();
            min_zoom = min_zoom.min(viewport.zoom);
            max_zoom = max_zoom.max(viewport.zoom);
        }

        ViewportStats {
            count,
            total_area,
            avg_area: if count > 0 { total_area / count as f64 } else { 0.0 },
            min_zoom: if count > 0 { min_zoom } else { 0.0 },
            max_zoom: if count > 0 { max_zoom } else { 0.0 },
        }
    }
}

/// Viewport statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportStats {
    /// Number of active viewports
    pub count: usize,
    /// Total area covered
    pub total_area: f64,
    /// Average viewport area
    pub avg_area: f64,
    /// Minimum zoom level
    pub min_zoom: f64,
    /// Maximum zoom level
    pub max_zoom: f64,
}

/// Check if two bounds are similar (within a tolerance).
fn bounds_similar(a: Bounds, b: Bounds) -> bool {
    const TOLERANCE: f64 = 0.0001;
    (a[0] - b[0]).abs() < TOLERANCE
        && (a[1] - b[1]).abs() < TOLERANCE
        && (a[2] - b[2]).abs() < TOLERANCE
        && (a[3] - b[3]).abs() < TOLERANCE
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_viewport_creation() {
        let client_id = Uuid::new_v4();
        let bounds = [-180.0, -90.0, 180.0, 90.0];
        let viewport = Viewport::new(client_id, bounds, 1.0);

        assert_eq!(viewport.bounds, bounds);
        assert_eq!(viewport.zoom, 1.0);
        assert_eq!(viewport.center, [0.0, 0.0]);
    }

    #[test]
    fn test_viewport_contains_point() {
        let client_id = Uuid::new_v4();
        let bounds = [-10.0, -10.0, 10.0, 10.0];
        let viewport = Viewport::new(client_id, bounds, 1.0);

        assert!(viewport.contains_point([0.0, 0.0]));
        assert!(viewport.contains_point([5.0, 5.0]));
        assert!(!viewport.contains_point([15.0, 15.0]));
    }

    #[test]
    fn test_viewport_overlaps() {
        let client_id = Uuid::new_v4();
        let bounds = [-10.0, -10.0, 10.0, 10.0];
        let viewport = Viewport::new(client_id, bounds, 1.0);

        assert!(viewport.overlaps([-5.0, -5.0, 5.0, 5.0])); // Contained
        assert!(viewport.overlaps([-15.0, -15.0, -5.0, -5.0])); // Partial overlap
        assert!(!viewport.overlaps([15.0, 15.0, 20.0, 20.0])); // No overlap
    }

    #[test]
    fn test_viewport_manager() {
        let channels = Arc::new(ChannelManager::new());
        let manager = ViewportManager::new(channels);

        let client_id = Uuid::new_v4();
        let message = ViewportUpdateMessage {
            client_id,
            bounds: [-10.0, -10.0, 10.0, 10.0],
            zoom: 5.0,
            center: [0.0, 0.0],
            timestamp: crate::messages::current_timestamp(),
            layers: Some(vec!["layer1".to_string()]),
        };

        manager.update_viewport(message).unwrap();

        let viewport = manager.get_viewport(&client_id).unwrap();
        assert_eq!(viewport.zoom, 5.0);
        assert_eq!(viewport.layers.len(), 1);
    }

    #[test]
    fn test_find_viewers() {
        let channels = Arc::new(ChannelManager::new());
        let manager = ViewportManager::new(channels);

        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();

        manager.update_viewport(ViewportUpdateMessage {
            client_id: client1,
            bounds: [-10.0, -10.0, 10.0, 10.0],
            zoom: 5.0,
            center: [0.0, 0.0],
            timestamp: crate::messages::current_timestamp(),
            layers: None,
        }).unwrap();

        manager.update_viewport(ViewportUpdateMessage {
            client_id: client2,
            bounds: [20.0, 20.0, 30.0, 30.0],
            zoom: 5.0,
            center: [25.0, 25.0],
            timestamp: crate::messages::current_timestamp(),
            layers: None,
        }).unwrap();

        let viewers = manager.find_viewers_at_point([0.0, 0.0]);
        assert_eq!(viewers.len(), 1);
        assert!(viewers.contains(&client1));

        let viewers = manager.find_viewers_at_point([25.0, 25.0]);
        assert_eq!(viewers.len(), 1);
        assert!(viewers.contains(&client2));
    }
}
