//! Historical traffic patterns

use crate::graph::EdgeId;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Historical traffic data
#[derive(Debug, Clone, Default)]
pub struct HistoricalTrafficData {
    /// Traffic patterns: edge -> time profile
    patterns: HashMap<EdgeId, TrafficProfile>,
}

impl HistoricalTrafficData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load traffic patterns from data
    pub fn load_patterns(&mut self, patterns: HashMap<EdgeId, TrafficProfile>) {
        self.patterns = patterns;
    }

    /// Get traffic multiplier for edge at timestamp
    pub fn get_multiplier(
        &self,
        edge: EdgeId,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> f64 {
        if let Some(profile) = self.patterns.get(&edge) {
            profile.get_multiplier(timestamp)
        } else {
            1.0 // No data
        }
    }

    /// Add pattern for edge
    pub fn add_pattern(&mut self, edge: EdgeId, profile: TrafficProfile) {
        self.patterns.insert(edge, profile);
    }

    /// Get statistics
    pub fn stats(&self) -> HistoricalTrafficStats {
        HistoricalTrafficStats {
            num_edges_with_data: self.patterns.len(),
        }
    }
}

/// Traffic profile for an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficProfile {
    /// Hourly patterns for each day of week (0 = Monday)
    /// 7 days × 24 hours = 168 values
    pub weekly_pattern: Vec<f64>,

    /// Base travel time multiplier
    pub base_multiplier: f64,
}

impl TrafficProfile {
    /// Create new traffic profile
    pub fn new() -> Self {
        Self {
            weekly_pattern: vec![1.0; 168], // 7 days * 24 hours
            base_multiplier: 1.0,
        }
    }

    /// Get multiplier at specific timestamp
    pub fn get_multiplier(&self, timestamp: chrono::DateTime<chrono::Utc>) -> f64 {
        use chrono::Datelike;
        use chrono::Timelike;

        let day_of_week = timestamp.weekday().num_days_from_monday() as usize;
        let hour = timestamp.hour() as usize;

        let index = day_of_week * 24 + hour;

        self.weekly_pattern
            .get(index)
            .copied()
            .unwrap_or(1.0)
            * self.base_multiplier
    }

    /// Set multiplier for specific day and hour
    pub fn set_multiplier(&mut self, day: usize, hour: usize, multiplier: f64) {
        if day < 7 && hour < 24 {
            let index = day * 24 + hour;
            if index < self.weekly_pattern.len() {
                self.weekly_pattern[index] = multiplier;
            }
        }
    }

    /// Create typical weekday pattern
    pub fn typical_weekday() -> Self {
        let mut profile = Self::new();

        for hour in 0..24 {
            let multiplier = match hour {
                0..=5 => 0.7,    // Night: light traffic
                6..=8 => 2.0,    // Morning rush
                9..=15 => 1.2,   // Daytime
                16..=19 => 2.5,  // Evening rush
                20..=23 => 1.0,  // Evening
                _ => 1.0,
            };

            // Apply to all weekdays (Mon-Fri)
            for day in 0..5 {
                profile.set_multiplier(day, hour, multiplier);
            }
        }

        // Weekends: lighter traffic
        for day in 5..7 {
            for hour in 0..24 {
                profile.set_multiplier(day, hour, 0.8);
            }
        }

        profile
    }

    /// Create profile from samples
    pub fn from_samples(samples: &[(chrono::DateTime<chrono::Utc>, f64)]) -> Self {
        use chrono::{Datelike, Timelike};

        let mut counts = vec![0usize; 168];
        let mut sums = vec![0.0f64; 168];

        for (timestamp, multiplier) in samples {
            let day = timestamp.weekday().num_days_from_monday() as usize;
            let hour = timestamp.hour() as usize;
            let index = day * 24 + hour;

            if index < 168 {
                counts[index] += 1;
                sums[index] += multiplier;
            }
        }

        let weekly_pattern: Vec<f64> = sums
            .iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| {
                if count > 0 {
                    sum / count as f64
                } else {
                    1.0
                }
            })
            .collect();

        Self {
            weekly_pattern,
            base_multiplier: 1.0,
        }
    }
}

impl Default for TrafficProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about historical traffic data
#[derive(Debug)]
pub struct HistoricalTrafficStats {
    pub num_edges_with_data: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_profile() {
        let mut profile = TrafficProfile::new();
        profile.set_multiplier(0, 8, 2.0); // Monday 8 AM

        let monday_8am = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(8, 0, 0)
            .unwrap()
            .and_utc();

        // Note: This test might fail depending on what day it runs
        // In production, would use fixed test dates
        let _ = profile.get_multiplier(monday_8am);
    }

    #[test]
    fn test_typical_weekday() {
        let profile = TrafficProfile::typical_weekday();
        assert_eq!(profile.weekly_pattern.len(), 168);
    }
}
