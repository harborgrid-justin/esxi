//! Session management for authenticated users

use crate::error::{AuthError, AuthResult};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Session last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Device information
    pub device_info: Option<String>,
    /// Session metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Session {
    /// Create a new session
    pub fn new(
        user_id: String,
        duration: Duration,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            created_at: now,
            last_activity: now,
            expires_at: now + duration,
            ip_address,
            user_agent,
            device_info: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Check if session is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Extend session expiration
    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
        self.touch();
    }

    /// Get session age
    pub fn age(&self) -> Duration {
        Utc::now() - self.created_at
    }

    /// Get time since last activity
    pub fn idle_time(&self) -> Duration {
        Utc::now() - self.last_activity
    }

    /// Get time until expiration
    pub fn time_until_expiration(&self) -> Duration {
        self.expires_at - Utc::now()
    }
}

/// Session storage trait for implementing different storage backends
#[async_trait]
pub trait SessionStorage: Send + Sync {
    /// Create a new session
    async fn create(&mut self, session: Session) -> AuthResult<()>;

    /// Get a session by ID
    async fn get(&self, session_id: &str) -> AuthResult<Option<Session>>;

    /// Update an existing session
    async fn update(&mut self, session: Session) -> AuthResult<()>;

    /// Delete a session by ID
    async fn delete(&mut self, session_id: &str) -> AuthResult<()>;

    /// Get all sessions for a user
    async fn get_user_sessions(&self, user_id: &str) -> AuthResult<Vec<Session>>;

    /// Delete all sessions for a user
    async fn delete_user_sessions(&mut self, user_id: &str) -> AuthResult<()>;

    /// Clean up expired sessions
    async fn cleanup_expired(&mut self) -> AuthResult<usize>;
}

/// In-memory session storage (for testing and development)
#[derive(Debug, Clone, Default)]
pub struct InMemorySessionStorage {
    sessions: HashMap<String, Session>,
}

impl InMemorySessionStorage {
    /// Create a new in-memory session storage
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

#[async_trait]
impl SessionStorage for InMemorySessionStorage {
    async fn create(&mut self, session: Session) -> AuthResult<()> {
        self.sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn get(&self, session_id: &str) -> AuthResult<Option<Session>> {
        Ok(self.sessions.get(session_id).cloned())
    }

    async fn update(&mut self, session: Session) -> AuthResult<()> {
        if !self.sessions.contains_key(&session.id) {
            return Err(AuthError::SessionNotFound);
        }
        self.sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn delete(&mut self, session_id: &str) -> AuthResult<()> {
        self.sessions.remove(session_id);
        Ok(())
    }

    async fn get_user_sessions(&self, user_id: &str) -> AuthResult<Vec<Session>> {
        let sessions = self
            .sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect();
        Ok(sessions)
    }

    async fn delete_user_sessions(&mut self, user_id: &str) -> AuthResult<()> {
        let session_ids: Vec<String> = self
            .sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .map(|s| s.id.clone())
            .collect();

        for id in session_ids {
            self.sessions.remove(&id);
        }

        Ok(())
    }

    async fn cleanup_expired(&mut self) -> AuthResult<usize> {
        let expired_ids: Vec<String> = self
            .sessions
            .values()
            .filter(|s| s.is_expired())
            .map(|s| s.id.clone())
            .collect();

        let count = expired_ids.len();
        for id in expired_ids {
            self.sessions.remove(&id);
        }

        Ok(count)
    }
}

/// Session manager
pub struct SessionManager<S: SessionStorage> {
    storage: S,
    default_duration: Duration,
    max_sessions_per_user: Option<usize>,
    idle_timeout: Option<Duration>,
}

impl<S: SessionStorage> SessionManager<S> {
    /// Create a new session manager
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            default_duration: Duration::hours(24),
            max_sessions_per_user: Some(5),
            idle_timeout: Some(Duration::hours(1)),
        }
    }

    /// Set default session duration
    pub fn set_default_duration(&mut self, duration: Duration) {
        self.default_duration = duration;
    }

    /// Set maximum sessions per user
    pub fn set_max_sessions_per_user(&mut self, max: Option<usize>) {
        self.max_sessions_per_user = max;
    }

    /// Set idle timeout
    pub fn set_idle_timeout(&mut self, timeout: Option<Duration>) {
        self.idle_timeout = timeout;
    }

    /// Create a new session
    pub async fn create_session(
        &mut self,
        user_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AuthResult<Session> {
        // Check if user has too many sessions
        if let Some(max) = self.max_sessions_per_user {
            let user_sessions = self.storage.get_user_sessions(&user_id).await?;
            let active_sessions = user_sessions.iter().filter(|s| s.is_valid()).count();

            if active_sessions >= max {
                return Err(AuthError::TooManySessions);
            }
        }

        let session = Session::new(user_id, self.default_duration, ip_address, user_agent);
        self.storage.create(session.clone()).await?;

        Ok(session)
    }

    /// Get and validate a session
    pub async fn get_session(&self, session_id: &str) -> AuthResult<Session> {
        let session = self
            .storage
            .get(session_id)
            .await?
            .ok_or(AuthError::SessionNotFound)?;

        // Check if expired
        if session.is_expired() {
            return Err(AuthError::SessionExpired);
        }

        // Check idle timeout
        if let Some(idle_timeout) = self.idle_timeout {
            if session.idle_time() > idle_timeout {
                return Err(AuthError::SessionExpired);
            }
        }

        Ok(session)
    }

    /// Validate and touch session
    pub async fn validate_session(&mut self, session_id: &str) -> AuthResult<Session> {
        let mut session = self.get_session(session_id).await?;
        session.touch();
        self.storage.update(session.clone()).await?;
        Ok(session)
    }

    /// Extend session
    pub async fn extend_session(
        &mut self,
        session_id: &str,
        duration: Option<Duration>,
    ) -> AuthResult<Session> {
        let mut session = self.get_session(session_id).await?;
        session.extend(duration.unwrap_or(self.default_duration));
        self.storage.update(session.clone()).await?;
        Ok(session)
    }

    /// Delete a session (logout)
    pub async fn delete_session(&mut self, session_id: &str) -> AuthResult<()> {
        self.storage.delete(session_id).await
    }

    /// Delete all user sessions (logout all devices)
    pub async fn delete_all_user_sessions(&mut self, user_id: &str) -> AuthResult<()> {
        self.storage.delete_user_sessions(user_id).await
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: &str) -> AuthResult<Vec<Session>> {
        let sessions = self.storage.get_user_sessions(user_id).await?;
        Ok(sessions.into_iter().filter(|s| s.is_valid()).collect())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&mut self) -> AuthResult<usize> {
        self.storage.cleanup_expired().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let session = Session::new(
            "user123".to_string(),
            Duration::hours(1),
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        assert_eq!(session.user_id, "user123");
        assert!(!session.is_expired());
        assert!(session.is_valid());
    }

    #[tokio::test]
    async fn test_session_expiration() {
        let mut session = Session::new(
            "user123".to_string(),
            Duration::seconds(-1), // Already expired
            None,
            None,
        );

        assert!(session.is_expired());
        assert!(!session.is_valid());
    }

    #[tokio::test]
    async fn test_session_touch() {
        let mut session = Session::new(
            "user123".to_string(),
            Duration::hours(1),
            None,
            None,
        );

        let initial_activity = session.last_activity;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        session.touch();

        assert!(session.last_activity > initial_activity);
    }

    #[tokio::test]
    async fn test_in_memory_storage() {
        let mut storage = InMemorySessionStorage::new();
        let session = Session::new("user123".to_string(), Duration::hours(1), None, None);
        let session_id = session.id.clone();

        // Create session
        storage.create(session.clone()).await.unwrap();

        // Get session
        let retrieved = storage.get(&session_id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, session_id);

        // Update session
        let mut updated = retrieved.clone();
        updated.touch();
        storage.update(updated).await.unwrap();

        // Delete session
        storage.delete(&session_id).await.unwrap();
        assert!(storage.get(&session_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_session_manager() {
        let storage = InMemorySessionStorage::new();
        let mut manager = SessionManager::new(storage);

        // Create session
        let session = manager
            .create_session(
                "user123".to_string(),
                Some("127.0.0.1".to_string()),
                Some("Mozilla/5.0".to_string()),
            )
            .await
            .unwrap();

        // Validate session
        let validated = manager.validate_session(&session.id).await.unwrap();
        assert_eq!(validated.user_id, "user123");

        // Delete session
        manager.delete_session(&session.id).await.unwrap();

        // Session should not exist
        assert!(manager.get_session(&session.id).await.is_err());
    }

    #[tokio::test]
    async fn test_max_sessions_per_user() {
        let storage = InMemorySessionStorage::new();
        let mut manager = SessionManager::new(storage);
        manager.set_max_sessions_per_user(Some(2));

        // Create 2 sessions (should succeed)
        manager
            .create_session("user123".to_string(), None, None)
            .await
            .unwrap();
        manager
            .create_session("user123".to_string(), None, None)
            .await
            .unwrap();

        // Third session should fail
        let result = manager.create_session("user123".to_string(), None, None).await;
        assert!(matches!(result, Err(AuthError::TooManySessions)));
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let mut storage = InMemorySessionStorage::new();

        // Create expired session
        let expired = Session::new("user1".to_string(), Duration::seconds(-1), None, None);
        storage.create(expired).await.unwrap();

        // Create valid session
        let valid = Session::new("user2".to_string(), Duration::hours(1), None, None);
        storage.create(valid.clone()).await.unwrap();

        // Cleanup
        let count = storage.cleanup_expired().await.unwrap();
        assert_eq!(count, 1);

        // Only valid session should remain
        assert!(storage.get(&valid.id).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_user_sessions() {
        let mut storage = InMemorySessionStorage::new();

        // Create multiple sessions for same user
        let session1 = Session::new("user123".to_string(), Duration::hours(1), None, None);
        let session2 = Session::new("user123".to_string(), Duration::hours(1), None, None);

        storage.create(session1).await.unwrap();
        storage.create(session2).await.unwrap();

        // Get all user sessions
        let sessions = storage.get_user_sessions("user123").await.unwrap();
        assert_eq!(sessions.len(), 2);

        // Delete all user sessions
        storage.delete_user_sessions("user123").await.unwrap();
        let sessions = storage.get_user_sessions("user123").await.unwrap();
        assert_eq!(sessions.len(), 0);
    }
}
