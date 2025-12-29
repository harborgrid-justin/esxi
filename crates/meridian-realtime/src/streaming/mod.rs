//! Data streaming for GPS, sensors, and event sourcing

pub mod gps;
pub mod sensor;
pub mod event;

pub use gps::{GpsStream, GpsUpdate, GpsQuality};
pub use sensor::{SensorStream, SensorReading, SensorType};
pub use event::{EventStream, Event, EventStore};

use tokio::sync::broadcast;
use crate::error::Result;

/// Stream trait for real-time data
#[async_trait::async_trait]
pub trait Stream: Send + Sync {
    /// Stream item type
    type Item: Clone + Send + Sync;

    /// Subscribe to stream
    fn subscribe(&self) -> broadcast::Receiver<Self::Item>;

    /// Publish to stream
    async fn publish(&self, item: Self::Item) -> Result<()>;

    /// Get subscriber count
    fn subscriber_count(&self) -> usize;
}

/// Stream statistics
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Total messages published
    pub messages_published: u64,

    /// Total messages received
    pub messages_received: u64,

    /// Total bytes transferred
    pub bytes_transferred: u64,

    /// Current subscriber count
    pub subscriber_count: usize,

    /// Messages per second (average)
    pub messages_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_stats() {
        let stats = StreamStats::default();
        assert_eq!(stats.messages_published, 0);
        assert_eq!(stats.subscriber_count, 0);
    }
}
