//! # Meridian Real-time Collaboration and Synchronization
//!
//! High-performance real-time collaboration system for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **Sub-100ms latency**: Optimized WebSocket communication with binary protocols
//! - **Horizontal scaling**: Redis-backed pub/sub for multi-instance deployments
//! - **CRDT synchronization**: Conflict-free replicated data types for geo data
//! - **Operational transformation**: Advanced conflict resolution
//! - **End-to-end encryption**: Optional E2EE for sensitive collaborative sessions
//! - **State recovery**: Automatic reconnection with state synchronization
//! - **Presence awareness**: Real-time user tracking and cursor sharing
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use meridian_realtime::{Server, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ServerConfig::default();
//!     let server = Server::new(config).await?;
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]

pub mod error;
pub mod server;
pub mod sync;
pub mod collaboration;
pub mod streaming;
pub mod pubsub;
pub mod protocol;
pub mod state;

// Re-exports
pub use error::{Error, Result};
pub use server::{Server, ServerConfig, Connection};
pub use sync::{CrdtMap, VectorClock, ConflictResolver};
pub use collaboration::{Presence, Cursor, Selection};
pub use protocol::{Message, BinaryProtocol};
pub use state::{Snapshot, Delta};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::server::{Server, ServerConfig, Connection, Room};
    pub use crate::sync::{CrdtMap, CrdtSet, VectorClock, ConflictResolver};
    pub use crate::collaboration::{Presence, Cursor, Selection, Annotation};
    pub use crate::streaming::{GpsStream, SensorStream, EventStream};
    pub use crate::pubsub::{Channel, Subscription};
    pub use crate::protocol::{Message, MessageType, BinaryProtocol};
    pub use crate::state::{Snapshot, Delta, StateManager};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default WebSocket port
pub const DEFAULT_WS_PORT: u16 = 9090;

/// Default Redis URL
pub const DEFAULT_REDIS_URL: &str = "redis://localhost:6379";

/// Maximum message size (10MB)
pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Default heartbeat interval (30 seconds)
pub const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Default client timeout (60 seconds)
pub const CLIENT_TIMEOUT_SECS: u64 = 60;

/// Maximum reconnection attempts
pub const MAX_RECONNECT_ATTEMPTS: u32 = 10;

/// Reconnection backoff base (milliseconds)
pub const RECONNECT_BACKOFF_MS: u64 = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_WS_PORT, 9090);
        assert!(MAX_MESSAGE_SIZE > 0);
        assert!(HEARTBEAT_INTERVAL_SECS > 0);
    }
}
