//! # Meridian Events
//!
//! A comprehensive event sourcing and CQRS (Command Query Responsibility Segregation) system
//! for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **Event Store**: Append-only event log with optimistic concurrency control
//! - **Event Versioning**: Schema evolution support with semantic versioning
//! - **Aggregate Root Pattern**: Domain-driven design aggregates with event sourcing
//! - **Command Handlers**: Command validation and execution with middleware
//! - **Event Projections**: Read model projections for CQRS query side
//! - **Snapshotting**: Performance optimization through aggregate snapshots
//! - **Event Replay**: Replay and reconstruct aggregates from event history
//! - **Saga Pattern**: Long-running distributed transactions with compensation
//! - **Event Bus**: Publish-subscribe pattern for event distribution
//! - **Idempotency**: Exactly-once processing with deduplication
//! - **Event Archival**: Archive old events and compact event streams
//! - **Causation Tracking**: Track event causation and correlation across boundaries
//!
//! ## Architecture
//!
//! The crate follows Domain-Driven Design (DDD) and Event Sourcing principles:
//!
//! ```text
//! ┌─────────────┐
//! │  Commands   │──┐
//! └─────────────┘  │
//!                  ├──> ┌──────────────┐      ┌─────────────┐
//!                  │    │ Command      │      │ Aggregate   │
//!                  └───>│ Handlers     │─────>│ Roots       │
//!                       └──────────────┘      └──────┬──────┘
//!                                                    │
//!                                                    │ Events
//!                                                    ▼
//!                                             ┌─────────────┐
//!                                             │ Event Store │
//!                                             └──────┬──────┘
//!                                                    │
//!                                  ┌─────────────────┼─────────────────┐
//!                                  │                 │                 │
//!                                  ▼                 ▼                 ▼
//!                           ┌────────────┐   ┌────────────┐   ┌────────────┐
//!                           │Projections │   │Event Bus   │   │  Sagas     │
//!                           └─────┬──────┘   └────────────┘   └────────────┘
//!                                 │
//!                                 ▼
//!                           ┌────────────┐
//!                           │Read Models │
//!                           └────────────┘
//! ```
//!
//! ## Usage
//!
//! ### Basic Event Sourcing
//!
//! ```rust,no_run
//! use meridian_events::prelude::*;
//!
//! // Define your events
//! #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct UserCreated {
//!     user_id: String,
//!     name: String,
//! }
//!
//! // Create an event store
//! let event_store = InMemoryEventStore::new();
//!
//! // Define your aggregate
//! // ... (implement AggregateRoot trait)
//! ```
//!
//! ### CQRS with Projections
//!
//! ```rust,no_run
//! use meridian_events::prelude::*;
//!
//! // Create a projection
//! let projection = ProjectionBuilder::new("user-list")
//!     .build(|event| {
//!         // Process event and update read model
//!         Ok(())
//!     });
//! ```
//!
//! ### Saga for Distributed Transactions
//!
//! ```rust,no_run
//! use meridian_events::prelude::*;
//!
//! // Create a saga
//! let mut saga = Saga::new(CorrelationId::new());
//!
//! // Add steps
//! // ... (add saga steps)
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// Core modules
pub mod error;
pub mod event;
pub mod store;

// DDD and Event Sourcing
pub mod aggregate;
pub mod command;
pub mod projection;

// Performance and optimization
pub mod snapshot;
pub mod replay;
pub mod archival;

// Distributed patterns
pub mod saga;
pub mod bus;
pub mod causation;
pub mod idempotency;

/// Prelude module for convenient imports.
pub mod prelude {
    //! Commonly used types and traits.

    // Error types
    pub use crate::error::{EventError, Result};

    // Event types
    pub use crate::event::{
        DomainEvent, EventEnvelope, EventId, EventMetadata, EventStream, EventUpgrader,
        EventUpgraderRegistry, EventVersion, StoredEvent, StreamId,
    };

    // Store types
    pub use crate::store::{
        AppendResult, EventStore, ExpectedVersion, InMemoryEventStore, ReadDirection,
        ReadOptions,
    };

    #[cfg(feature = "rocksdb-backend")]
    pub use crate::store::rocksdb_store::RocksDbEventStore;

    // Aggregate types
    pub use crate::aggregate::{
        Aggregate, AggregateId, AggregateRepository, AggregateRoot, EventSourcedRepository,
        SnapshotRepository,
    };

    // Command types
    pub use crate::command::{
        Command, CommandDispatcher, CommandHandler, CommandMiddleware, CommandPipeline,
        CommandResult, LoggingMiddleware, RangeValidator, StringLengthValidator,
        ValidationMiddleware, ValidationRules, Validator,
    };

    // Projection types
    pub use crate::projection::{
        BaseProjection, CatchUpSubscription, InMemoryProjectionState, InMemoryReadModel,
        LiveSubscription, Projection, ProjectionBuilder, ProjectionManager, ReadModel,
        TypedProjection,
    };

    // Snapshot types
    pub use crate::snapshot::{
        AlwaysStrategy, FrequencyStrategy, InMemorySnapshotStore, NeverStrategy, Snapshot,
        SnapshotManager, SnapshotMetadata, SnapshotStore, SnapshotStrategy,
    };

    // Replay types
    pub use crate::replay::{
        DebugReplayHandler, ReplayHandler, ReplayOptions, ReplayResult, ReplayService,
        ReplayStatistics, StatisticsReplayHandler,
    };

    // Saga types
    pub use crate::saga::{
        ProcessManager, Saga, SagaId, SagaManager, SagaMetadata, SagaState, SagaStep,
        SimpleSagaStep, StepResult,
    };

    // Bus types
    pub use crate::bus::{
        EventBus, EventDispatcher, EventRouter, EventSubscriber, FilteredEventBus,
        SimpleSubscriber, SubscriberId, Subscription,
    };

    // Causation types
    pub use crate::causation::{
        CausationChain, CausationContext, CausationId, CausationTracker, CorrelationId,
    };

    // Idempotency types
    pub use crate::idempotency::{
        DeduplicationTracker, IdempotencyKey, IdempotencyManager, IdempotencyStore,
        InMemoryIdempotencyStore, OperationRecord, OperationStatus,
    };

    // Archival types
    pub use crate::archival::{
        ArchivalDestination, ArchivalMetadata, ArchivalPolicy, ArchivalService, ArchiveManager,
        CompactionResult, CompactionService, CompactionStrategy, FileArchive, SnapshotCompaction,
        TimeBasedPolicy, VersionBasedPolicy,
    };
}

// Re-export commonly used external types
pub use async_trait::async_trait;
pub use chrono;
pub use serde;
pub use uuid;

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_crate_version() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.5");
    }

    #[test]
    fn test_prelude_imports() {
        // Verify that key types are accessible from prelude
        let _event_id = EventId::new();
        let _stream_id = StreamId::new("test");
        let _aggregate_id = AggregateId::new("test");
        let _causation_id = CausationId::new();
        let _correlation_id = CorrelationId::new();
    }
}
