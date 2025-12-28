//! Collaboration rooms for real-time multi-user sessions.

use crate::channel::ChannelManager;
use crate::error::{Result, StreamError};
use crate::messages::{ClientId, RoomId, RoomMessage, StreamMessage, Timestamp, current_timestamp};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Maximum number of participants per room.
const MAX_PARTICIPANTS_PER_ROOM: usize = 100;

/// Maximum number of rooms.
const MAX_ROOMS: usize = 1000;

/// Participant information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Client identifier
    pub client_id: ClientId,
    /// User identifier (optional)
    pub user_id: Option<String>,
    /// User display name
    pub user_name: Option<String>,
    /// Join timestamp
    pub joined_at: Timestamp,
    /// Last activity timestamp
    pub last_activity: Timestamp,
    /// User-specific data
    pub user_data: serde_json::Value,
    /// Participant permissions
    pub permissions: ParticipantPermissions,
}

/// Participant permissions in a room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPermissions {
    /// Can edit room state
    pub can_edit: bool,
    /// Can invite others
    pub can_invite: bool,
    /// Can remove participants
    pub can_remove: bool,
    /// Is room owner
    pub is_owner: bool,
}

impl Default for ParticipantPermissions {
    fn default() -> Self {
        Self {
            can_edit: true,
            can_invite: false,
            can_remove: false,
            is_owner: false,
        }
    }
}

/// Room configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    /// Maximum participants
    pub max_participants: usize,
    /// Require authentication
    pub require_auth: bool,
    /// Room is public
    pub is_public: bool,
    /// Auto-cleanup when empty
    pub auto_cleanup: bool,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            max_participants: MAX_PARTICIPANTS_PER_ROOM,
            require_auth: false,
            is_public: true,
            auto_cleanup: true,
        }
    }
}

/// Room state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomState {
    /// Room identifier
    pub room_id: RoomId,
    /// Room name
    pub name: String,
    /// Room description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
    /// Current state version
    pub version: u64,
    /// Room data (GIS layers, features, etc.)
    pub data: serde_json::Value,
    /// Room configuration
    pub config: RoomConfig,
}

/// Internal room state.
struct Room {
    /// Room state
    state: RoomState,
    /// Participants
    participants: DashMap<ClientId, Participant>,
    /// Associated channel ID
    channel_id: String,
}

impl Room {
    /// Create a new room.
    fn new(room_id: RoomId, name: String, config: RoomConfig) -> Self {
        let channel_id = format!("room:{}", room_id);
        Self {
            state: RoomState {
                room_id: room_id.clone(),
                name,
                description: None,
                created_at: current_timestamp(),
                updated_at: current_timestamp(),
                version: 0,
                data: serde_json::Value::Null,
                config,
            },
            participants: DashMap::new(),
            channel_id,
        }
    }

    /// Get the number of participants.
    fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Check if a client is a participant.
    fn has_participant(&self, client_id: &ClientId) -> bool {
        self.participants.contains_key(client_id)
    }

    /// Get participant.
    fn get_participant(&self, client_id: &ClientId) -> Option<Participant> {
        self.participants.get(client_id).map(|p| p.clone())
    }

    /// Add a participant.
    fn add_participant(&self, client_id: ClientId, user_data: Option<serde_json::Value>) -> Result<()> {
        if self.participant_count() >= self.state.config.max_participants {
            return Err(StreamError::generic("Room is full"));
        }

        let participant = Participant {
            client_id,
            user_id: None,
            user_name: None,
            joined_at: current_timestamp(),
            last_activity: current_timestamp(),
            user_data: user_data.unwrap_or(serde_json::Value::Null),
            permissions: if self.participants.is_empty() {
                // First participant becomes owner
                ParticipantPermissions {
                    can_edit: true,
                    can_invite: true,
                    can_remove: true,
                    is_owner: true,
                }
            } else {
                ParticipantPermissions::default()
            },
        };

        self.participants.insert(client_id, participant);
        Ok(())
    }

    /// Remove a participant.
    fn remove_participant(&self, client_id: &ClientId) -> Option<Participant> {
        self.participants.remove(client_id).map(|(_, p)| p)
    }

    /// Update participant activity.
    fn update_activity(&self, client_id: &ClientId) {
        if let Some(mut participant) = self.participants.get_mut(client_id) {
            participant.last_activity = current_timestamp();
        }
    }

    /// Update room state.
    fn update_state(&mut self, data: serde_json::Value) {
        self.state.data = data;
        self.state.version += 1;
        self.state.updated_at = current_timestamp();
    }
}

/// Room manager for handling collaboration rooms.
#[derive(Clone)]
pub struct RoomManager {
    /// Active rooms
    rooms: Arc<DashMap<RoomId, Room>>,
    /// Channel manager for pub/sub
    channels: Arc<ChannelManager>,
}

impl RoomManager {
    /// Create a new room manager.
    pub fn new(channels: Arc<ChannelManager>) -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            channels,
        }
    }

    /// Create a new room.
    pub fn create_room(&self, room_id: RoomId, name: String, config: RoomConfig) -> Result<RoomState> {
        if self.rooms.len() >= MAX_ROOMS {
            return Err(StreamError::generic("Maximum number of rooms exceeded"));
        }

        if self.rooms.contains_key(&room_id) {
            return Err(StreamError::generic("Room already exists"));
        }

        let room = Room::new(room_id.clone(), name, config);
        let state = room.state.clone();

        // Create associated channel
        self.channels.create_channel(room.channel_id.clone())?;

        self.rooms.insert(room_id.clone(), room);
        info!("Created room: {}", room_id);

        Ok(state)
    }

    /// Get room state.
    pub fn get_room_state(&self, room_id: &RoomId) -> Result<RoomState> {
        let room = self
            .rooms
            .get(room_id)
            .ok_or_else(|| StreamError::RoomNotFound(room_id.clone()))?;
        Ok(room.state.clone())
    }

    /// Join a room.
    pub fn join_room(
        &self,
        room_id: &RoomId,
        client_id: ClientId,
        user_data: Option<serde_json::Value>,
    ) -> Result<RoomState> {
        let room = self
            .rooms
            .get(room_id)
            .ok_or_else(|| StreamError::RoomNotFound(room_id.clone()))?;

        // Add participant
        room.add_participant(client_id, user_data.clone())?;

        // Subscribe to room channel
        self.channels.subscribe(client_id, room.channel_id.clone())?;

        // Broadcast join message
        let join_msg = StreamMessage::Room(RoomMessage::ParticipantJoined {
            room_id: room_id.clone(),
            client_id,
            user_data,
        });
        let _ = self.channels.publish(&room.channel_id, join_msg);

        info!("Client {} joined room {}", client_id, room_id);
        Ok(room.state.clone())
    }

    /// Leave a room.
    pub fn leave_room(&self, room_id: &RoomId, client_id: ClientId) -> Result<()> {
        let room = self
            .rooms
            .get(room_id)
            .ok_or_else(|| StreamError::RoomNotFound(room_id.clone()))?;

        // Remove participant
        room.remove_participant(&client_id);

        // Unsubscribe from room channel
        self.channels.unsubscribe(client_id, &room.channel_id)?;

        // Broadcast leave message
        let leave_msg = StreamMessage::Room(RoomMessage::ParticipantLeft {
            room_id: room_id.clone(),
            client_id,
        });
        let _ = self.channels.publish(&room.channel_id, leave_msg);

        info!("Client {} left room {}", client_id, room_id);

        // Auto-cleanup if enabled and empty
        if room.state.config.auto_cleanup && room.participant_count() == 0 {
            drop(room);
            self.delete_room(room_id)?;
        }

        Ok(())
    }

    /// Update room state.
    pub fn update_room_state(
        &self,
        room_id: &RoomId,
        client_id: ClientId,
        data: serde_json::Value,
    ) -> Result<u64> {
        let mut room = self
            .rooms
            .get_mut(room_id)
            .ok_or_else(|| StreamError::RoomNotFound(room_id.clone()))?;

        // Check permissions
        if let Some(participant) = room.get_participant(&client_id) {
            if !participant.permissions.can_edit {
                return Err(StreamError::permission_denied("Cannot edit room state"));
            }
        } else {
            return Err(StreamError::permission_denied("Not a room participant"));
        }

        // Update state
        room.update_state(data.clone());
        let version = room.state.version;

        // Broadcast state update
        let update_msg = StreamMessage::Room(RoomMessage::StateUpdate {
            room_id: room_id.clone(),
            state: data,
            version,
        });
        let _ = self.channels.publish(&room.channel_id, update_msg);

        Ok(version)
    }

    /// Get room participants.
    pub fn get_participants(&self, room_id: &RoomId) -> Result<Vec<Participant>> {
        let room = self
            .rooms
            .get(room_id)
            .ok_or_else(|| StreamError::RoomNotFound(room_id.clone()))?;

        Ok(room
            .participants
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Update participant activity.
    pub fn update_participant_activity(&self, room_id: &RoomId, client_id: ClientId) -> Result<()> {
        let room = self
            .rooms
            .get(room_id)
            .ok_or_else(|| StreamError::RoomNotFound(room_id.clone()))?;

        room.update_activity(&client_id);
        Ok(())
    }

    /// Delete a room.
    pub fn delete_room(&self, room_id: &RoomId) -> Result<()> {
        if let Some((_, room)) = self.rooms.remove(room_id) {
            // Delete associated channel
            self.channels.delete_channel(&room.channel_id)?;
            info!("Deleted room: {}", room_id);
        }
        Ok(())
    }

    /// Get the number of active rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Get all room IDs.
    pub fn list_rooms(&self) -> Vec<RoomId> {
        self.rooms.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Cleanup inactive rooms.
    pub fn cleanup_inactive_rooms(&self, inactive_threshold: std::time::Duration) -> usize {
        let threshold_ms = inactive_threshold.as_millis() as u64;
        let current = current_timestamp();
        let mut removed = 0;

        self.rooms.retain(|room_id, room| {
            if room.state.config.auto_cleanup
                && room.participant_count() == 0
                && (current - room.state.updated_at) > threshold_ms {
                info!("Removing inactive room: {}", room_id);
                let _ = self.channels.delete_channel(&room.channel_id);
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_creation() {
        let channels = Arc::new(ChannelManager::new());
        let manager = RoomManager::new(channels);

        let room_id = "test-room".to_string();
        let name = "Test Room".to_string();
        let config = RoomConfig::default();

        let state = manager.create_room(room_id.clone(), name, config).unwrap();
        assert_eq!(state.room_id, room_id);
        assert_eq!(manager.room_count(), 1);
    }

    #[test]
    fn test_join_leave_room() {
        let channels = Arc::new(ChannelManager::new());
        let manager = RoomManager::new(channels);

        let room_id = "test-room".to_string();
        manager
            .create_room(room_id.clone(), "Test".to_string(), RoomConfig::default())
            .unwrap();

        let client_id = uuid::Uuid::new_v4();
        manager.join_room(&room_id, client_id, None).unwrap();

        let participants = manager.get_participants(&room_id).unwrap();
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0].client_id, client_id);

        manager.leave_room(&room_id, client_id).unwrap();
    }

    #[test]
    fn test_room_state_update() {
        let channels = Arc::new(ChannelManager::new());
        let manager = RoomManager::new(channels);

        let room_id = "test-room".to_string();
        manager
            .create_room(room_id.clone(), "Test".to_string(), RoomConfig::default())
            .unwrap();

        let client_id = uuid::Uuid::new_v4();
        manager.join_room(&room_id, client_id, None).unwrap();

        let new_data = serde_json::json!({"layers": []});
        let version = manager
            .update_room_state(&room_id, client_id, new_data)
            .unwrap();

        assert_eq!(version, 1);
    }
}
