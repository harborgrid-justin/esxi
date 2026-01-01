//! Load Balancer Implementation
//!
//! Enterprise load balancing with multiple strategies: round-robin, least-connections, weighted, IP hash.

use crate::config::{LoadBalancerStrategy as ConfigStrategy, UpstreamConfig};
use parking_lot::RwLock;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use thiserror::Error;

/// Load balancer errors
#[derive(Debug, Error)]
pub enum LoadBalancerError {
    /// No healthy upstreams available to handle request
    #[error("No healthy upstreams available")]
    NoHealthyUpstreams,

    /// Invalid upstream configuration
    #[error("Invalid upstream configuration")]
    InvalidConfig,
}

/// Load balancer strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBalancerStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Least active connections
    LeastConnections,
    /// Weighted distribution
    Weighted,
    /// IP-based hashing
    IpHash,
    /// Random selection
    Random,
}

impl From<ConfigStrategy> for LoadBalancerStrategy {
    fn from(config: ConfigStrategy) -> Self {
        match config {
            ConfigStrategy::RoundRobin => Self::RoundRobin,
            ConfigStrategy::LeastConnections => Self::LeastConnections,
            ConfigStrategy::Weighted => Self::Weighted,
            ConfigStrategy::IpHash => Self::IpHash,
            ConfigStrategy::Random => Self::Random,
        }
    }
}

/// Upstream server
#[derive(Debug, Clone)]
pub struct Upstream {
    /// Unique upstream identifier
    pub id: String,
    /// Upstream server URL
    pub url: String,
    /// Load balancing weight (for weighted strategy)
    pub weight: u32,
    /// Health status of upstream
    pub healthy: bool,
    /// Active connection count
    pub active_connections: Arc<AtomicUsize>,
}

impl Upstream {
    /// Create a new upstream
    pub fn new(id: String, url: String, weight: u32) -> Self {
        Self {
            id,
            url,
            weight,
            healthy: true,
            active_connections: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Increment active connections
    pub fn acquire(&self) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement active connections
    pub fn release(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }

    /// Get current connection count
    pub fn connections(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }
}

/// Load Balancer
pub struct LoadBalancer {
    upstreams: Arc<RwLock<Vec<Upstream>>>,
    strategy: LoadBalancerStrategy,
    round_robin_index: Arc<AtomicUsize>,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: LoadBalancerStrategy) -> Self {
        Self {
            upstreams: Arc::new(RwLock::new(vec![])),
            strategy,
            round_robin_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create from config
    pub fn from_config(
        upstreams: Vec<UpstreamConfig>,
        strategy: ConfigStrategy,
    ) -> Result<Self, LoadBalancerError> {
        if upstreams.is_empty() {
            return Err(LoadBalancerError::InvalidConfig);
        }

        let lb = Self::new(strategy.into());

        for config in upstreams {
            lb.add_upstream(Upstream::new(
                config.id,
                config.url,
                config.weight,
            ));
        }

        Ok(lb)
    }

    /// Add upstream server
    pub fn add_upstream(&self, upstream: Upstream) {
        self.upstreams.write().push(upstream);
    }

    /// Remove upstream server
    pub fn remove_upstream(&self, id: &str) {
        self.upstreams.write().retain(|u| u.id != id);
    }

    /// Mark upstream as healthy/unhealthy
    pub fn set_health(&self, id: &str, healthy: bool) {
        let mut upstreams = self.upstreams.write();
        if let Some(upstream) = upstreams.iter_mut().find(|u| u.id == id) {
            upstream.healthy = healthy;
        }
    }

    /// Select upstream based on strategy
    pub fn select(&self, client_ip: Option<&str>) -> Result<Upstream, LoadBalancerError> {
        let upstreams = self.upstreams.read();
        let healthy: Vec<&Upstream> = upstreams.iter().filter(|u| u.healthy).collect();

        if healthy.is_empty() {
            return Err(LoadBalancerError::NoHealthyUpstreams);
        }

        let selected = match self.strategy {
            LoadBalancerStrategy::RoundRobin => self.round_robin(&healthy),
            LoadBalancerStrategy::LeastConnections => self.least_connections(&healthy),
            LoadBalancerStrategy::Weighted => self.weighted(&healthy),
            LoadBalancerStrategy::IpHash => self.ip_hash(&healthy, client_ip),
            LoadBalancerStrategy::Random => self.random(&healthy),
        };

        Ok(selected.clone())
    }

    /// Round-robin selection
    fn round_robin<'a>(&self, upstreams: &[&'a Upstream]) -> &'a Upstream {
        let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst);
        upstreams[index % upstreams.len()]
    }

    /// Least connections selection
    fn least_connections<'a>(&self, upstreams: &[&'a Upstream]) -> &'a Upstream {
        upstreams
            .iter()
            .min_by_key(|u| u.connections())
            .copied()
            .unwrap()
    }

    /// Weighted round-robin selection
    fn weighted<'a>(&self, upstreams: &[&'a Upstream]) -> &'a Upstream {
        // Calculate total weight
        let total_weight: u32 = upstreams.iter().map(|u| u.weight).sum();

        if total_weight == 0 {
            return upstreams[0];
        }

        // Get weighted index
        let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst);
        let mut target = (index as u32 % total_weight) + 1;

        // Find upstream based on weight
        for upstream in upstreams {
            if target <= upstream.weight {
                return upstream;
            }
            target -= upstream.weight;
        }

        upstreams[0]
    }

    /// IP hash selection (consistent hashing)
    fn ip_hash<'a>(&self, upstreams: &[&'a Upstream], client_ip: Option<&str>) -> &'a Upstream {
        if let Some(ip) = client_ip {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            ip.hash(&mut hasher);
            let hash = hasher.finish();
            let index = (hash as usize) % upstreams.len();
            upstreams[index]
        } else {
            // Fallback to round-robin if no IP
            self.round_robin(upstreams)
        }
    }

    /// Random selection
    fn random<'a>(&self, upstreams: &[&'a Upstream]) -> &'a Upstream {
        use std::time::SystemTime;

        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let index = (seed as usize) % upstreams.len();
        upstreams[index]
    }

    /// Get all upstreams
    pub fn upstreams(&self) -> Vec<Upstream> {
        self.upstreams.read().clone()
    }

    /// Get healthy upstream count
    pub fn healthy_count(&self) -> usize {
        self.upstreams.read().iter().filter(|u| u.healthy).count()
    }

    /// Get total upstream count
    pub fn total_count(&self) -> usize {
        self.upstreams.read().len()
    }
}

impl Clone for LoadBalancer {
    fn clone(&self) -> Self {
        Self {
            upstreams: Arc::clone(&self.upstreams),
            strategy: self.strategy,
            round_robin_index: Arc::clone(&self.round_robin_index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_upstreams() -> Vec<Upstream> {
        vec![
            Upstream::new("up1".to_string(), "http://localhost:8001".to_string(), 1),
            Upstream::new("up2".to_string(), "http://localhost:8002".to_string(), 2),
            Upstream::new("up3".to_string(), "http://localhost:8003".to_string(), 1),
        ]
    }

    #[test]
    fn test_round_robin() {
        let lb = LoadBalancer::new(LoadBalancerStrategy::RoundRobin);
        for upstream in create_upstreams() {
            lb.add_upstream(upstream);
        }

        let selected1 = lb.select(None).unwrap();
        let selected2 = lb.select(None).unwrap();
        let selected3 = lb.select(None).unwrap();
        let selected4 = lb.select(None).unwrap();

        // Should cycle through upstreams
        assert_ne!(selected1.id, selected2.id);
        assert_eq!(selected1.id, selected4.id);
    }

    #[test]
    fn test_least_connections() {
        let lb = LoadBalancer::new(LoadBalancerStrategy::LeastConnections);
        for upstream in create_upstreams() {
            lb.add_upstream(upstream);
        }

        // All should have 0 connections initially
        let selected1 = lb.select(None).unwrap();
        selected1.acquire();

        // Should select a different upstream with 0 connections
        let selected2 = lb.select(None).unwrap();
        assert_ne!(selected1.id, selected2.id);
    }

    #[test]
    fn test_ip_hash_consistency() {
        let lb = LoadBalancer::new(LoadBalancerStrategy::IpHash);
        for upstream in create_upstreams() {
            lb.add_upstream(upstream);
        }

        let client_ip = "192.168.1.100";

        // Same IP should always select same upstream
        let selected1 = lb.select(Some(client_ip)).unwrap();
        let selected2 = lb.select(Some(client_ip)).unwrap();
        let selected3 = lb.select(Some(client_ip)).unwrap();

        assert_eq!(selected1.id, selected2.id);
        assert_eq!(selected2.id, selected3.id);
    }

    #[test]
    fn test_health_check() {
        let lb = LoadBalancer::new(LoadBalancerStrategy::RoundRobin);
        lb.add_upstream(Upstream::new(
            "up1".to_string(),
            "http://localhost:8001".to_string(),
            1,
        ));
        lb.add_upstream(Upstream::new(
            "up2".to_string(),
            "http://localhost:8002".to_string(),
            1,
        ));

        assert_eq!(lb.healthy_count(), 2);

        lb.set_health("up1", false);
        assert_eq!(lb.healthy_count(), 1);

        let selected = lb.select(None).unwrap();
        assert_eq!(selected.id, "up2");
    }

    #[test]
    fn test_no_healthy_upstreams() {
        let lb = LoadBalancer::new(LoadBalancerStrategy::RoundRobin);
        lb.add_upstream(Upstream::new(
            "up1".to_string(),
            "http://localhost:8001".to_string(),
            1,
        ));

        lb.set_health("up1", false);

        let result = lb.select(None);
        assert!(result.is_err());
    }
}
