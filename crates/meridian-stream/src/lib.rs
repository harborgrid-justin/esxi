//! # Meridian Stream
//!
//! Real-time data streaming and collaboration for the Meridian GIS Platform.
//!
//! This crate provides WebSocket-based streaming infrastructure for real-time
//! GIS applications, including:
//!
//! - **WebSocket Server & Client**: Production-ready WebSocket server and client
//!   implementations with automatic reconnection, ping/pong keepalive, and
//!   connection lifecycle management.
//!
//! - **Pub/Sub Channels**: High-performance publish/subscribe messaging system
//!   using Tokio broadcast channels and concurrent hash maps for efficient
//!   message distribution.
//!
//! - **Collaboration Rooms**: Multi-user collaboration spaces with participant
//!   tracking, permissions, and state synchronization.
//!
//! - **Real-time Sync**: Operational transform-based synchronization with
//!   conflict resolution for concurrent editing.
//!
//! - **Viewport Tracking**: Spatial awareness for efficient data streaming
//!   based on user viewport and zoom level.
//!
//! ## Features
//!
//! - **Concurrent**: Built on Tokio for efficient async I/O
//! - **Scalable**: Support for thousands of concurrent connections
//! - **Type-safe**: Strongly-typed message system with serde
//! - **GIS-aware**: Specialized message types for geospatial features
//! - **Production-ready**: Comprehensive error handling and logging
//!
//! ## Quick Start
//!
//! ### Server Example
//!
//! ```rust,no_run
//! use meridian_stream::server::{ServerBuilder, StreamServer};
//! use std::net::SocketAddr;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()?;
//!     let server = ServerBuilder::new(addr)
//!         .max_connections(5000)
//!         .build();
//!
//!     Arc::new(server).run().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Client Example
//!
//! ```rust,no_run
//! use meridian_stream::client::{ClientBuilder, StreamClient};
//! use meridian_stream::messages::StreamMessage;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = ClientBuilder::new("ws://localhost:8080")
//!         .auto_reconnect(true)
//!         .build();
//!
//!     client.connect().await?;
//!
//!     // Send a message
//!     client.send(StreamMessage::ping()).await?;
//!
//!     // Receive a message
//!     let msg = client.recv().await?;
//!     println!("Received: {:?}", msg);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Channel (Pub/Sub) Example
//!
//! ```rust
//! use meridian_stream::channel::ChannelManager;
//! use meridian_stream::messages::StreamMessage;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = ChannelManager::new();
//!     let channel_id = "layer:buildings".to_string();
//!     let client_id = Uuid::new_v4();
//!
//!     // Subscribe to a channel
//!     let mut subscription = manager.subscribe(client_id, channel_id.clone())?;
//!
//!     // Publish a message
//!     manager.publish(&channel_id, StreamMessage::ping())?;
//!
//!     // Receive the message
//!     let msg = subscription.recv().await?;
//!     println!("Received: {:?}", msg);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Room (Collaboration) Example
//!
//! ```rust
//! use meridian_stream::channel::ChannelManager;
//! use meridian_stream::room::{RoomManager, RoomConfig};
//! use std::sync::Arc;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let channels = Arc::new(ChannelManager::new());
//!     let rooms = RoomManager::new(channels);
//!
//!     // Create a room
//!     let room_id = "project-123".to_string();
//!     rooms.create_room(room_id.clone(), "My Project".to_string(), RoomConfig::default())?;
//!
//!     // Join the room
//!     let client_id = Uuid::new_v4();
//!     let state = rooms.join_room(&room_id, client_id, None)?;
//!     println!("Joined room: {:?}", state);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The streaming system is built on several key components:
//!
//! - **Messages**: Strongly-typed message definitions for all streaming events
//! - **Channels**: Pub/sub system for topic-based message distribution
//! - **Server/Client**: WebSocket infrastructure for bidirectional communication
//! - **Rooms**: Collaboration spaces with participant management
//! - **Sync**: Operational transforms for conflict-free concurrent editing
//! - **Viewport**: Spatial indexing for efficient viewport-based subscriptions
//! - **Handlers**: Message routing and business logic

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod channel;
pub mod client;
pub mod error;
pub mod handlers;
pub mod messages;
pub mod room;
pub mod server;
pub mod sync;
pub mod viewport;

// Re-export commonly used types
pub use channel::{ChannelManager, Subscription};
pub use client::{ClientBuilder, ClientConfig, ClientState, StreamClient};
pub use error::{Result, StreamError};
pub use handlers::MessageHandler;
pub use messages::{
    ChannelId, ClientId, FeatureId, FeatureOperation, FeatureUpdateMessage, LayerId,
    LayerOperation, LayerUpdateMessage, PresenceStatus, PresenceUpdateMessage, RoomId,
    RoomMessage, StreamMessage, SubscribeMessage, SyncMessage, UnsubscribeMessage,
    ViewportUpdateMessage,
};
pub use room::{Participant, ParticipantPermissions, RoomConfig, RoomManager, RoomState};
pub use server::{ServerBuilder, ServerConfig, StreamServer};
pub use sync::{ConflictResolver, ConflictStrategy, Operation, SyncManager};
pub use viewport::{Bounds, Point, Viewport, ViewportManager, ViewportStats};

/// Version of the meridian-stream crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default WebSocket port for streaming.
pub const DEFAULT_WS_PORT: u16 = 8080;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_WS_PORT, 8080);
    }
}
