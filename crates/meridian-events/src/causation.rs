//! Causation and correlation tracking for distributed event flows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Causation identifier - tracks what caused an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(Uuid);

impl CausationId {
    /// Generate a new causation ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CausationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CausationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for CausationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Correlation identifier - tracks related events across boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    /// Generate a new correlation ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for CorrelationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Context for tracking causation and correlation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausationContext {
    /// Current causation ID
    pub causation_id: CausationId,
    /// Correlation ID for the entire flow
    pub correlation_id: CorrelationId,
    /// Additional context data
    pub metadata: HashMap<String, String>,
}

impl CausationContext {
    /// Create a new causation context.
    pub fn new() -> Self {
        Self {
            causation_id: CausationId::new(),
            correlation_id: CorrelationId::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new context with the same correlation ID but new causation.
    pub fn with_new_causation(&self) -> Self {
        Self {
            causation_id: CausationId::new(),
            correlation_id: self.correlation_id,
            metadata: self.metadata.clone(),
        }
    }

    /// Add metadata to the context.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for CausationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracker for building causation chains.
#[derive(Debug, Clone)]
pub struct CausationChain {
    /// Chain of causation IDs
    chain: Vec<CausationId>,
    /// Correlation ID
    correlation_id: CorrelationId,
}

impl CausationChain {
    /// Create a new causation chain.
    pub fn new(correlation_id: CorrelationId) -> Self {
        Self {
            chain: Vec::new(),
            correlation_id,
        }
    }

    /// Add a causation ID to the chain.
    pub fn add(&mut self, causation_id: CausationId) {
        self.chain.push(causation_id);
    }

    /// Get the full chain.
    pub fn chain(&self) -> &[CausationId] {
        &self.chain
    }

    /// Get the correlation ID.
    pub fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }

    /// Get the length of the chain.
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    /// Get the root causation ID (first in chain).
    pub fn root(&self) -> Option<CausationId> {
        self.chain.first().copied()
    }

    /// Get the immediate causation ID (last in chain).
    pub fn immediate(&self) -> Option<CausationId> {
        self.chain.last().copied()
    }
}

/// Manager for tracking causation relationships across the system.
#[derive(Debug, Default)]
pub struct CausationTracker {
    /// Active causation chains indexed by correlation ID
    chains: HashMap<CorrelationId, CausationChain>,
}

impl CausationTracker {
    /// Create a new causation tracker.
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
        }
    }

    /// Start a new causation chain.
    pub fn start_chain(&mut self, context: &CausationContext) {
        let mut chain = CausationChain::new(context.correlation_id);
        chain.add(context.causation_id);
        self.chains.insert(context.correlation_id, chain);
    }

    /// Add to an existing causation chain.
    pub fn add_to_chain(&mut self, correlation_id: CorrelationId, causation_id: CausationId) {
        self.chains
            .entry(correlation_id)
            .or_insert_with(|| CausationChain::new(correlation_id))
            .add(causation_id);
    }

    /// Get a causation chain.
    pub fn get_chain(&self, correlation_id: &CorrelationId) -> Option<&CausationChain> {
        self.chains.get(correlation_id)
    }

    /// Remove a causation chain (when flow completes).
    pub fn remove_chain(&mut self, correlation_id: &CorrelationId) -> Option<CausationChain> {
        self.chains.remove(correlation_id)
    }

    /// Get the number of active chains.
    pub fn active_chains(&self) -> usize {
        self.chains.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causation_id_generation() {
        let id1 = CausationId::new();
        let id2 = CausationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_correlation_id_generation() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_causation_context() {
        let ctx = CausationContext::new();
        let new_ctx = ctx.with_new_causation();

        assert_ne!(ctx.causation_id, new_ctx.causation_id);
        assert_eq!(ctx.correlation_id, new_ctx.correlation_id);
    }

    #[test]
    fn test_causation_chain() {
        let correlation_id = CorrelationId::new();
        let mut chain = CausationChain::new(correlation_id);

        assert!(chain.is_empty());

        let cause1 = CausationId::new();
        let cause2 = CausationId::new();

        chain.add(cause1);
        chain.add(cause2);

        assert_eq!(chain.len(), 2);
        assert_eq!(chain.root(), Some(cause1));
        assert_eq!(chain.immediate(), Some(cause2));
    }

    #[test]
    fn test_causation_tracker() {
        let mut tracker = CausationTracker::new();
        let ctx = CausationContext::new();

        tracker.start_chain(&ctx);
        assert_eq!(tracker.active_chains(), 1);

        let chain = tracker.get_chain(&ctx.correlation_id);
        assert!(chain.is_some());
        assert_eq!(chain.unwrap().len(), 1);

        tracker.remove_chain(&ctx.correlation_id);
        assert_eq!(tracker.active_chains(), 0);
    }
}
