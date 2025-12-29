//! Collaboration features for real-time multi-user editing

pub mod cursor;
pub mod selection;
pub mod annotation;
pub mod presence;

pub use cursor::{Cursor, CursorManager};
pub use selection::{Selection, SelectionManager};
pub use annotation::{Annotation, AnnotationManager};
pub use presence::{Presence, PresenceManager, UserStatus};

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Collaboration session
pub struct CollaborationSession {
    /// Session ID
    id: String,

    /// Presence manager
    presence: Arc<PresenceManager>,

    /// Cursor manager
    cursors: Arc<CursorManager>,

    /// Selection manager
    selections: Arc<SelectionManager>,

    /// Annotation manager
    annotations: Arc<AnnotationManager>,
}

impl CollaborationSession {
    /// Create new collaboration session
    pub fn new(session_id: String) -> Self {
        Self {
            id: session_id.clone(),
            presence: Arc::new(PresenceManager::new()),
            cursors: Arc::new(CursorManager::new()),
            selections: Arc::new(SelectionManager::new()),
            annotations: Arc::new(AnnotationManager::new()),
        }
    }

    /// Get session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get presence manager
    pub fn presence(&self) -> &PresenceManager {
        &self.presence
    }

    /// Get cursor manager
    pub fn cursors(&self) -> &CursorManager {
        &self.cursors
    }

    /// Get selection manager
    pub fn selections(&self) -> &SelectionManager {
        &self.selections
    }

    /// Get annotation manager
    pub fn annotations(&self) -> &AnnotationManager {
        &self.annotations
    }
}

/// User information for collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    /// User ID
    pub user_id: String,

    /// Display name
    pub name: String,

    /// Avatar URL
    pub avatar_url: Option<String>,

    /// User color (for cursor/selection highlighting)
    pub color: String,

    /// Custom metadata
    pub metadata: serde_json::Value,
}

impl CollaboratorInfo {
    /// Create new collaborator info
    pub fn new(user_id: String, name: String) -> Self {
        Self {
            user_id: user_id.clone(),
            name,
            avatar_url: None,
            color: generate_user_color(&user_id),
            metadata: serde_json::json!({}),
        }
    }

    /// With avatar URL
    pub fn with_avatar(mut self, url: String) -> Self {
        self.avatar_url = Some(url);
        self
    }

    /// With custom color
    pub fn with_color(mut self, color: String) -> Self {
        self.color = color;
        self
    }

    /// With metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Generate consistent color for a user
fn generate_user_color(user_id: &str) -> String {
    // Simple hash-based color generation
    let hash = user_id.bytes().fold(0u32, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as u32)
    });

    let hue = (hash % 360) as f32;
    let saturation = 70.0;
    let lightness = 60.0;

    format!("hsl({}, {}%, {}%)", hue, saturation, lightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_session() {
        let session = CollaborationSession::new("session1".to_string());
        assert_eq!(session.id(), "session1");
    }

    #[test]
    fn test_collaborator_info() {
        let info = CollaboratorInfo::new("user1".to_string(), "Alice".to_string());
        assert_eq!(info.user_id, "user1");
        assert_eq!(info.name, "Alice");
        assert!(!info.color.is_empty());
    }

    #[test]
    fn test_user_color_generation() {
        let color1 = generate_user_color("user1");
        let color2 = generate_user_color("user2");

        assert!(color1.starts_with("hsl("));
        assert!(color2.starts_with("hsl("));
        assert_ne!(color1, color2); // Different users should get different colors
    }
}
