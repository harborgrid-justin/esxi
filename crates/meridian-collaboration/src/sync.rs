//! # State Synchronization Protocol
//!
//! This module implements efficient synchronization protocols for collaborative editing.
//! It supports both full-state sync and delta-based sync for optimal bandwidth usage.

use crate::crdt::{ReplicaId, VersionVector};
use crate::ot::{compose, transform, Operation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Synchronization message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Request current state
    StateRequest {
        session_id: String,
        client_version: u64,
    },

    /// Full state response
    StateResponse {
        session_id: String,
        content: String,
        version: u64,
        version_vector: VersionVector,
    },

    /// Delta sync request
    DeltaRequest {
        session_id: String,
        from_version: u64,
        version_vector: VersionVector,
    },

    /// Delta sync response
    DeltaResponse {
        session_id: String,
        operations: Vec<OperationMessage>,
        current_version: u64,
    },

    /// Client operation
    ClientOperation {
        session_id: String,
        operation: Operation,
        base_version: u64,
        client_id: ReplicaId,
    },

    /// Server acknowledgment
    ServerAck {
        session_id: String,
        version: u64,
        transformed_operation: Option<Operation>,
    },

    /// Sync heartbeat
    Heartbeat {
        session_id: String,
        client_id: ReplicaId,
        timestamp: DateTime<Utc>,
    },
}

/// Operation message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMessage {
    pub operation: Operation,
    pub version: u64,
    pub author: ReplicaId,
    pub timestamp: DateTime<Utc>,
}

/// Client-side synchronization state
#[derive(Debug, Clone)]
pub struct ClientSyncState {
    /// Client's replica ID
    pub client_id: ReplicaId,

    /// Current local content
    pub content: String,

    /// Local version
    pub local_version: u64,

    /// Server version (last known)
    pub server_version: u64,

    /// Operations waiting for server acknowledgment
    pending_ops: VecDeque<Operation>,

    /// Operations buffer for concurrent edits
    buffer: VecDeque<Operation>,

    /// Last sync timestamp
    pub last_sync: DateTime<Utc>,
}

impl ClientSyncState {
    /// Create a new client sync state
    pub fn new(client_id: ReplicaId, initial_content: String) -> Self {
        Self {
            client_id,
            content: initial_content,
            local_version: 0,
            server_version: 0,
            pending_ops: VecDeque::new(),
            buffer: VecDeque::new(),
            last_sync: Utc::now(),
        }
    }

    /// Apply a local operation
    pub fn apply_local_operation(&mut self, operation: Operation) -> Result<(), SyncError> {
        // Apply to local content
        self.content = operation
            .apply(&self.content)
            .map_err(|e| SyncError::OperationError(e.to_string()))?;

        // Compose with pending operations if possible
        if let Some(last_pending) = self.pending_ops.back_mut() {
            if let Ok(composed) = compose(last_pending, &operation) {
                *last_pending = composed;
            } else {
                self.pending_ops.push_back(operation);
            }
        } else {
            self.pending_ops.push_back(operation);
        }

        self.local_version += 1;

        Ok(())
    }

    /// Receive server acknowledgment
    pub fn receive_ack(
        &mut self,
        version: u64,
        transformed_op: Option<Operation>,
    ) -> Result<(), SyncError> {
        if !self.pending_ops.is_empty() {
            self.pending_ops.pop_front();
        }

        self.server_version = version;
        self.last_sync = Utc::now();

        // Apply transformed operation if provided
        if let Some(op) = transformed_op {
            self.content = op
                .apply(&self.content)
                .map_err(|e| SyncError::OperationError(e.to_string()))?;
        }

        Ok(())
    }

    /// Receive remote operation from server
    pub fn receive_remote_operation(&mut self, operation: Operation) -> Result<(), SyncError> {
        // Transform against pending operations
        let mut transformed = operation;

        for pending in &self.pending_ops {
            transformed = transform(&transformed, pending)
                .map_err(|e| SyncError::TransformError(e.to_string()))?;
        }

        // Apply transformed operation
        self.content = transformed
            .apply(&self.content)
            .map_err(|e| SyncError::OperationError(e.to_string()))?;

        self.server_version += 1;
        self.last_sync = Utc::now();

        Ok(())
    }

    /// Get operation to send to server
    pub fn get_pending_operation(&self) -> Option<&Operation> {
        self.pending_ops.front()
    }

    /// Check if there are pending operations
    pub fn has_pending_operations(&self) -> bool {
        !self.pending_ops.is_empty()
    }

    /// Get synchronization status
    pub fn sync_status(&self) -> SyncStatus {
        SyncStatus {
            local_version: self.local_version,
            server_version: self.server_version,
            pending_count: self.pending_ops.len(),
            is_synced: self.pending_ops.is_empty(),
            last_sync: self.last_sync,
        }
    }
}

/// Server-side synchronization state
#[derive(Debug, Clone)]
pub struct ServerSyncState {
    /// Current server content
    pub content: String,

    /// Current version
    pub version: u64,

    /// Operation history
    pub history: VecDeque<OperationMessage>,

    /// Maximum history size
    max_history: usize,

    /// Version vector for causal ordering
    pub version_vector: VersionVector,

    /// Client states
    client_states: HashMap<ReplicaId, ClientState>,
}

#[derive(Debug, Clone)]
struct ClientState {
    last_seen_version: u64,
    last_activity: DateTime<Utc>,
}

impl ServerSyncState {
    /// Create a new server sync state
    pub fn new(initial_content: String) -> Self {
        Self {
            content: initial_content,
            version: 0,
            history: VecDeque::new(),
            max_history: 1000,
            version_vector: VersionVector::new(),
            client_states: HashMap::new(),
        }
    }

    /// Set maximum history size
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        self.trim_history();
    }

    /// Apply a client operation
    pub fn apply_client_operation(
        &mut self,
        operation: Operation,
        client_id: ReplicaId,
        base_version: u64,
    ) -> Result<Operation, SyncError> {
        // Get operations since client's base version
        let missing_ops: Vec<_> = self
            .history
            .iter()
            .filter(|op| op.version > base_version)
            .collect();

        // Transform client operation against missing operations
        let mut transformed = operation;
        for op_msg in &missing_ops {
            transformed = transform(&transformed, &op_msg.operation)
                .map_err(|e| SyncError::TransformError(e.to_string()))?;
        }

        // Apply to server content
        self.content = transformed
            .apply(&self.content)
            .map_err(|e| SyncError::OperationError(e.to_string()))?;

        // Update version
        self.version += 1;
        self.version_vector.increment(client_id);

        // Record operation
        let op_msg = OperationMessage {
            operation: transformed.clone(),
            version: self.version,
            author: client_id,
            timestamp: Utc::now(),
        };

        self.history.push_back(op_msg);
        self.trim_history();

        // Update client state
        self.client_states.insert(
            client_id,
            ClientState {
                last_seen_version: self.version,
                last_activity: Utc::now(),
            },
        );

        Ok(transformed)
    }

    /// Get operations since a version
    pub fn get_operations_since(&self, version: u64) -> Vec<&OperationMessage> {
        self.history
            .iter()
            .filter(|op| op.version > version)
            .collect()
    }

    /// Get delta for synchronization
    pub fn get_delta(&self, from_version: u64) -> Vec<OperationMessage> {
        self.get_operations_since(from_version)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Trim history to maximum size
    fn trim_history(&mut self) {
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Clean up inactive clients
    pub fn cleanup_inactive_clients(&mut self, threshold: chrono::Duration) {
        let cutoff = Utc::now() - threshold;
        self.client_states
            .retain(|_, state| state.last_activity > cutoff);
    }

    /// Get active client count
    pub fn active_client_count(&self) -> usize {
        self.client_states.len()
    }
}

/// Synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub local_version: u64,
    pub server_version: u64,
    pub pending_count: usize,
    pub is_synced: bool,
    pub last_sync: DateTime<Utc>,
}

/// Synchronization errors
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Operation error: {0}")]
    OperationError(String),

    #[error("Transform error: {0}")]
    TransformError(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u64, actual: u64 },

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Client not found: {0}")]
    ClientNotFound(String),
}

/// Sync protocol handler
#[derive(Debug)]
pub struct SyncProtocol {
    client_state: Option<ClientSyncState>,
    server_state: Option<ServerSyncState>,
}

impl SyncProtocol {
    /// Create a new client-side sync protocol
    pub fn new_client(client_id: ReplicaId, initial_content: String) -> Self {
        Self {
            client_state: Some(ClientSyncState::new(client_id, initial_content)),
            server_state: None,
        }
    }

    /// Create a new server-side sync protocol
    pub fn new_server(initial_content: String) -> Self {
        Self {
            client_state: None,
            server_state: Some(ServerSyncState::new(initial_content)),
        }
    }

    /// Get client state
    pub fn client_state(&self) -> Option<&ClientSyncState> {
        self.client_state.as_ref()
    }

    /// Get mutable client state
    pub fn client_state_mut(&mut self) -> Option<&mut ClientSyncState> {
        self.client_state.as_mut()
    }

    /// Get server state
    pub fn server_state(&self) -> Option<&ServerSyncState> {
        self.server_state.as_ref()
    }

    /// Get mutable server state
    pub fn server_state_mut(&mut self) -> Option<&mut ServerSyncState> {
        self.server_state.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_sync_state() {
        let client_id = ReplicaId::new();
        let mut client = ClientSyncState::new(client_id, "Hello".to_string());

        let mut op = Operation::new();
        op.retain(5).insert(" World");

        client.apply_local_operation(op).unwrap();

        assert_eq!(client.content, "Hello World");
        assert_eq!(client.local_version, 1);
        assert!(client.has_pending_operations());
    }

    #[test]
    fn test_server_sync_state() {
        let mut server = ServerSyncState::new("Hello".to_string());

        let mut op = Operation::new();
        op.retain(5).insert(" World");

        let client_id = ReplicaId::new();
        let transformed = server.apply_client_operation(op, client_id, 0).unwrap();

        assert_eq!(server.content, "Hello World");
        assert_eq!(server.version, 1);
        assert_eq!(server.history.len(), 1);
    }

    #[test]
    fn test_client_server_sync() {
        let client_id = ReplicaId::new();
        let mut client = ClientSyncState::new(client_id, "Hello".to_string());
        let mut server = ServerSyncState::new("Hello".to_string());

        // Client creates operation
        let mut op = Operation::new();
        op.retain(5).insert(" World");
        client.apply_local_operation(op.clone()).unwrap();

        // Server applies operation
        let transformed = server.apply_client_operation(op, client_id, 0).unwrap();

        // Client receives acknowledgment
        client.receive_ack(1, None).unwrap();

        assert_eq!(client.content, "Hello World");
        assert_eq!(server.content, "Hello World");
        assert!(!client.has_pending_operations());
    }

    #[test]
    fn test_concurrent_operations() {
        let client1_id = ReplicaId::new();
        let client2_id = ReplicaId::new();

        let mut client1 = ClientSyncState::new(client1_id, "Hello".to_string());
        let mut client2 = ClientSyncState::new(client2_id, "Hello".to_string());
        let mut server = ServerSyncState::new("Hello".to_string());

        // Client 1 inserts at end
        let mut op1 = Operation::new();
        op1.retain(5).insert("!");
        client1.apply_local_operation(op1.clone()).unwrap();

        // Client 2 inserts at beginning
        let mut op2 = Operation::new();
        op2.insert("Hi, ");
        client2.apply_local_operation(op2.clone()).unwrap();

        // Server applies both operations
        server.apply_client_operation(op1, client1_id, 0).unwrap();
        server.apply_client_operation(op2, client2_id, 0).unwrap();

        assert!(server.content.contains("Hi,"));
        assert!(server.content.contains("Hello"));
        assert!(server.content.contains("!"));
    }

    #[test]
    fn test_sync_status() {
        let client_id = ReplicaId::new();
        let client = ClientSyncState::new(client_id, "Hello".to_string());

        let status = client.sync_status();
        assert_eq!(status.local_version, 0);
        assert_eq!(status.server_version, 0);
        assert!(status.is_synced);
    }

    #[test]
    fn test_delta_sync() {
        let mut server = ServerSyncState::new("".to_string());
        let client_id = ReplicaId::new();

        // Apply multiple operations
        for i in 0..5 {
            let mut op = Operation::new();
            op.retain(i).insert("X");
            server.apply_client_operation(op, client_id, i as u64).unwrap();
        }

        // Get delta from version 2
        let delta = server.get_delta(2);
        assert_eq!(delta.len(), 3);
    }

    #[test]
    fn test_history_trimming() {
        let mut server = ServerSyncState::new("".to_string());
        server.set_max_history(5);

        let client_id = ReplicaId::new();

        // Add more operations than max history
        for i in 0..10 {
            let mut op = Operation::new();
            op.retain(i).insert("X");
            server.apply_client_operation(op, client_id, i as u64).unwrap();
        }

        assert_eq!(server.history.len(), 5);
    }
}
