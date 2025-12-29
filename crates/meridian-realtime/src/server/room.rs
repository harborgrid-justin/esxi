//! Room/channel management for organizing connections

use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::protocol::Message;
use crate::server::ConnectionId;

/// Room ID type
pub type RoomId = String;

/// Room for grouping connections
pub struct Room {
    /// Room ID
    id: RoomId,

    /// Room name
    name: String,

    /// Connections in this room
    connections: Arc<DashMap<ConnectionId, mpsc::UnboundedSender<Message>>>,

    /// Room metadata
    metadata: Arc<parking_lot::RwLock<serde_json::Value>>,

    /// Maximum connections
    max_connections: usize,
}

impl Room {
    /// Create new room
    pub fn new(id: RoomId, name: String, max_connections: usize) -> Self {
        Self {
            id,
            name,
            connections: Arc::new(DashMap::new()),
            metadata: Arc::new(parking_lot::RwLock::new(serde_json::json!({}))),
            max_connections,
        }
    }

    /// Get room ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get room name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Add connection to room
    pub async fn add_connection(
        &self,
        conn_id: ConnectionId,
        tx: mpsc::UnboundedSender<Message>,
    ) -> Result<()> {
        if self.connections.len() >= self.max_connections {
            return Err(Error::Connection(format!(
                "Room {} is full (max: {})",
                self.id, self.max_connections
            )));
        }

        self.connections.insert(conn_id, tx);
        info!("Connection {} added to room {}", conn_id, self.id);

        // Broadcast join event
        self.broadcast_except(
            Message::new(
                crate::protocol::MessageType::UserJoined,
                conn_id.to_string().into_bytes(),
            ),
            conn_id,
        )
        .await;

        Ok(())
    }

    /// Remove connection from room
    pub async fn remove_connection(&self, conn_id: ConnectionId) {
        if self.connections.remove(&conn_id).is_some() {
            info!("Connection {} removed from room {}", conn_id, self.id);

            // Broadcast leave event
            self.broadcast_except(
                Message::new(
                    crate::protocol::MessageType::UserLeft,
                    conn_id.to_string().into_bytes(),
                ),
                conn_id,
            )
            .await;
        }
    }

    /// Broadcast message to all connections
    pub async fn broadcast(&self, message: Message) {
        let mut failed_connections = Vec::new();

        for entry in self.connections.iter() {
            let conn_id = *entry.key();
            let tx = entry.value();

            if tx.send(message.clone()).is_err() {
                failed_connections.push(conn_id);
            }
        }

        // Remove failed connections
        for conn_id in failed_connections {
            self.connections.remove(&conn_id);
            debug!("Removed failed connection {} from room {}", conn_id, self.id);
        }
    }

    /// Broadcast message to all connections except one
    pub async fn broadcast_except(&self, message: Message, except: ConnectionId) {
        let mut failed_connections = Vec::new();

        for entry in self.connections.iter() {
            let conn_id = *entry.key();
            if conn_id == except {
                continue;
            }

            let tx = entry.value();
            if tx.send(message.clone()).is_err() {
                failed_connections.push(conn_id);
            }
        }

        // Remove failed connections
        for conn_id in failed_connections {
            self.connections.remove(&conn_id);
            debug!("Removed failed connection {} from room {}", conn_id, self.id);
        }
    }

    /// Send message to specific connection
    pub async fn send_to(&self, conn_id: ConnectionId, message: Message) -> Result<()> {
        let tx = self
            .connections
            .get(&conn_id)
            .ok_or_else(|| Error::UserNotFound(conn_id.to_string()))?;

        tx.send(message)
            .map_err(|_| Error::ChannelClosed)?;

        Ok(())
    }

    /// Get all connection IDs
    pub fn connection_ids(&self) -> Vec<ConnectionId> {
        self.connections.iter().map(|e| *e.key()).collect()
    }

    /// Get room metadata
    pub fn metadata(&self) -> serde_json::Value {
        self.metadata.read().clone()
    }

    /// Set room metadata
    pub fn set_metadata(&self, metadata: serde_json::Value) {
        *self.metadata.write() = metadata;
    }

    /// Check if room is empty
    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}

/// Room manager
pub struct RoomManager {
    /// All rooms
    rooms: Arc<DashMap<RoomId, Arc<Room>>>,

    /// Maximum number of rooms
    max_rooms: usize,
}

impl RoomManager {
    /// Create new room manager
    pub fn new(max_rooms: usize) -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            max_rooms,
        }
    }

    /// Get room by ID
    pub fn get(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.get(room_id).map(|r| r.value().clone())
    }

    /// Get or create room
    pub fn get_or_create(&self, room_id: &str) -> Result<Arc<Room>> {
        if let Some(room) = self.get(room_id) {
            return Ok(room);
        }

        if self.rooms.len() >= self.max_rooms {
            return Err(Error::Internal(format!(
                "Maximum number of rooms ({}) reached",
                self.max_rooms
            )));
        }

        let room = Arc::new(Room::new(
            room_id.to_string(),
            room_id.to_string(),
            100, // Default max connections
        ));

        self.rooms.insert(room_id.to_string(), room.clone());
        info!("Created room: {}", room_id);

        Ok(room)
    }

    /// Create room with custom settings
    pub fn create_room(
        &self,
        room_id: RoomId,
        name: String,
        max_connections: usize,
    ) -> Result<Arc<Room>> {
        if self.rooms.contains_key(&room_id) {
            return Err(Error::Internal(format!("Room {} already exists", room_id)));
        }

        if self.rooms.len() >= self.max_rooms {
            return Err(Error::Internal(format!(
                "Maximum number of rooms ({}) reached",
                self.max_rooms
            )));
        }

        let room = Arc::new(Room::new(room_id.clone(), name, max_connections));
        self.rooms.insert(room_id.clone(), room.clone());
        info!("Created room: {}", room_id);

        Ok(room)
    }

    /// Remove room
    pub fn remove_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.remove(room_id).map(|(_, room)| {
            info!("Removed room: {}", room_id);
            room
        })
    }

    /// Remove empty rooms
    pub fn cleanup_empty_rooms(&self) {
        let empty_rooms: Vec<RoomId> = self
            .rooms
            .iter()
            .filter(|r| r.value().is_empty())
            .map(|r| r.key().clone())
            .collect();

        for room_id in empty_rooms {
            self.rooms.remove(&room_id);
            debug!("Cleaned up empty room: {}", room_id);
        }
    }

    /// Get all room IDs
    pub fn room_ids(&self) -> Vec<RoomId> {
        self.rooms.iter().map(|r| r.key().clone()).collect()
    }

    /// Get room count
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Get total connections across all rooms
    pub fn total_connections(&self) -> usize {
        self.rooms
            .iter()
            .map(|r| r.value().connection_count())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_creation() {
        let room = Room::new("test".to_string(), "Test Room".to_string(), 10);
        assert_eq!(room.id(), "test");
        assert_eq!(room.name(), "Test Room");
        assert_eq!(room.connection_count(), 0);
        assert!(room.is_empty());
    }

    #[test]
    fn test_room_manager() {
        let manager = RoomManager::new(100);
        assert_eq!(manager.room_count(), 0);

        let room = manager.get_or_create("test").unwrap();
        assert_eq!(room.id(), "test");
        assert_eq!(manager.room_count(), 1);

        let same_room = manager.get_or_create("test").unwrap();
        assert_eq!(manager.room_count(), 1);
        assert_eq!(room.id(), same_room.id());
    }

    #[tokio::test]
    async fn test_room_broadcast() {
        let room = Room::new("test".to_string(), "Test Room".to_string(), 10);
        let (tx, mut rx) = mpsc::unbounded_channel();

        let conn_id = Uuid::new_v4();
        room.add_connection(conn_id, tx).await.unwrap();

        // Should receive join event
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.msg_type, crate::protocol::MessageType::UserJoined);
    }
}
