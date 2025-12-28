//! Real-time synchronization with conflict resolution and operational transforms.

use crate::error::{Result, StreamError};
use crate::messages::{ClientId, SyncMessage, Timestamp, current_timestamp};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{debug, info};

/// Maximum number of operations to keep in history.
const MAX_OPERATION_HISTORY: usize = 1000;

/// Unique identifier for an entity.
pub type EntityId = String;

/// Operation transform type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op_type", rename_all = "snake_case")]
pub enum Operation {
    /// Insert operation
    Insert {
        position: usize,
        value: serde_json::Value,
    },
    /// Delete operation
    Delete {
        position: usize,
        length: usize,
    },
    /// Update operation
    Update {
        path: String,
        value: serde_json::Value,
    },
    /// Move operation
    Move {
        from: usize,
        to: usize,
    },
    /// Custom operation
    Custom {
        name: String,
        params: serde_json::Value,
    },
}

impl Operation {
    /// Transform this operation against another operation.
    pub fn transform(&self, other: &Operation, is_left: bool) -> Result<Operation> {
        match (self, other) {
            // Insert vs Insert
            (
                Operation::Insert { position: pos1, value: val1 },
                Operation::Insert { position: pos2, .. },
            ) => {
                let new_pos = if pos1 > pos2 || (pos1 == pos2 && !is_left) {
                    pos1 + 1
                } else {
                    *pos1
                };
                Ok(Operation::Insert {
                    position: new_pos,
                    value: val1.clone(),
                })
            }

            // Insert vs Delete
            (
                Operation::Insert { position: pos1, value: val1 },
                Operation::Delete { position: pos2, length },
            ) => {
                let new_pos = if pos1 > pos2 {
                    pos1.saturating_sub(*length)
                } else {
                    *pos1
                };
                Ok(Operation::Insert {
                    position: new_pos,
                    value: val1.clone(),
                })
            }

            // Delete vs Insert
            (
                Operation::Delete { position: pos1, length: len1 },
                Operation::Insert { position: pos2, .. },
            ) => {
                let new_pos = if pos1 >= pos2 {
                    pos1 + 1
                } else {
                    *pos1
                };
                Ok(Operation::Delete {
                    position: new_pos,
                    length: *len1,
                })
            }

            // Delete vs Delete
            (
                Operation::Delete { position: pos1, length: len1 },
                Operation::Delete { position: pos2, length: len2 },
            ) => {
                if pos1 == pos2 {
                    // Same position, keep smaller length
                    Ok(Operation::Delete {
                        position: *pos1,
                        length: (*len1).min(*len2),
                    })
                } else if pos1 > pos2 {
                    Ok(Operation::Delete {
                        position: pos1.saturating_sub(*len2),
                        length: *len1,
                    })
                } else {
                    Ok(Operation::Delete {
                        position: *pos1,
                        length: *len1,
                    })
                }
            }

            // For other combinations, return the operation as-is
            _ => Ok(self.clone()),
        }
    }

    /// Apply this operation to a JSON value.
    pub fn apply(&self, value: &mut serde_json::Value) -> Result<()> {
        match self {
            Operation::Update { path, value: new_val } => {
                if path.is_empty() {
                    *value = new_val.clone();
                } else {
                    apply_json_path(value, path, new_val.clone())?;
                }
            }
            Operation::Custom { name, params: _ } => {
                debug!("Applying custom operation: {}", name);
                // Custom operations would be handled by application-specific logic
            }
            _ => {
                // Insert, Delete, Move operations would need more context
                // about the data structure they're operating on
            }
        }
        Ok(())
    }
}

/// Apply a value to a JSON path.
fn apply_json_path(root: &mut serde_json::Value, path: &str, value: serde_json::Value) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part, set the value
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), value);
                return Ok(());
            }
        } else {
            // Navigate to the next level
            if let Some(obj) = current.as_object_mut() {
                current = obj.entry(part.to_string())
                    .or_insert(serde_json::json!({}));
            } else {
                return Err(StreamError::invalid_message("Invalid JSON path"));
            }
        }
    }

    Ok(())
}

/// Versioned state with operation history.
#[derive(Debug, Clone)]
struct VersionedState {
    /// Current version
    version: u64,
    /// Current state
    state: serde_json::Value,
    /// Operation history
    history: VecDeque<(u64, ClientId, Operation)>,
    /// Last update timestamp
    last_update: Timestamp,
}

impl VersionedState {
    /// Create a new versioned state.
    fn new(initial_state: serde_json::Value) -> Self {
        Self {
            version: 0,
            state: initial_state,
            history: VecDeque::new(),
            last_update: current_timestamp(),
        }
    }

    /// Apply an operation.
    fn apply_operation(&mut self, client_id: ClientId, operation: Operation) -> Result<u64> {
        // Apply the operation
        operation.apply(&mut self.state)?;

        // Increment version
        self.version += 1;

        // Add to history
        self.history.push_back((self.version, client_id, operation));
        if self.history.len() > MAX_OPERATION_HISTORY {
            self.history.pop_front();
        }

        self.last_update = current_timestamp();

        Ok(self.version)
    }

    /// Get operations since a version.
    fn get_operations_since(&self, version: u64) -> Vec<(u64, ClientId, Operation)> {
        self.history
            .iter()
            .filter(|(v, _, _)| *v > version)
            .cloned()
            .collect()
    }
}

/// Synchronization manager for handling real-time state sync.
#[derive(Clone)]
pub struct SyncManager {
    /// Entity states
    states: Arc<DashMap<EntityId, VersionedState>>,
}

impl SyncManager {
    /// Create a new sync manager.
    pub fn new() -> Self {
        Self {
            states: Arc::new(DashMap::new()),
        }
    }

    /// Initialize entity state.
    pub fn init_state(&self, entity_id: EntityId, initial_state: serde_json::Value) -> Result<u64> {
        let state = VersionedState::new(initial_state);
        let version = state.version;
        self.states.insert(entity_id, state);
        Ok(version)
    }

    /// Get current state.
    pub fn get_state(&self, entity_id: &EntityId) -> Result<(serde_json::Value, u64)> {
        let state = self.states
            .get(entity_id)
            .ok_or_else(|| StreamError::generic("Entity not found"))?;
        Ok((state.state.clone(), state.version))
    }

    /// Apply an operation.
    pub fn apply_operation(
        &self,
        entity_id: &EntityId,
        client_id: ClientId,
        operation: Operation,
        base_version: u64,
    ) -> Result<(u64, Vec<Operation>)> {
        let mut state = self.states
            .get_mut(entity_id)
            .ok_or_else(|| StreamError::generic("Entity not found"))?;

        // Check if we need to transform
        if base_version < state.version {
            // Get operations since base version
            let pending_ops = state.get_operations_since(base_version);

            // Transform the incoming operation against pending operations
            let mut transformed_op = operation;
            for (_, _, pending_op) in &pending_ops {
                transformed_op = transformed_op.transform(pending_op, true)?;
            }

            // Apply transformed operation
            let new_version = state.apply_operation(client_id, transformed_op.clone())?;

            // Return transformed operations that need to be sent to other clients
            Ok((new_version, vec![transformed_op]))
        } else if base_version == state.version {
            // No transformation needed
            let new_version = state.apply_operation(client_id, operation.clone())?;
            Ok((new_version, vec![operation]))
        } else {
            // Base version is ahead of current version - conflict
            Err(StreamError::sync_conflict(format!(
                "Base version {} is ahead of current version {}",
                base_version, state.version
            )))
        }
    }

    /// Handle sync message.
    pub fn handle_sync_message(
        &self,
        entity_id: &EntityId,
        client_id: ClientId,
        message: SyncMessage,
    ) -> Result<Option<SyncMessage>> {
        match message {
            SyncMessage::RequestState { .. } => {
                let (state, version) = self.get_state(entity_id)?;
                Ok(Some(SyncMessage::StateResponse {
                    entity_type: "unknown".to_string(),
                    entity_id: entity_id.clone(),
                    state,
                    version,
                }))
            }

            SyncMessage::Operation {
                operation,
                version: base_version,
                ..
            } => {
                let op: Operation = serde_json::from_value(operation)
                    .map_err(|e| StreamError::invalid_message(format!("Invalid operation: {}", e)))?;

                match self.apply_operation(entity_id, client_id, op, base_version) {
                    Ok((new_version, _)) => {
                        Ok(Some(SyncMessage::Ack {
                            entity_id: entity_id.clone(),
                            version: new_version,
                        }))
                    }
                    Err(StreamError::SyncConflict(_)) => {
                        let state = self.states.get(entity_id).unwrap();
                        Ok(Some(SyncMessage::Conflict {
                            entity_id: entity_id.clone(),
                            local_version: base_version,
                            remote_version: state.version,
                        }))
                    }
                    Err(e) => Err(e),
                }
            }

            _ => Ok(None),
        }
    }

    /// Get entity count.
    pub fn entity_count(&self) -> usize {
        self.states.len()
    }

    /// Remove entity state.
    pub fn remove_state(&self, entity_id: &EntityId) -> Result<()> {
        self.states.remove(entity_id);
        info!("Removed state for entity: {}", entity_id);
        Ok(())
    }

    /// Cleanup old entities.
    pub fn cleanup_old_entities(&self, max_age: std::time::Duration) -> usize {
        let threshold_ms = max_age.as_millis() as u64;
        let current = current_timestamp();
        let mut removed = 0;

        self.states.retain(|entity_id, state| {
            if (current - state.last_update) > threshold_ms {
                info!("Removing old entity: {}", entity_id);
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Last write wins
    LastWriteWins,
    /// First write wins
    FirstWriteWins,
    /// Manual resolution required
    Manual,
    /// Operational transform
    OperationalTransform,
}

/// Conflict resolver for handling synchronization conflicts.
pub struct ConflictResolver {
    strategy: ConflictStrategy,
}

impl ConflictResolver {
    /// Create a new conflict resolver.
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }

    /// Resolve a conflict between two operations.
    pub fn resolve(
        &self,
        local_op: &Operation,
        remote_op: &Operation,
        local_timestamp: Timestamp,
        remote_timestamp: Timestamp,
    ) -> Result<Operation> {
        match self.strategy {
            ConflictStrategy::LastWriteWins => {
                if remote_timestamp > local_timestamp {
                    Ok(remote_op.clone())
                } else {
                    Ok(local_op.clone())
                }
            }
            ConflictStrategy::FirstWriteWins => {
                if local_timestamp < remote_timestamp {
                    Ok(local_op.clone())
                } else {
                    Ok(remote_op.clone())
                }
            }
            ConflictStrategy::OperationalTransform => {
                local_op.transform(remote_op, true)
            }
            ConflictStrategy::Manual => {
                Err(StreamError::sync_conflict("Manual resolution required"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_sync_manager_init() {
        let manager = SyncManager::new();
        let entity_id = "entity-1".to_string();
        let initial_state = serde_json::json!({"value": 0});

        let version = manager.init_state(entity_id.clone(), initial_state).unwrap();
        assert_eq!(version, 0);

        let (state, ver) = manager.get_state(&entity_id).unwrap();
        assert_eq!(ver, 0);
        assert_eq!(state["value"], 0);
    }

    #[test]
    fn test_apply_operation() {
        let manager = SyncManager::new();
        let entity_id = "entity-1".to_string();
        let initial_state = serde_json::json!({"value": 0});
        manager.init_state(entity_id.clone(), initial_state).unwrap();

        let client_id = Uuid::new_v4();
        let operation = Operation::Update {
            path: "value".to_string(),
            value: serde_json::json!(10),
        };

        let (new_version, _) = manager
            .apply_operation(&entity_id, client_id, operation, 0)
            .unwrap();

        assert_eq!(new_version, 1);

        let (state, _) = manager.get_state(&entity_id).unwrap();
        assert_eq!(state["value"], 10);
    }

    #[test]
    fn test_operation_transform_insert_insert() {
        let op1 = Operation::Insert {
            position: 5,
            value: serde_json::json!("a"),
        };
        let op2 = Operation::Insert {
            position: 3,
            value: serde_json::json!("b"),
        };

        let transformed = op1.transform(&op2, false).unwrap();
        if let Operation::Insert { position, .. } = transformed {
            assert_eq!(position, 6); // Position shifted by 1
        } else {
            panic!("Expected Insert operation");
        }
    }

    #[test]
    fn test_conflict_resolver() {
        let resolver = ConflictResolver::new(ConflictStrategy::LastWriteWins);
        let op1 = Operation::Update {
            path: "value".to_string(),
            value: serde_json::json!(1),
        };
        let op2 = Operation::Update {
            path: "value".to_string(),
            value: serde_json::json!(2),
        };

        let result = resolver.resolve(&op1, &op2, 100, 200).unwrap();
        if let Operation::Update { value, .. } = result {
            assert_eq!(value, 2); // op2 wins (later timestamp)
        }
    }
}
