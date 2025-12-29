//! Traffic integration and prediction

pub mod historical;
pub mod realtime;
pub mod prediction;

pub use historical::HistoricalTrafficData;
pub use realtime::RealtimeTrafficManager;
pub use prediction::TrafficPredictor;

use crate::graph::{EdgeId, Graph};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Traffic manager
pub struct TrafficManager {
    config: TrafficConfig,
    historical: Option<HistoricalTrafficData>,
    realtime: Option<RealtimeTrafficManager>,
    predictor: Option<TrafficPredictor>,
}

impl TrafficManager {
    pub fn new(config: TrafficConfig) -> Self {
        Self {
            config,
            historical: None,
            realtime: None,
            predictor: None,
        }
    }

    /// Load historical traffic patterns
    pub fn load_historical(&mut self, data: HistoricalTrafficData) {
        self.historical = Some(data);
    }

    /// Enable real-time traffic
    pub fn enable_realtime(&mut self, manager: RealtimeTrafficManager) {
        self.realtime = Some(manager);
    }

    /// Enable traffic prediction
    pub fn enable_prediction(&mut self, predictor: TrafficPredictor) {
        self.predictor = Some(predictor);
    }

    /// Get traffic multiplier for edge at given time
    pub fn get_traffic_multiplier(
        &self,
        edge: EdgeId,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> f64 {
        // Priority: realtime > prediction > historical > 1.0

        if let Some(ref rt) = self.realtime {
            if let Some(multiplier) = rt.get_current_multiplier(edge) {
                return multiplier;
            }
        }

        if let Some(ref pred) = self.predictor {
            if let Some(multiplier) = pred.predict_multiplier(edge, timestamp) {
                return multiplier;
            }
        }

        if let Some(ref hist) = self.historical {
            return hist.get_multiplier(edge, timestamp);
        }

        1.0 // No traffic data
    }

    /// Update edge costs with traffic
    pub fn apply_traffic(&self, graph: &mut Graph, timestamp: chrono::DateTime<chrono::Utc>) {
        // Apply traffic multipliers to all edges
        for edge_id in 0..graph.edge_count() {
            let multiplier = self.get_traffic_multiplier(EdgeId(edge_id), timestamp);
            // In real implementation, would modify edge costs
            let _ = multiplier; // Use variable
        }
    }
}

/// Traffic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficConfig {
    pub enable_historical: bool,
    pub enable_realtime: bool,
    pub enable_prediction: bool,
    pub update_interval_seconds: u64,
}

impl Default for TrafficConfig {
    fn default() -> Self {
        Self {
            enable_historical: true,
            enable_realtime: false,
            enable_prediction: false,
            update_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Traffic speed classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficSpeed {
    Free,      // > 80% of speed limit
    Light,     // 60-80%
    Moderate,  // 40-60%
    Heavy,     // 20-40%
    Congested, // < 20%
}

impl TrafficSpeed {
    pub fn multiplier(&self) -> f64 {
        match self {
            TrafficSpeed::Free => 1.0,
            TrafficSpeed::Light => 1.2,
            TrafficSpeed::Moderate => 1.5,
            TrafficSpeed::Heavy => 2.5,
            TrafficSpeed::Congested => 4.0,
        }
    }
}
