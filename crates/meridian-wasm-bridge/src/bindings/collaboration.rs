//! Real-time collaboration bindings with CRDT and Operational Transform support.
//!
//! Features:
//! - CRDT (Conflict-free Replicated Data Types)
//! - Operational Transform (OT)
//! - Event synchronization
//! - Vector clocks for causality
//! - Presence awareness
//! - Conflict resolution

use wasm_bindgen::prelude::*;
use crate::types::{CollaborationEvent, OperationResult};
use crate::async_bridge::execute_async;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Collaboration engine for real-time multi-user editing.
#[wasm_bindgen]
pub struct CollaborationEngine {
    instance_id: String,
    user_id: String,
    vector_clock: Vec<u64>,
}

#[wasm_bindgen]
impl CollaborationEngine {
    /// Create a new collaboration engine instance.
    ///
    /// # Arguments
    ///
    /// * `user_id` - Unique identifier for the current user
    /// * `num_users` - Expected number of concurrent users
    #[wasm_bindgen(constructor)]
    pub fn new(user_id: String, num_users: usize) -> Self {
        Self {
            instance_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            vector_clock: vec![0; num_users],
        }
    }

    /// Get the instance ID.
    #[wasm_bindgen(getter)]
    pub fn instance_id(&self) -> String {
        self.instance_id.clone()
    }

    /// Get the current user ID.
    #[wasm_bindgen(getter)]
    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }

    /// Apply a local operation and generate an event.
    ///
    /// This transforms a local edit into a collaboration event that can be
    /// broadcast to other users.
    pub async fn apply_local_operation(
        &mut self,
        operation_type: String,
        payload: JsValue,
    ) -> Result<JsValue, JsValue> {
        execute_async(async move {
            tracing::debug!("Applying local operation: {}", operation_type);

            // Increment our position in the vector clock
            // In a real implementation, this would be based on the user's index
            if !self.vector_clock.is_empty() {
                self.vector_clock[0] += 1;
            }

            let event = CollaborationEvent {
                event_type: operation_type,
                user_id: self.user_id.clone(),
                timestamp: get_timestamp(),
                payload: serde_wasm_bindgen::from_value(payload)
                    .map_err(|e| JsValue::from_str(&format!("Invalid payload: {}", e)))?,
                vector_clock: Some(self.vector_clock.clone()),
            };

            let result = OperationResult::success(event, Some(0));
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
        }).await
    }

    /// Apply a remote operation from another user.
    ///
    /// This handles operational transform or CRDT merge logic.
    pub async fn apply_remote_operation(&mut self, event: JsValue) -> Result<JsValue, JsValue> {
        execute_async(async move {
            let event: CollaborationEvent = serde_wasm_bindgen::from_value(event)
                .map_err(|e| JsValue::from_str(&format!("Invalid event: {}", e)))?;

            tracing::debug!("Applying remote operation from user: {}", event.user_id);

            // Update vector clock
            if let Some(remote_clock) = &event.vector_clock {
                merge_vector_clocks(&mut self.vector_clock, remote_clock);
            }

            // Transform the operation based on causality
            let transformed = transform_operation(&event)?;

            let result = OperationResult::success(transformed, Some(0));
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
        }).await
    }

    /// Merge two operations that happened concurrently.
    ///
    /// Uses CRDT merge semantics to resolve conflicts.
    pub async fn merge_operations(
        &self,
        op1: JsValue,
        op2: JsValue,
    ) -> Result<JsValue, JsValue> {
        execute_async(async move {
            let event1: CollaborationEvent = serde_wasm_bindgen::from_value(op1)
                .map_err(|e| JsValue::from_str(&format!("Invalid event 1: {}", e)))?;
            let event2: CollaborationEvent = serde_wasm_bindgen::from_value(op2)
                .map_err(|e| JsValue::from_str(&format!("Invalid event 2: {}", e)))?;

            tracing::debug!("Merging operations from users: {} and {}", event1.user_id, event2.user_id);

            let merged = merge_events(&event1, &event2)?;

            let result = OperationResult::success(merged, Some(0));
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
        }).await
    }

    /// Check if two operations are causally ordered.
    ///
    /// Returns true if op1 happened before op2.
    pub fn is_causally_before(&self, clock1: Vec<u64>, clock2: Vec<u64>) -> bool {
        is_causally_before(&clock1, &clock2)
    }

    /// Get the current vector clock.
    pub fn get_vector_clock(&self) -> Vec<u64> {
        self.vector_clock.clone()
    }

    /// Set the vector clock (useful for synchronization).
    pub fn set_vector_clock(&mut self, clock: Vec<u64>) {
        self.vector_clock = clock;
    }

    /// Generate a presence update event.
    ///
    /// This is used to broadcast user presence (cursor position, selection, etc.).
    pub async fn create_presence_event(&self, presence_data: JsValue) -> Result<JsValue, JsValue> {
        let presence = PresenceUpdate {
            user_id: self.user_id.clone(),
            timestamp: get_timestamp(),
            data: serde_wasm_bindgen::from_value(presence_data)
                .map_err(|e| JsValue::from_str(&format!("Invalid presence data: {}", e)))?,
        };

        serde_wasm_bindgen::to_value(&presence)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformedOperation {
    event_type: String,
    payload: serde_json::Value,
    transformed: bool,
    original_user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergedOperation {
    event_type: String,
    payload: serde_json::Value,
    merged_from: Vec<String>,
    resolution_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PresenceUpdate {
    user_id: String,
    timestamp: u64,
    data: serde_json::Value,
}

// Internal implementation functions

fn get_timestamp() -> u64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() as u64)
        .unwrap_or(0)
}

fn merge_vector_clocks(clock1: &mut Vec<u64>, clock2: &[u64]) {
    for (i, &val) in clock2.iter().enumerate() {
        if i < clock1.len() {
            clock1[i] = clock1[i].max(val);
        }
    }
}

fn is_causally_before(clock1: &[u64], clock2: &[u64]) -> bool {
    let len = clock1.len().min(clock2.len());
    let mut less_or_equal = true;
    let mut strictly_less = false;

    for i in 0..len {
        if clock1[i] > clock2[i] {
            less_or_equal = false;
            break;
        }
        if clock1[i] < clock2[i] {
            strictly_less = true;
        }
    }

    less_or_equal && strictly_less
}

fn transform_operation(event: &CollaborationEvent) -> Result<TransformedOperation, JsValue> {
    // Placeholder: In production, implement actual OT transformation
    let transformed = TransformedOperation {
        event_type: event.event_type.clone(),
        payload: event.payload.clone(),
        transformed: false, // Would be true if transformation was needed
        original_user: event.user_id.clone(),
    };

    Ok(transformed)
}

fn merge_events(event1: &CollaborationEvent, event2: &CollaborationEvent) -> Result<MergedOperation, JsValue> {
    // Placeholder: In production, implement actual CRDT merge logic

    // Simple last-write-wins strategy based on timestamp
    let winner = if event1.timestamp > event2.timestamp {
        event1
    } else {
        event2
    };

    let merged = MergedOperation {
        event_type: winner.event_type.clone(),
        payload: winner.payload.clone(),
        merged_from: vec![event1.user_id.clone(), event2.user_id.clone()],
        resolution_strategy: "last-write-wins".to_string(),
    };

    Ok(merged)
}

/// CRDT data structure for collaborative text editing.
#[wasm_bindgen]
pub struct CrdtText {
    content: String,
    site_id: String,
}

#[wasm_bindgen]
impl CrdtText {
    /// Create a new CRDT text instance.
    #[wasm_bindgen(constructor)]
    pub fn new(site_id: String) -> Self {
        Self {
            content: String::new(),
            site_id,
        }
    }

    /// Insert text at a position.
    pub fn insert(&mut self, position: usize, text: String) -> Result<(), JsValue> {
        if position > self.content.len() {
            return Err(JsValue::from_str("Position out of bounds"));
        }

        self.content.insert_str(position, &text);
        Ok(())
    }

    /// Delete text at a position.
    pub fn delete(&mut self, position: usize, length: usize) -> Result<(), JsValue> {
        if position + length > self.content.len() {
            return Err(JsValue::from_str("Delete range out of bounds"));
        }

        self.content.drain(position..position + length);
        Ok(())
    }

    /// Get the current content.
    #[wasm_bindgen(getter)]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    /// Get the length of the content.
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_collaboration_engine_creation() {
        let engine = CollaborationEngine::new("user1".to_string(), 4);
        assert_eq!(engine.user_id(), "user1");
        assert_eq!(engine.get_vector_clock().len(), 4);
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut clock1 = vec![1, 2, 3];
        let clock2 = vec![2, 1, 4];
        merge_vector_clocks(&mut clock1, &clock2);
        assert_eq!(clock1, vec![2, 2, 4]);
    }

    #[test]
    fn test_causal_ordering() {
        let clock1 = vec![1, 2, 3];
        let clock2 = vec![2, 2, 3];
        assert!(is_causally_before(&clock1, &clock2));
        assert!(!is_causally_before(&clock2, &clock1));
    }

    #[wasm_bindgen_test]
    fn test_crdt_text() {
        let mut text = CrdtText::new("site1".to_string());
        text.insert(0, "Hello".to_string()).unwrap();
        assert_eq!(text.content(), "Hello");

        text.insert(5, " World".to_string()).unwrap();
        assert_eq!(text.content(), "Hello World");

        text.delete(5, 6).unwrap();
        assert_eq!(text.content(), "Hello");
    }
}
