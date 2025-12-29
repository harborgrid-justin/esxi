//! Real-time traffic updates

use crate::graph::EdgeId;
use hashbrown::HashMap;
use std::sync::{Arc, RwLock};

/// Real-time traffic manager
#[derive(Clone)]
pub struct RealtimeTrafficManager {
    /// Current traffic state
    state: Arc<RwLock<TrafficState>>,

    /// Update interval
    update_interval: std::time::Duration,
}

impl RealtimeTrafficManager {
    pub fn new(update_interval: std::time::Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(TrafficState::default())),
            update_interval,
        }
    }

    /// Update traffic for edge
    pub fn update_edge(&self, edge: EdgeId, speed_multiplier: f64) {
        if let Ok(mut state) = self.state.write() {
            state.edge_multipliers.insert(edge, speed_multiplier);
            state.last_update = chrono::Utc::now();
        }
    }

    /// Bulk update
    pub fn update_batch(&self, updates: HashMap<EdgeId, f64>) {
        if let Ok(mut state) = self.state.write() {
            state.edge_multipliers.extend(updates);
            state.last_update = chrono::Utc::now();
        }
    }

    /// Get current multiplier for edge
    pub fn get_current_multiplier(&self, edge: EdgeId) -> Option<f64> {
        self.state
            .read()
            .ok()?
            .edge_multipliers
            .get(&edge)
            .copied()
    }

    /// Get last update time
    pub fn last_update(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.state.read().ok().map(|s| s.last_update)
    }

    /// Clear all traffic data
    pub fn clear(&self) {
        if let Ok(mut state) = self.state.write() {
            state.edge_multipliers.clear();
        }
    }

    /// Process traffic incident
    pub fn add_incident(&self, incident: TrafficIncident) {
        if let Ok(mut state) = self.state.write() {
            // Apply incident to affected edges
            for &edge in &incident.affected_edges {
                let multiplier = incident.severity.multiplier();
                state.edge_multipliers.insert(edge, multiplier);
            }

            state.incidents.push(incident);
        }
    }

    /// Remove expired incidents
    pub fn cleanup_incidents(&self, now: chrono::DateTime<chrono::Utc>) {
        if let Ok(mut state) = self.state.write() {
            state.incidents.retain(|inc| {
                inc.end_time.map(|end| end > now).unwrap_or(true)
            });
        }
    }

    /// Get statistics
    pub fn stats(&self) -> RealtimeTrafficStats {
        if let Ok(state) = self.state.read() {
            RealtimeTrafficStats {
                num_edges_updated: state.edge_multipliers.len(),
                active_incidents: state.incidents.len(),
                last_update: state.last_update,
            }
        } else {
            RealtimeTrafficStats::default()
        }
    }
}

/// Internal traffic state
#[derive(Default)]
struct TrafficState {
    /// Edge multipliers
    edge_multipliers: HashMap<EdgeId, f64>,

    /// Active incidents
    incidents: Vec<TrafficIncident>,

    /// Last update timestamp
    last_update: chrono::DateTime<chrono::Utc>,
}

/// Traffic incident
#[derive(Debug, Clone)]
pub struct TrafficIncident {
    pub id: String,
    pub incident_type: IncidentType,
    pub severity: IncidentSeverity,
    pub affected_edges: Vec<EdgeId>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub description: String,
}

/// Type of traffic incident
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentType {
    Accident,
    Construction,
    Congestion,
    Weather,
    Event,
    Other,
}

/// Severity of incident
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentSeverity {
    Minor,
    Moderate,
    Major,
    Severe,
}

impl IncidentSeverity {
    pub fn multiplier(&self) -> f64 {
        match self {
            IncidentSeverity::Minor => 1.5,
            IncidentSeverity::Moderate => 2.0,
            IncidentSeverity::Major => 3.0,
            IncidentSeverity::Severe => 5.0,
        }
    }
}

/// Statistics about real-time traffic
#[derive(Debug, Default)]
pub struct RealtimeTrafficStats {
    pub num_edges_updated: usize,
    pub active_incidents: usize,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_traffic() {
        let manager = RealtimeTrafficManager::new(std::time::Duration::from_secs(60));

        manager.update_edge(EdgeId(0), 1.5);
        assert_eq!(manager.get_current_multiplier(EdgeId(0)), Some(1.5));

        manager.clear();
        assert_eq!(manager.get_current_multiplier(EdgeId(0)), None);
    }

    #[test]
    fn test_incident() {
        let manager = RealtimeTrafficManager::new(std::time::Duration::from_secs(60));

        let incident = TrafficIncident {
            id: "inc1".to_string(),
            incident_type: IncidentType::Accident,
            severity: IncidentSeverity::Major,
            affected_edges: vec![EdgeId(0), EdgeId(1)],
            start_time: chrono::Utc::now(),
            end_time: None,
            description: "Test incident".to_string(),
        };

        manager.add_incident(incident);

        let stats = manager.stats();
        assert_eq!(stats.active_incidents, 1);
    }
}
