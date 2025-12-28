//! Message handlers for processing incoming streaming messages.

use crate::channel::ChannelManager;
use crate::error::Result;
use crate::messages::{
    ClientId, FeatureUpdateMessage, LayerUpdateMessage, PresenceUpdateMessage, RoomMessage,
    StreamMessage, SubscribeMessage, SyncMessage, UnsubscribeMessage, ViewportUpdateMessage,
};
use crate::room::RoomManager;
use crate::sync::SyncManager;
use crate::viewport::ViewportManager;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Handler for processing streaming messages.
pub struct MessageHandler {
    /// Channel manager
    channels: Arc<ChannelManager>,
    /// Room manager
    rooms: Arc<RoomManager>,
    /// Sync manager
    sync: Arc<SyncManager>,
    /// Viewport manager
    viewports: Arc<ViewportManager>,
}

impl MessageHandler {
    /// Create a new message handler.
    pub fn new(
        channels: Arc<ChannelManager>,
        rooms: Arc<RoomManager>,
        sync: Arc<SyncManager>,
        viewports: Arc<ViewportManager>,
    ) -> Self {
        Self {
            channels,
            rooms,
            sync,
            viewports,
        }
    }

    /// Handle an incoming message.
    pub async fn handle(&self, client_id: ClientId, message: StreamMessage) -> Result<Option<StreamMessage>> {
        debug!("Handling message from client {}: {:?}", client_id, message);

        match message {
            StreamMessage::FeatureUpdate(msg) => {
                self.handle_feature_update(client_id, msg).await
            }
            StreamMessage::LayerUpdate(msg) => {
                self.handle_layer_update(client_id, msg).await
            }
            StreamMessage::ViewportUpdate(msg) => {
                self.handle_viewport_update(client_id, msg).await
            }
            StreamMessage::PresenceUpdate(msg) => {
                self.handle_presence_update(client_id, msg).await
            }
            StreamMessage::Room(msg) => {
                self.handle_room_message(client_id, msg).await
            }
            StreamMessage::Subscribe(msg) => {
                self.handle_subscribe(client_id, msg).await
            }
            StreamMessage::Unsubscribe(msg) => {
                self.handle_unsubscribe(client_id, msg).await
            }
            StreamMessage::Sync(msg) => {
                self.handle_sync(client_id, msg).await
            }
            StreamMessage::Ping { timestamp } => {
                Ok(Some(StreamMessage::Pong { timestamp }))
            }
            StreamMessage::Pong { .. } => {
                // Pong received, no response needed
                Ok(None)
            }
            StreamMessage::Custom { channel, data } => {
                self.handle_custom(client_id, channel, data).await
            }
            StreamMessage::Error(_) => {
                // Error message received, log it
                warn!("Received error message from client {}", client_id);
                Ok(None)
            }
        }
    }

    /// Handle feature update message.
    async fn handle_feature_update(
        &self,
        _client_id: ClientId,
        msg: FeatureUpdateMessage,
    ) -> Result<Option<StreamMessage>> {
        info!(
            "Feature update: {} on layer {} (version {})",
            msg.feature_id, msg.layer_id, msg.version
        );

        // Broadcast to layer channel
        let channel_id = format!("layer:{}", msg.layer_id);
        let envelope = StreamMessage::FeatureUpdate(msg.clone());

        match self.channels.publish(&channel_id, envelope) {
            Ok(count) => {
                debug!("Broadcast feature update to {} subscribers", count);
            }
            Err(e) => {
                warn!("Failed to broadcast feature update: {}", e);
            }
        }

        // Also broadcast to spatial viewport subscribers if we have geometry
        if let Some(data) = &msg.data {
            if let Some(geometry) = data.get("geometry") {
                if let Some(coords) = extract_point_from_geometry(geometry) {
                    let point = [coords[0], coords[1]];
                    let _ = self.viewports.broadcast_to_point(point, StreamMessage::FeatureUpdate(msg));
                }
            }
        }

        Ok(None)
    }

    /// Handle layer update message.
    async fn handle_layer_update(
        &self,
        _client_id: ClientId,
        msg: LayerUpdateMessage,
    ) -> Result<Option<StreamMessage>> {
        info!("Layer update: {} ({:?})", msg.layer_id, msg.operation);

        // Broadcast to layer channel
        let channel_id = format!("layer:{}", msg.layer_id);
        let envelope = StreamMessage::LayerUpdate(msg.clone());

        match self.channels.publish(&channel_id, envelope.clone()) {
            Ok(count) => {
                debug!("Broadcast layer update to {} subscribers", count);
            }
            Err(e) => {
                warn!("Failed to broadcast layer update: {}", e);
            }
        }

        // Also broadcast to all viewers of this layer
        let _ = self.viewports.broadcast_to_layer(&msg.layer_id, envelope);

        Ok(None)
    }

    /// Handle viewport update message.
    async fn handle_viewport_update(
        &self,
        _client_id: ClientId,
        msg: ViewportUpdateMessage,
    ) -> Result<Option<StreamMessage>> {
        debug!(
            "Viewport update: client {} at zoom {} (bounds: {:?})",
            msg.client_id, msg.zoom, msg.bounds
        );

        // Update viewport
        self.viewports.update_viewport(msg)?;

        Ok(None)
    }

    /// Handle presence update message.
    async fn handle_presence_update(
        &self,
        _client_id: ClientId,
        msg: PresenceUpdateMessage,
    ) -> Result<Option<StreamMessage>> {
        debug!("Presence update: client {} ({:?})", msg.client_id, msg.status);

        // Broadcast to presence channel
        let channel_id = "presence".to_string();
        let envelope = StreamMessage::PresenceUpdate(msg.clone());

        match self.channels.publish(&channel_id, envelope) {
            Ok(count) => {
                debug!("Broadcast presence update to {} subscribers", count);
            }
            Err(e) => {
                warn!("Failed to broadcast presence update: {}", e);
            }
        }

        Ok(None)
    }

    /// Handle room message.
    async fn handle_room_message(
        &self,
        _client_id: ClientId,
        msg: RoomMessage,
    ) -> Result<Option<StreamMessage>> {
        match &msg {
            RoomMessage::Join { room_id, client_id, user_data } => {
                info!("Client {} joining room {}", client_id, room_id);

                match self.rooms.join_room(room_id, *client_id, user_data.clone()) {
                    Ok(state) => {
                        // Return room state to client
                        Ok(Some(StreamMessage::Room(RoomMessage::StateUpdate {
                            room_id: room_id.clone(),
                            state: serde_json::to_value(&state).unwrap_or(serde_json::Value::Null),
                            version: state.version,
                        })))
                    }
                    Err(e) => {
                        error!("Failed to join room: {}", e);
                        Ok(Some(StreamMessage::error("ROOM_JOIN_FAILED", e.to_string())))
                    }
                }
            }
            RoomMessage::Leave { room_id, client_id } => {
                info!("Client {} leaving room {}", client_id, room_id);

                match self.rooms.leave_room(room_id, *client_id) {
                    Ok(_) => Ok(None),
                    Err(e) => {
                        error!("Failed to leave room: {}", e);
                        Ok(Some(StreamMessage::error("ROOM_LEAVE_FAILED", e.to_string())))
                    }
                }
            }
            RoomMessage::StateUpdate { room_id,  .. } => {
                info!("Room state update for {}", room_id);

                // This would typically come from the room manager
                // For now, just broadcast to room channel
                let channel_id = format!("room:{}", room_id);
                let envelope = StreamMessage::Room(msg);

                match self.channels.publish(&channel_id, envelope) {
                    Ok(count) => {
                        debug!("Broadcast room state to {} subscribers", count);
                        Ok(None)
                    }
                    Err(e) => {
                        warn!("Failed to broadcast room state: {}", e);
                        Ok(Some(StreamMessage::error("ROOM_UPDATE_FAILED", e.to_string())))
                    }
                }
            }
            _ => {
                // ParticipantJoined and ParticipantLeft are typically generated by the server
                Ok(None)
            }
        }
    }

    /// Handle subscribe message.
    async fn handle_subscribe(
        &self,
        client_id: ClientId,
        msg: SubscribeMessage,
    ) -> Result<Option<StreamMessage>> {
        info!("Client {} subscribing to channel {}", client_id, msg.channel);

        match self.channels.subscribe(client_id, msg.channel.clone()) {
            Ok(_) => {
                debug!("Client {} subscribed to {}", client_id, msg.channel);
                Ok(None)
            }
            Err(e) => {
                error!("Failed to subscribe: {}", e);
                Ok(Some(StreamMessage::error("SUBSCRIBE_FAILED", e.to_string())))
            }
        }
    }

    /// Handle unsubscribe message.
    async fn handle_unsubscribe(
        &self,
        client_id: ClientId,
        msg: UnsubscribeMessage,
    ) -> Result<Option<StreamMessage>> {
        info!("Client {} unsubscribing from channel {}", client_id, msg.channel);

        match self.channels.unsubscribe(client_id, &msg.channel) {
            Ok(_) => {
                debug!("Client {} unsubscribed from {}", client_id, msg.channel);
                Ok(None)
            }
            Err(e) => {
                error!("Failed to unsubscribe: {}", e);
                Ok(Some(StreamMessage::error("UNSUBSCRIBE_FAILED", e.to_string())))
            }
        }
    }

    /// Handle sync message.
    async fn handle_sync(
        &self,
        client_id: ClientId,
        msg: SyncMessage,
    ) -> Result<Option<StreamMessage>> {
        debug!("Sync message from client {}", client_id);

        let entity_id = match &msg {
            SyncMessage::RequestState { entity_id, .. } => entity_id.clone(),
            SyncMessage::Operation { entity_id, .. } => entity_id.clone(),
            _ => return Ok(None),
        };

        match self.sync.handle_sync_message(&entity_id, client_id, msg) {
            Ok(Some(response)) => Ok(Some(StreamMessage::Sync(response))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Sync error: {}", e);
                Ok(Some(StreamMessage::error("SYNC_FAILED", e.to_string())))
            }
        }
    }

    /// Handle custom message.
    async fn handle_custom(
        &self,
        _client_id: ClientId,
        channel: String,
        data: serde_json::Value,
    ) -> Result<Option<StreamMessage>> {
        debug!("Custom message on channel {}", channel);

        // Broadcast custom message to channel
        let envelope = StreamMessage::Custom {
            channel: channel.clone(),
            data,
        };

        match self.channels.publish(&channel, envelope) {
            Ok(count) => {
                debug!("Broadcast custom message to {} subscribers", count);
                Ok(None)
            }
            Err(e) => {
                warn!("Failed to broadcast custom message: {}", e);
                Ok(Some(StreamMessage::error("CUSTOM_BROADCAST_FAILED", e.to_string())))
            }
        }
    }

    /// Cleanup resources for a disconnected client.
    pub async fn cleanup_client(&self, client_id: ClientId) -> Result<()> {
        info!("Cleaning up resources for client {}", client_id);

        // Unsubscribe from all channels
        self.channels.unsubscribe_all(client_id)?;

        // Remove viewport
        self.viewports.remove_viewport(&client_id)?;

        // Note: Room cleanup is handled by room manager when clients leave

        Ok(())
    }
}

/// Extract a point from GeoJSON geometry.
fn extract_point_from_geometry(geometry: &serde_json::Value) -> Option<Vec<f64>> {
    let geom_type = geometry.get("type")?.as_str()?;
    let coordinates = geometry.get("coordinates")?;

    match geom_type {
        "Point" => {
            let coords = coordinates.as_array()?;
            if coords.len() >= 2 {
                Some(vec![
                    coords[0].as_f64()?,
                    coords[1].as_f64()?,
                ])
            } else {
                None
            }
        }
        "Polygon" | "MultiPolygon" | "LineString" | "MultiLineString" => {
            // For complex geometries, we could compute centroid
            // For now, just return None
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{FeatureOperation, current_timestamp};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_handler_creation() {
        let channels = Arc::new(ChannelManager::new());
        let rooms = Arc::new(RoomManager::new(channels.clone()));
        let sync = Arc::new(SyncManager::new());
        let viewports = Arc::new(ViewportManager::new(channels.clone()));

        let _handler = MessageHandler::new(channels, rooms, sync, viewports);
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let channels = Arc::new(ChannelManager::new());
        let rooms = Arc::new(RoomManager::new(channels.clone()));
        let sync = Arc::new(SyncManager::new());
        let viewports = Arc::new(ViewportManager::new(channels.clone()));
        let handler = MessageHandler::new(channels, rooms, sync, viewports);

        let client_id = Uuid::new_v4();
        let timestamp = current_timestamp();
        let msg = StreamMessage::Ping { timestamp };

        let response = handler.handle(client_id, msg).await.unwrap();
        assert!(matches!(response, Some(StreamMessage::Pong { .. })));
    }

    #[tokio::test]
    async fn test_handle_subscribe() {
        let channels = Arc::new(ChannelManager::new());
        let rooms = Arc::new(RoomManager::new(channels.clone()));
        let sync = Arc::new(SyncManager::new());
        let viewports = Arc::new(ViewportManager::new(channels.clone()));
        let handler = MessageHandler::new(channels.clone(), rooms, sync, viewports);

        let client_id = Uuid::new_v4();
        let msg = StreamMessage::Subscribe(SubscribeMessage {
            channel: "test-channel".to_string(),
            client_id,
            filter: None,
        });

        let response = handler.handle(client_id, msg).await.unwrap();
        assert!(response.is_none());

        // Verify subscription
        let subs = channels.get_client_channels(&client_id);
        assert_eq!(subs.len(), 1);
    }

    #[test]
    fn test_extract_point_from_geometry() {
        let point_geom = serde_json::json!({
            "type": "Point",
            "coordinates": [10.0, 20.0]
        });

        let coords = extract_point_from_geometry(&point_geom).unwrap();
        assert_eq!(coords, vec![10.0, 20.0]);
    }
}
