//! GPS/location streaming for real-time tracking

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::error::{Error, Result};
use crate::streaming::Stream;

/// GPS quality indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GpsQuality {
    /// No fix
    NoFix,

    /// GPS fix (SPS)
    GpsFix,

    /// Differential GPS fix (DGPS)
    DgpsFix,

    /// RTK fixed
    RtkFixed,

    /// RTK float
    RtkFloat,

    /// Estimated/Dead reckoning
    Estimated,
}

/// GPS update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsUpdate {
    /// Tracker ID (e.g., vehicle ID, device ID)
    pub tracker_id: String,

    /// Longitude
    pub lon: f64,

    /// Latitude
    pub lat: f64,

    /// Altitude (meters)
    pub altitude: Option<f64>,

    /// Speed (m/s)
    pub speed: Option<f64>,

    /// Heading/bearing (degrees, 0-360)
    pub heading: Option<f64>,

    /// Horizontal dilution of precision
    pub hdop: Option<f64>,

    /// Number of satellites
    pub satellites: Option<u8>,

    /// GPS quality
    pub quality: GpsQuality,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Custom metadata
    pub metadata: serde_json::Value,
}

impl GpsUpdate {
    /// Create new GPS update
    pub fn new(tracker_id: String, lon: f64, lat: f64) -> Self {
        Self {
            tracker_id,
            lon,
            lat,
            altitude: None,
            speed: None,
            heading: None,
            hdop: None,
            satellites: None,
            quality: GpsQuality::GpsFix,
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    /// With altitude
    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = Some(altitude);
        self
    }

    /// With speed
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }

    /// With heading
    pub fn with_heading(mut self, heading: f64) -> Self {
        self.heading = Some(heading);
        self
    }

    /// With GPS quality
    pub fn with_quality(mut self, quality: GpsQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Check if position is valid
    pub fn is_valid(&self) -> bool {
        self.lon >= -180.0
            && self.lon <= 180.0
            && self.lat >= -90.0
            && self.lat <= 90.0
            && self.quality != GpsQuality::NoFix
    }

    /// Calculate distance to another position (Haversine formula)
    pub fn distance_to(&self, other: &GpsUpdate) -> f64 {
        const EARTH_RADIUS_M: f64 = 6_371_000.0;

        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();
        let delta_lat = (other.lat - self.lat).to_radians();
        let delta_lon = (other.lon - self.lon).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_M * c
    }
}

/// GPS stream for real-time location updates
pub struct GpsStream {
    /// Broadcast channel
    tx: broadcast::Sender<GpsUpdate>,

    /// Message counter
    message_count: Arc<AtomicU64>,
}

impl GpsStream {
    /// Create new GPS stream
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);

        Self {
            tx,
            message_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get message count
    pub fn message_count(&self) -> u64 {
        self.message_count.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl Stream for GpsStream {
    type Item = GpsUpdate;

    fn subscribe(&self) -> broadcast::Receiver<Self::Item> {
        self.tx.subscribe()
    }

    async fn publish(&self, item: Self::Item) -> Result<()> {
        if !item.is_valid() {
            return Err(Error::InvalidMessage("Invalid GPS coordinates".to_string()));
        }

        self.tx
            .send(item)
            .map_err(|e| Error::Internal(format!("Failed to publish GPS update: {}", e)))?;

        self.message_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// GPS tracker for maintaining latest position
pub struct GpsTracker {
    /// Latest positions by tracker ID
    positions: Arc<dashmap::DashMap<String, GpsUpdate>>,

    /// Stream
    stream: Arc<GpsStream>,
}

impl GpsTracker {
    /// Create new GPS tracker
    pub fn new(stream_capacity: usize) -> Self {
        Self {
            positions: Arc::new(dashmap::DashMap::new()),
            stream: Arc::new(GpsStream::new(stream_capacity)),
        }
    }

    /// Update position
    pub async fn update_position(&self, update: GpsUpdate) -> Result<()> {
        let tracker_id = update.tracker_id.clone();

        // Publish to stream
        self.stream.publish(update.clone()).await?;

        // Update latest position
        self.positions.insert(tracker_id, update);

        Ok(())
    }

    /// Get latest position
    pub fn get_position(&self, tracker_id: &str) -> Option<GpsUpdate> {
        self.positions.get(tracker_id).map(|p| p.clone())
    }

    /// Get all positions
    pub fn get_all_positions(&self) -> Vec<GpsUpdate> {
        self.positions
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get positions in bounds
    pub fn get_positions_in_bounds(
        &self,
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    ) -> Vec<GpsUpdate> {
        self.positions
            .iter()
            .filter(|entry| {
                let pos = entry.value();
                pos.lon >= min_lon
                    && pos.lon <= max_lon
                    && pos.lat >= min_lat
                    && pos.lat <= max_lat
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove tracker
    pub fn remove_tracker(&self, tracker_id: &str) {
        self.positions.remove(tracker_id);
    }

    /// Get stream
    pub fn stream(&self) -> &GpsStream {
        &self.stream
    }

    /// Get tracker count
    pub fn tracker_count(&self) -> usize {
        self.positions.len()
    }

    /// Clear old positions
    pub fn clear_old_positions(&self, age_secs: i64) {
        let now = Utc::now();
        let old_trackers: Vec<String> = self
            .positions
            .iter()
            .filter(|entry| {
                (now - entry.value().timestamp).num_seconds() > age_secs
            })
            .map(|entry| entry.key().clone())
            .collect();

        for tracker_id in old_trackers {
            self.positions.remove(&tracker_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_update() {
        let update = GpsUpdate::new("tracker1".to_string(), -122.4194, 37.7749)
            .with_altitude(100.0)
            .with_speed(25.0)
            .with_heading(180.0);

        assert_eq!(update.tracker_id, "tracker1");
        assert_eq!(update.altitude, Some(100.0));
        assert_eq!(update.speed, Some(25.0));
        assert!(update.is_valid());
    }

    #[test]
    fn test_gps_distance() {
        let pos1 = GpsUpdate::new("t1".to_string(), -122.4194, 37.7749);
        let pos2 = GpsUpdate::new("t2".to_string(), -122.4184, 37.7759);

        let distance = pos1.distance_to(&pos2);
        assert!(distance > 0.0 && distance < 200.0); // Should be ~150 meters
    }

    #[tokio::test]
    async fn test_gps_stream() {
        let stream = GpsStream::new(100);
        let mut rx = stream.subscribe();

        let update = GpsUpdate::new("tracker1".to_string(), -122.4194, 37.7749);
        stream.publish(update.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.tracker_id, "tracker1");
        assert_eq!(stream.message_count(), 1);
    }

    #[tokio::test]
    async fn test_gps_tracker() {
        let tracker = GpsTracker::new(100);

        let update = GpsUpdate::new("tracker1".to_string(), -122.4194, 37.7749);
        tracker.update_position(update.clone()).await.unwrap();

        assert_eq!(tracker.tracker_count(), 1);

        let pos = tracker.get_position("tracker1").unwrap();
        assert_eq!(pos.tracker_id, "tracker1");
    }
}
