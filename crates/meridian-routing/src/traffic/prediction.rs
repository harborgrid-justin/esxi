//! Traffic prediction using historical patterns and ML

use crate::graph::EdgeId;
use crate::traffic::historical::TrafficProfile;
use hashbrown::HashMap;

/// Traffic predictor
pub struct TrafficPredictor {
    /// Historical patterns for prediction
    patterns: HashMap<EdgeId, TrafficProfile>,

    /// Prediction model (simplified)
    model: PredictionModel,
}

impl TrafficPredictor {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            model: PredictionModel::Historical,
        }
    }

    /// Load historical patterns
    pub fn load_patterns(&mut self, patterns: HashMap<EdgeId, TrafficProfile>) {
        self.patterns = patterns;
    }

    /// Predict traffic multiplier for edge at future time
    pub fn predict_multiplier(
        &self,
        edge: EdgeId,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Option<f64> {
        match self.model {
            PredictionModel::Historical => {
                // Use historical pattern
                self.patterns.get(&edge).map(|p| p.get_multiplier(timestamp))
            }
            PredictionModel::MachineLearning => {
                // Placeholder for ML model
                // Would use features like: time of day, day of week, weather, events
                self.patterns.get(&edge).map(|p| p.get_multiplier(timestamp))
            }
        }
    }

    /// Predict for multiple edges
    pub fn predict_batch(
        &self,
        edges: &[EdgeId],
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> HashMap<EdgeId, f64> {
        edges
            .iter()
            .filter_map(|&edge| {
                self.predict_multiplier(edge, timestamp)
                    .map(|mult| (edge, mult))
            })
            .collect()
    }

    /// Train prediction model (placeholder)
    pub fn train(&mut self, _training_data: &[TrainingSample]) {
        // In production, would train ML model here
        log::info!("Training prediction model (placeholder)");
    }

    /// Set prediction model
    pub fn set_model(&mut self, model: PredictionModel) {
        self.model = model;
    }
}

impl Default for TrafficPredictor {
    fn default() -> Self {
        Self::new()
    }
}

/// Prediction model type
#[derive(Debug, Clone, Copy)]
pub enum PredictionModel {
    /// Use historical patterns
    Historical,

    /// Use machine learning
    MachineLearning,
}

/// Training sample for ML model
pub struct TrainingSample {
    pub edge: EdgeId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub observed_multiplier: f64,
    pub features: TrafficFeatures,
}

/// Features for traffic prediction
pub struct TrafficFeatures {
    pub day_of_week: u8,
    pub hour_of_day: u8,
    pub is_holiday: bool,
    pub weather_condition: WeatherCondition,
    pub special_event: bool,
}

/// Weather conditions
#[derive(Debug, Clone, Copy)]
pub enum WeatherCondition {
    Clear,
    Rain,
    Snow,
    Fog,
    Storm,
}

impl WeatherCondition {
    pub fn traffic_impact(&self) -> f64 {
        match self {
            WeatherCondition::Clear => 1.0,
            WeatherCondition::Rain => 1.3,
            WeatherCondition::Snow => 2.0,
            WeatherCondition::Fog => 1.5,
            WeatherCondition::Storm => 2.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traffic::historical::TrafficProfile;

    #[test]
    fn test_predictor() {
        let mut predictor = TrafficPredictor::new();

        let mut patterns = HashMap::new();
        patterns.insert(EdgeId(0), TrafficProfile::typical_weekday());

        predictor.load_patterns(patterns);

        let future = chrono::Utc::now() + chrono::Duration::hours(1);
        let prediction = predictor.predict_multiplier(EdgeId(0), future);

        assert!(prediction.is_some());
    }
}
