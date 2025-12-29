//! IoT sensor data streaming

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::streaming::Stream;

/// Sensor type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    /// Temperature sensor (Celsius)
    Temperature,

    /// Humidity sensor (percentage)
    Humidity,

    /// Pressure sensor (hPa)
    Pressure,

    /// Air quality index
    AirQuality,

    /// Noise level (dB)
    Noise,

    /// Light level (lux)
    Light,

    /// Motion/Movement
    Motion,

    /// Proximity (distance in cm)
    Proximity,

    /// Water level (meters)
    WaterLevel,

    /// Soil moisture (percentage)
    SoilMoisture,

    /// Wind speed (m/s)
    WindSpeed,

    /// Wind direction (degrees)
    WindDirection,

    /// Rainfall (mm)
    Rainfall,

    /// CO2 level (ppm)
    Co2,

    /// Custom sensor type
    Custom(String),
}

/// Sensor reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    /// Sensor ID
    pub sensor_id: String,

    /// Sensor type
    pub sensor_type: SensorType,

    /// Sensor value
    pub value: f64,

    /// Unit of measurement
    pub unit: String,

    /// Location (lon, lat)
    pub location: Option<(f64, f64)>,

    /// Altitude
    pub altitude: Option<f64>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Quality/confidence (0.0-1.0)
    pub quality: Option<f64>,

    /// Custom metadata
    pub metadata: serde_json::Value,
}

impl SensorReading {
    /// Create new sensor reading
    pub fn new(sensor_id: String, sensor_type: SensorType, value: f64) -> Self {
        let unit = match sensor_type {
            SensorType::Temperature => "°C",
            SensorType::Humidity | SensorType::SoilMoisture => "%",
            SensorType::Pressure => "hPa",
            SensorType::Noise => "dB",
            SensorType::Light => "lux",
            SensorType::Proximity => "cm",
            SensorType::WaterLevel | SensorType::Rainfall => "mm",
            SensorType::WindSpeed => "m/s",
            SensorType::WindDirection => "°",
            SensorType::Co2 => "ppm",
            _ => "",
        }
        .to_string();

        Self {
            sensor_id,
            sensor_type,
            value,
            unit,
            location: None,
            altitude: None,
            timestamp: Utc::now(),
            quality: None,
            metadata: serde_json::json!({}),
        }
    }

    /// With location
    pub fn with_location(mut self, lon: f64, lat: f64) -> Self {
        self.location = Some((lon, lat));
        self
    }

    /// With altitude
    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = Some(altitude);
        self
    }

    /// With quality
    pub fn with_quality(mut self, quality: f64) -> Self {
        self.quality = Some(quality.clamp(0.0, 1.0));
        self
    }

    /// Check if reading is fresh
    pub fn is_fresh(&self, age_secs: i64) -> bool {
        (Utc::now() - self.timestamp).num_seconds() <= age_secs
    }
}

/// Sensor stream for real-time sensor data
pub struct SensorStream {
    /// Broadcast channel
    tx: broadcast::Sender<SensorReading>,

    /// Message counter
    message_count: Arc<AtomicU64>,
}

impl SensorStream {
    /// Create new sensor stream
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
impl Stream for SensorStream {
    type Item = SensorReading;

    fn subscribe(&self) -> broadcast::Receiver<Self::Item> {
        self.tx.subscribe()
    }

    async fn publish(&self, item: Self::Item) -> Result<()> {
        self.tx
            .send(item)
            .map_err(|e| crate::error::Error::Internal(format!("Failed to publish sensor reading: {}", e)))?;

        self.message_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// Sensor network for managing multiple sensors
pub struct SensorNetwork {
    /// Latest readings by sensor ID
    readings: Arc<dashmap::DashMap<String, SensorReading>>,

    /// Stream
    stream: Arc<SensorStream>,
}

impl SensorNetwork {
    /// Create new sensor network
    pub fn new(stream_capacity: usize) -> Self {
        Self {
            readings: Arc::new(dashmap::DashMap::new()),
            stream: Arc::new(SensorStream::new(stream_capacity)),
        }
    }

    /// Update sensor reading
    pub async fn update_reading(&self, reading: SensorReading) -> Result<()> {
        let sensor_id = reading.sensor_id.clone();

        // Publish to stream
        self.stream.publish(reading.clone()).await?;

        // Update latest reading
        self.readings.insert(sensor_id, reading);

        Ok(())
    }

    /// Get latest reading
    pub fn get_reading(&self, sensor_id: &str) -> Option<SensorReading> {
        self.readings.get(sensor_id).map(|r| r.clone())
    }

    /// Get all readings
    pub fn get_all_readings(&self) -> Vec<SensorReading> {
        self.readings
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get readings by sensor type
    pub fn get_readings_by_type(&self, sensor_type: SensorType) -> Vec<SensorReading> {
        self.readings
            .iter()
            .filter(|entry| entry.value().sensor_type == sensor_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get readings in bounds
    pub fn get_readings_in_bounds(
        &self,
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    ) -> Vec<SensorReading> {
        self.readings
            .iter()
            .filter(|entry| {
                if let Some((lon, lat)) = entry.value().location {
                    lon >= min_lon && lon <= max_lon && lat >= min_lat && lat <= max_lat
                } else {
                    false
                }
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove sensor
    pub fn remove_sensor(&self, sensor_id: &str) {
        self.readings.remove(sensor_id);
    }

    /// Get stream
    pub fn stream(&self) -> &SensorStream {
        &self.stream
    }

    /// Get sensor count
    pub fn sensor_count(&self) -> usize {
        self.readings.len()
    }

    /// Clear old readings
    pub fn clear_old_readings(&self, age_secs: i64) {
        let old_sensors: Vec<String> = self
            .readings
            .iter()
            .filter(|entry| !entry.value().is_fresh(age_secs))
            .map(|entry| entry.key().clone())
            .collect();

        for sensor_id in old_sensors {
            self.readings.remove(&sensor_id);
        }
    }

    /// Get statistics for sensor type
    pub fn get_type_statistics(&self, sensor_type: SensorType) -> SensorStatistics {
        let readings = self.get_readings_by_type(sensor_type);

        if readings.is_empty() {
            return SensorStatistics::default();
        }

        let values: Vec<f64> = readings.iter().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;

        let mean = sum / count;
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        SensorStatistics {
            count: values.len(),
            mean,
            min,
            max,
        }
    }
}

/// Sensor statistics
#[derive(Debug, Clone, Default)]
pub struct SensorStatistics {
    /// Number of readings
    pub count: usize,

    /// Mean value
    pub mean: f64,

    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_reading() {
        let reading = SensorReading::new(
            "temp1".to_string(),
            SensorType::Temperature,
            22.5,
        )
        .with_location(-122.4194, 37.7749)
        .with_quality(0.95);

        assert_eq!(reading.sensor_id, "temp1");
        assert_eq!(reading.value, 22.5);
        assert_eq!(reading.unit, "°C");
        assert!(reading.is_fresh(60));
    }

    #[tokio::test]
    async fn test_sensor_stream() {
        let stream = SensorStream::new(100);
        let mut rx = stream.subscribe();

        let reading = SensorReading::new("sensor1".to_string(), SensorType::Temperature, 25.0);
        stream.publish(reading.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.sensor_id, "sensor1");
        assert_eq!(stream.message_count(), 1);
    }

    #[tokio::test]
    async fn test_sensor_network() {
        let network = SensorNetwork::new(100);

        let reading1 = SensorReading::new("temp1".to_string(), SensorType::Temperature, 22.0);
        let reading2 = SensorReading::new("temp2".to_string(), SensorType::Temperature, 24.0);
        let reading3 = SensorReading::new("hum1".to_string(), SensorType::Humidity, 60.0);

        network.update_reading(reading1).await.unwrap();
        network.update_reading(reading2).await.unwrap();
        network.update_reading(reading3).await.unwrap();

        assert_eq!(network.sensor_count(), 3);

        let temp_readings = network.get_readings_by_type(SensorType::Temperature);
        assert_eq!(temp_readings.len(), 2);

        let stats = network.get_type_statistics(SensorType::Temperature);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean, 23.0);
    }
}
