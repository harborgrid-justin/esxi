//! # Meridian Collaboration Engine
//!
//! **Enterprise-grade real-time collaboration engine with CRDTs and Operational Transform**
//!
//! This crate provides comprehensive infrastructure for building collaborative applications
//! with support for multiple synchronization strategies, conflict resolution, and real-time
//! presence tracking.
//!
//! ## Features
//!
//! ### Conflict-Free Replicated Data Types (CRDTs)
//!
//! State-based CRDTs for distributed state synchronization without coordination:
//!
//! - **LWW-Register**: Last-Write-Wins register for single values
//! - **G-Counter**: Grow-only counter (increment only)
//! - **PN-Counter**: Positive-Negative counter (increment/decrement)
//! - **G-Set**: Grow-only set (add-only)
//! - **OR-Set**: Observed-Remove set (add and remove)
//! - **LWW-Map**: Last-Write-Wins map for key-value state
//! - **RGA**: Replicated Growable Array for collaborative text editing
//!
//! ### Operational Transform (OT)
//!
//! Industry-standard OT for real-time text collaboration:
//!
//! - Operation composition and transformation
//! - Cursor and selection transformation
//! - Undo/redo support
//! - Convergence guarantees (TP1, TP2 properties)
//!
//! ### Collaboration Features
//!
//! - **Presence Tracking**: Real-time user cursors and selections
//! - **Session Management**: Multi-user session coordination
//! - **Synchronization**: Efficient delta-based sync protocols
//! - **Conflict Resolution**: Automatic and manual conflict resolution
//! - **Version History**: Complete version tracking and time travel
//!
//! ## Quick Start
//!
//! ### Using CRDTs
//!
//! ```rust
//! use meridian_collaboration::crdt::{ReplicaId, LWWRegister, GCounter, ORSet};
//!
//! // LWW Register for simple values
//! let replica = ReplicaId::new();
//! let mut register = LWWRegister::new(42, replica);
//! register.set(100);
//! assert_eq!(*register.get(), 100);
//!
//! // G-Counter for distributed counting
//! let mut counter = GCounter::new(replica);
//! counter.increment_by(10);
//! assert_eq!(counter.value(), 10);
//!
//! // OR-Set for collaborative collections
//! let mut set = ORSet::new(replica);
//! set.insert("item1".to_string());
//! set.insert("item2".to_string());
//! assert_eq!(set.len(), 2);
//! ```
//!
//! ### Using Operational Transform
//!
//! ```rust
//! use meridian_collaboration::ot::Operation;
//!
//! // Create and apply operations
//! let mut op = Operation::new();
//! op.retain(5).insert(" World");
//!
//! let result = op.apply("Hello").unwrap();
//! assert_eq!(result, "Hello World");
//!
//! // Transform concurrent operations
//! use meridian_collaboration::ot::transform;
//!
//! let mut op1 = Operation::new();
//! op1.retain(5).insert("!");
//!
//! let mut op2 = Operation::new();
//! op2.retain(0).insert("Hi, ");
//!
//! let op1_prime = transform(&op1, &op2).unwrap();
//! // op1_prime can now be applied after op2
//! ```
//!
//! ### Collaborative Text Editing
//!
//! ```rust
//! use meridian_collaboration::crdt::{ReplicaId, RGA};
//!
//! let replica = ReplicaId::new();
//! let mut rga = RGA::new(replica);
//!
//! rga.insert_str(0, "Hello World");
//! assert_eq!(rga.to_string(), "Hello World");
//!
//! rga.delete_range(5, 11); // Remove " World"
//! assert_eq!(rga.to_string(), "Hello");
//! ```
//!
//! ### Collaboration Sessions
//!
//! ```rust
//! use meridian_collaboration::session::{CollaborationSession, SessionManager};
//! use meridian_collaboration::presence::UserPresence;
//! use meridian_collaboration::crdt::ReplicaId;
//! use meridian_collaboration::ot::Operation;
//!
//! // Create a session
//! let mut session = CollaborationSession::new(
//!     "doc-123".to_string(),
//!     "Initial content".to_string()
//! );
//!
//! // Add a user
//! let replica_id = ReplicaId::new();
//! let presence = UserPresence::new(
//!     "user-1".to_string(),
//!     replica_id,
//!     "Alice".to_string(),
//!     "#FF0000".to_string()
//! );
//! session.add_user(presence);
//!
//! // Apply an operation
//! let mut op = Operation::new();
//! op.retain(15).insert(" - Updated");
//! session.apply_operation(op, replica_id).unwrap();
//!
//! assert_eq!(session.get_content(), "Initial content - Updated");
//! ```
//!
//! ## Architecture
//!
//! ### CRDT Layer
//!
//! The CRDT layer provides eventual consistency without coordination. All CRDT types
//! implement the `CvRDT` trait for state-based replication with merge semantics.
//!
//! ### OT Layer
//!
//! The Operational Transform layer provides real-time consistency with immediate
//! operation application and transformation. Ideal for low-latency text editing.
//!
//! ### Hybrid Approach
//!
//! For optimal performance, combine both:
//! - Use OT for real-time text editing
//! - Use CRDTs for metadata, presence, and auxiliary state
//!
//! ## Performance
//!
//! - **CRDT operations**: O(1) for most operations
//! - **OT transform**: O(n) where n is operation length
//! - **Delta sync**: Efficient incremental updates
//! - **Snapshot compression**: Periodic state snapshots reduce history size
//!
//! ## Safety and Correctness
//!
//! - **Strong convergence**: All replicas converge to identical state
//! - **Causal consistency**: Operations respect causal ordering
//! - **Conflict-free**: Automatic deterministic conflict resolution
//! - **Memory safe**: Pure Rust with no unsafe code
//!
//! ## Use Cases
//!
//! - **Collaborative text editors**: Google Docs-style editing
//! - **Real-time whiteboards**: Shared drawing and diagramming
//! - **Collaborative spreadsheets**: Multi-user data editing
//! - **Project management**: Shared task lists and kanban boards
//! - **Chat applications**: Message history with CRDT sets
//! - **Distributed databases**: Eventually consistent data structures
//!
//! ## Advanced Features
//!
//! ### Custom Conflict Resolution
//!
//! ```rust
//! use meridian_collaboration::conflict::{ConflictDetector, ConflictResolver, ResolutionStrategy};
//!
//! let mut detector = ConflictDetector::new();
//! let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins);
//! ```
//!
//! ### Version History and Time Travel
//!
//! ```rust
//! use meridian_collaboration::history::{VersionHistory, VersionId};
//! use meridian_collaboration::crdt::ReplicaId;
//! use meridian_collaboration::ot::Operation;
//!
//! let mut history = VersionHistory::new("Initial".to_string());
//!
//! let mut op = Operation::new();
//! op.retain(7).insert(" content");
//! history.add_version(op, ReplicaId::new(), Some("Add content".to_string()));
//!
//! // Time travel to any version
//! let content_v1 = history.get_content_at_version(VersionId(1)).unwrap();
//! ```
//!
//! ### Efficient Synchronization
//!
//! ```rust
//! use meridian_collaboration::sync::{ClientSyncState, ServerSyncState};
//! use meridian_collaboration::crdt::ReplicaId;
//!
//! let client_id = ReplicaId::new();
//! let client = ClientSyncState::new(client_id, "Initial".to_string());
//! let server = ServerSyncState::new("Initial".to_string());
//! ```
//!
//! ## License
//!
//! Proprietary - Meridian Enterprise Platform

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod crdt;
pub mod ot;
pub mod presence;
pub mod session;
pub mod sync;
pub mod conflict;
pub mod history;

// Re-export commonly used types
pub use crdt::{
    CvRDT, CrdtValue, ReplicaId, VersionVector,
    LWWRegister, GCounter, PNCounter, GSet, ORSet, LWWMap, RGA,
};

pub use ot::{
    Operation, OpComponent, OtError,
    transform, compose,
};

pub use presence::{
    UserPresence, PresenceStatus, CursorPosition, Selection, PresenceManager,
};

pub use session::{
    CollaborationSession, SessionManager, SessionId,
};

pub use sync::{
    SyncMessage, ClientSyncState, ServerSyncState, SyncProtocol,
};

pub use conflict::{
    ConflictType, ConflictDetector, ConflictResolver, ResolutionStrategy,
};

pub use history::{
    VersionHistory, VersionId, Snapshot, HistoryEntry,
};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Get version information
pub fn version() -> &'static str {
    VERSION
}

/// Prelude module for convenient imports
pub mod prelude {
    //! Prelude module containing commonly used types and traits
    //!
    //! ```rust
    //! use meridian_collaboration::prelude::*;
    //! ```

    pub use crate::crdt::{
        CvRDT, CrdtValue, ReplicaId, VersionVector,
        LWWRegister, GCounter, PNCounter, GSet, ORSet, LWWMap, RGA,
    };

    pub use crate::ot::{
        Operation, OpComponent,
        transform, compose,
    };

    pub use crate::presence::{
        UserPresence, PresenceManager, CursorPosition, Selection,
    };

    pub use crate::session::{
        CollaborationSession, SessionManager,
    };
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_collaboration_workflow() {
        // Create two replicas
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        // Create session
        let mut session = session::CollaborationSession::new(
            "test-doc".to_string(),
            "Hello".to_string(),
        );

        // Add users
        let user1 = presence::UserPresence::new(
            "user1".to_string(),
            replica1,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );

        let user2 = presence::UserPresence::new(
            "user2".to_string(),
            replica2,
            "Bob".to_string(),
            "#00FF00".to_string(),
        );

        session.add_user(user1);
        session.add_user(user2);

        // Apply operations
        let mut op1 = ot::Operation::new();
        op1.retain(5).insert(" World");
        session.apply_operation(op1, replica1).unwrap();

        assert_eq!(session.get_content(), "Hello World");
        assert_eq!(session.version, 1);
        assert_eq!(session.presence.active_count(), 2);
    }

    #[test]
    fn test_crdt_convergence() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        // Two counters that will be merged
        let mut counter1 = crdt::GCounter::new(replica1);
        let mut counter2 = crdt::GCounter::new(replica2);

        counter1.increment_by(5);
        counter2.increment_by(10);

        counter1.merge(&counter2);
        counter2.merge(&counter1);

        // Should converge
        assert_eq!(counter1.value(), 15);
        assert_eq!(counter2.value(), 15);
    }

    #[test]
    fn test_ot_convergence() {
        // Concurrent operations should converge
        let mut op1 = ot::Operation::new();
        op1.insert("A");

        let mut op2 = ot::Operation::new();
        op2.insert("B");

        let op1_prime = ot::transform(&op1, &op2).unwrap();
        let op2_prime = ot::transform(&op2, &op1).unwrap();

        // Apply in both orders
        let result1 = op2.apply("").unwrap();
        let result1 = op1_prime.apply(&result1).unwrap();

        let result2 = op1.apply("").unwrap();
        let result2 = op2_prime.apply(&result2).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_history_and_undo() {
        let mut history = history::VersionHistory::new("Hello".to_string());
        let replica = ReplicaId::new();

        let mut op = ot::Operation::new();
        op.retain(5).insert(" World");
        history.add_version(op, replica, None);

        assert_eq!(history.get_current_content().unwrap(), "Hello World");

        let undo_op = history.undo_to_version(history::VersionId(0)).unwrap();
        let reverted = undo_op.apply("Hello World").unwrap();

        assert_eq!(reverted, "Hello");
    }

    #[test]
    fn test_rga_text_editing() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut rga1 = crdt::RGA::new(replica1);
        let mut rga2 = crdt::RGA::new(replica2);

        rga1.insert_str(0, "Hello");
        rga2.merge(&rga1);

        rga1.insert_str(5, " World");
        rga2.insert_str(0, "Hi, ");

        // Merge changes
        rga1.merge(&rga2);
        rga2.merge(&rga1);

        // Should converge
        assert_eq!(rga1.to_string(), rga2.to_string());
    }

    #[test]
    fn test_presence_tracking() {
        let mut manager = presence::PresenceManager::new();

        let replica = ReplicaId::new();
        let mut user = presence::UserPresence::new(
            "user1".to_string(),
            replica,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );

        user.set_cursor(presence::CursorPosition::new(10));
        user.set_selection(presence::Selection::new(5, 15));

        manager.update_user(user);

        assert_eq!(manager.active_count(), 1);

        // Transform through operation
        let mut op = ot::Operation::new();
        op.retain(3).insert("XXX");

        manager.transform_all_through_op(&op);

        let user = manager.get_user("user1").unwrap();
        assert_eq!(user.cursor.unwrap().offset, 13); // Cursor moved by 3
    }
}
