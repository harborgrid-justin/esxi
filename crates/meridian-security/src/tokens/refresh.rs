//! Refresh token management
//!
//! Manages long-lived refresh tokens for maintaining user sessions.
//!
//! ## Security Features
//! - Token rotation (new refresh token on each use)
//! - Token families for revocation detection
//! - Secure random token generation
//! - Expiration tracking
//! - Automatic cleanup of expired tokens
//!
//! ## OWASP Best Practices
//! - Refresh token rotation prevents token theft
//! - Token families detect compromise
//! - Secure storage (hash tokens before storing)
//! - Short validity windows
//! - Automatic revocation on suspicious activity
//!
//! ## Refresh Token Flow
//! 1. User authenticates → receives access + refresh tokens
//! 2. Access token expires → client uses refresh token
//! 3. Server validates refresh token → issues new pair
//! 4. Old refresh token is revoked (rotation)
//! 5. If old token reused → entire family revoked (compromise detected)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::{
    config::JWT_REFRESH_TOKEN_LIFETIME,
    error::{SecurityError, SecurityResult},
};

/// Refresh token with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    /// Token ID (JTI)
    pub token_id: String,

    /// User ID
    pub user_id: String,

    /// Token family ID (for detecting token theft)
    pub family_id: String,

    /// Session ID
    pub session_id: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Whether token has been used (should be rotated)
    pub used: bool,

    /// Device/client information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_info: Option<String>,

    /// IP address when token was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
}

impl RefreshToken {
    /// Create a new refresh token
    pub fn new(
        user_id: &str,
        session_id: &str,
        family_id: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let lifetime = Duration::from_std(JWT_REFRESH_TOKEN_LIFETIME)
            .expect("Invalid refresh token lifetime");

        Self {
            token_id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            family_id: family_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            session_id: session_id.to_string(),
            created_at: now,
            expires_at: now + lifetime,
            used: false,
            device_info: None,
            ip_address: None,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Check if token has been used
    pub fn is_used(&self) -> bool {
        self.used
    }

    /// Add device information
    pub fn with_device_info(mut self, info: String) -> Self {
        self.device_info = Some(info);
        self
    }

    /// Add IP address
    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }
}

/// Stored refresh token (hashed for security)
#[derive(Clone)]
struct StoredRefreshToken {
    /// SHA-256 hash of the token
    token_hash: String,

    /// Token metadata
    metadata: RefreshToken,
}

/// Refresh token manager
///
/// Manages the lifecycle of refresh tokens including creation, validation,
/// rotation, and revocation.
///
/// # Example
/// ```rust,no_run
/// use meridian_security::tokens::refresh::RefreshTokenManager;
///
/// let mut manager = RefreshTokenManager::new();
///
/// // Create refresh token
/// let (token, metadata) = manager.create_token("user123", "session456", None).unwrap();
///
/// // Later, validate and rotate
/// let (new_token, new_metadata) = manager.validate_and_rotate(&token).unwrap();
///
/// // Old token is now revoked, new token should be used
/// ```
pub struct RefreshTokenManager {
    /// Stored tokens (indexed by token hash)
    tokens: Arc<RwLock<HashMap<String, StoredRefreshToken>>>,

    /// Token families (for revocation detection)
    families: Arc<RwLock<HashMap<String, Vec<String>>>>, // family_id -> token_hashes
}

impl RefreshTokenManager {
    /// Create a new refresh token manager
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            families: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a cryptographically secure random token
    fn generate_token_string() -> String {
        use rand::RngCore;
        let mut bytes = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        base64::encode(bytes)
    }

    /// Hash a token for storage
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Create a new refresh token
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `session_id` - Session identifier
    /// * `family_id` - Optional family ID (for rotation)
    ///
    /// # Returns
    /// (token_string, token_metadata)
    pub fn create_token(
        &mut self,
        user_id: &str,
        session_id: &str,
        family_id: Option<String>,
    ) -> SecurityResult<(String, RefreshToken)> {
        let token_string = Self::generate_token_string();
        let token_hash = Self::hash_token(&token_string);

        let metadata = RefreshToken::new(user_id, session_id, family_id);

        let stored = StoredRefreshToken {
            token_hash: token_hash.clone(),
            metadata: metadata.clone(),
        };

        let mut tokens = self.tokens.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens write lock".to_string())
        })?;

        let mut families = self.families.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire families write lock".to_string())
        })?;

        // Store token
        tokens.insert(token_hash.clone(), stored);

        // Add to family
        families
            .entry(metadata.family_id.clone())
            .or_insert_with(Vec::new)
            .push(token_hash);

        Ok((token_string, metadata))
    }

    /// Validate a refresh token
    ///
    /// # Arguments
    /// * `token` - Token string to validate
    ///
    /// # Returns
    /// Token metadata if valid
    pub fn validate_token(&self, token: &str) -> SecurityResult<RefreshToken> {
        let token_hash = Self::hash_token(token);

        let tokens = self.tokens.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens read lock".to_string())
        })?;

        let stored = tokens
            .get(&token_hash)
            .ok_or_else(|| SecurityError::InvalidToken("Refresh token not found".to_string()))?;

        let metadata = &stored.metadata;

        // Check expiration
        if metadata.is_expired() {
            return Err(SecurityError::TokenExpired);
        }

        // Check if already used (potential token theft)
        if metadata.is_used() {
            // Token reuse detected! Revoke entire family
            drop(tokens); // Release read lock before write
            self.revoke_family(&metadata.family_id)?;
            return Err(SecurityError::RefreshTokenError(
                "Token reuse detected - session revoked".to_string(),
            ));
        }

        Ok(metadata.clone())
    }

    /// Validate and rotate a refresh token
    ///
    /// This is the primary method for using refresh tokens.
    /// It validates the token and issues a new one (rotation).
    ///
    /// # Security
    /// - Old token is marked as used
    /// - New token is in same family (for revocation detection)
    /// - If old token reused → entire family revoked
    pub fn validate_and_rotate(&mut self, token: &str) -> SecurityResult<(String, RefreshToken)> {
        let token_hash = Self::hash_token(token);

        // Validate token
        let metadata = self.validate_token(token)?;

        // Mark old token as used
        {
            let mut tokens = self.tokens.write().map_err(|_| {
                SecurityError::InternalError("Failed to acquire tokens write lock".to_string())
            })?;

            if let Some(stored) = tokens.get_mut(&token_hash) {
                stored.metadata.used = true;
            }
        }

        // Create new token in same family
        let (new_token, new_metadata) = self.create_token(
            &metadata.user_id,
            &metadata.session_id,
            Some(metadata.family_id.clone()),
        )?;

        Ok((new_token, new_metadata))
    }

    /// Revoke a specific token
    pub fn revoke_token(&mut self, token: &str) -> SecurityResult<()> {
        let token_hash = Self::hash_token(token);

        let mut tokens = self.tokens.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens write lock".to_string())
        })?;

        tokens.remove(&token_hash);

        Ok(())
    }

    /// Revoke an entire token family (when compromise detected)
    pub fn revoke_family(&self, family_id: &str) -> SecurityResult<()> {
        let mut families = self.families.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire families write lock".to_string())
        })?;

        if let Some(token_hashes) = families.remove(family_id) {
            let mut tokens = self.tokens.write().map_err(|_| {
                SecurityError::InternalError("Failed to acquire tokens write lock".to_string())
            })?;

            for hash in token_hashes {
                tokens.remove(&hash);
            }
        }

        Ok(())
    }

    /// Revoke all tokens for a user
    pub fn revoke_user_tokens(&mut self, user_id: &str) -> SecurityResult<()> {
        let mut tokens = self.tokens.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens write lock".to_string())
        })?;

        let to_remove: Vec<String> = tokens
            .iter()
            .filter(|(_, stored)| stored.metadata.user_id == user_id)
            .map(|(hash, _)| hash.clone())
            .collect();

        for hash in to_remove {
            tokens.remove(&hash);
        }

        Ok(())
    }

    /// Revoke all tokens for a session
    pub fn revoke_session_tokens(&mut self, session_id: &str) -> SecurityResult<()> {
        let mut tokens = self.tokens.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens write lock".to_string())
        })?;

        let to_remove: Vec<String> = tokens
            .iter()
            .filter(|(_, stored)| stored.metadata.session_id == session_id)
            .map(|(hash, _)| hash.clone())
            .collect();

        for hash in to_remove {
            tokens.remove(&hash);
        }

        Ok(())
    }

    /// Clean up expired tokens
    ///
    /// Should be called periodically to remove expired tokens from storage
    pub fn cleanup_expired(&mut self) -> SecurityResult<usize> {
        let mut tokens = self.tokens.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens write lock".to_string())
        })?;

        let now = Utc::now();
        let to_remove: Vec<String> = tokens
            .iter()
            .filter(|(_, stored)| stored.metadata.expires_at <= now)
            .map(|(hash, _)| hash.clone())
            .collect();

        let count = to_remove.len();

        for hash in to_remove {
            tokens.remove(&hash);
        }

        Ok(count)
    }

    /// Get count of active tokens
    pub fn token_count(&self) -> SecurityResult<usize> {
        let tokens = self.tokens.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens read lock".to_string())
        })?;

        Ok(tokens.len())
    }

    /// Get tokens for a user
    pub fn get_user_tokens(&self, user_id: &str) -> SecurityResult<Vec<RefreshToken>> {
        let tokens = self.tokens.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire tokens read lock".to_string())
        })?;

        let user_tokens = tokens
            .values()
            .filter(|stored| stored.metadata.user_id == user_id)
            .map(|stored| stored.metadata.clone())
            .collect();

        Ok(user_tokens)
    }
}

impl Default for RefreshTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let mut manager = RefreshTokenManager::new();

        let (token, metadata) = manager.create_token("user123", "session456", None).unwrap();

        assert!(!token.is_empty());
        assert_eq!(metadata.user_id, "user123");
        assert_eq!(metadata.session_id, "session456");
        assert!(!metadata.is_expired());
        assert!(!metadata.is_used());

        let validated = manager.validate_token(&token).unwrap();
        assert_eq!(validated.user_id, "user123");
    }

    #[test]
    fn test_token_rotation() {
        let mut manager = RefreshTokenManager::new();

        let (token1, meta1) = manager.create_token("user123", "session456", None).unwrap();

        let (token2, meta2) = manager.validate_and_rotate(&token1).unwrap();

        assert_ne!(token1, token2);
        assert_eq!(meta1.family_id, meta2.family_id); // Same family
        assert_eq!(meta2.user_id, "user123");

        // Old token should be marked as used
        let result = manager.validate_and_rotate(&token1);
        assert!(result.is_err()); // Token reuse should fail
    }

    #[test]
    fn test_token_reuse_detection() {
        let mut manager = RefreshTokenManager::new();

        let (token, _) = manager.create_token("user123", "session456", None).unwrap();

        // Use token once (rotate)
        let _ = manager.validate_and_rotate(&token).unwrap();

        // Try to use again - should detect reuse and revoke family
        let result = manager.validate_and_rotate(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_token() {
        let mut manager = RefreshTokenManager::new();

        let (token, _) = manager.create_token("user123", "session456", None).unwrap();

        manager.revoke_token(&token).unwrap();

        let result = manager.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_user_tokens() {
        let mut manager = RefreshTokenManager::new();

        let (token1, _) = manager.create_token("user123", "session1", None).unwrap();
        let (token2, _) = manager.create_token("user123", "session2", None).unwrap();
        let (token3, _) = manager.create_token("user456", "session3", None).unwrap();

        manager.revoke_user_tokens("user123").unwrap();

        assert!(manager.validate_token(&token1).is_err());
        assert!(manager.validate_token(&token2).is_err());
        assert!(manager.validate_token(&token3).is_ok()); // Different user
    }

    #[test]
    fn test_revoke_session() {
        let mut manager = RefreshTokenManager::new();

        let (token1, _) = manager.create_token("user123", "session1", None).unwrap();
        let (token2, _) = manager.create_token("user123", "session2", None).unwrap();

        manager.revoke_session_tokens("session1").unwrap();

        assert!(manager.validate_token(&token1).is_err());
        assert!(manager.validate_token(&token2).is_ok());
    }

    #[test]
    fn test_token_family() {
        let mut manager = RefreshTokenManager::new();

        let (token1, meta1) = manager.create_token("user123", "session456", None).unwrap();
        let family_id = meta1.family_id.clone();

        let (token2, meta2) = manager.validate_and_rotate(&token1).unwrap();
        assert_eq!(meta2.family_id, family_id);

        let (token3, meta3) = manager.validate_and_rotate(&token2).unwrap();
        assert_eq!(meta3.family_id, family_id);
    }

    #[test]
    fn test_get_user_tokens() {
        let mut manager = RefreshTokenManager::new();

        manager.create_token("user123", "session1", None).unwrap();
        manager.create_token("user123", "session2", None).unwrap();
        manager.create_token("user456", "session3", None).unwrap();

        let user_tokens = manager.get_user_tokens("user123").unwrap();
        assert_eq!(user_tokens.len(), 2);
    }

    #[test]
    fn test_token_count() {
        let mut manager = RefreshTokenManager::new();

        assert_eq!(manager.token_count().unwrap(), 0);

        manager.create_token("user123", "session1", None).unwrap();
        manager.create_token("user123", "session2", None).unwrap();

        assert_eq!(manager.token_count().unwrap(), 2);
    }

    #[test]
    fn test_cleanup_expired() {
        let mut manager = RefreshTokenManager::new();

        // In real scenario, tokens would expire over time
        // This is a basic structure test
        let cleaned = manager.cleanup_expired().unwrap();
        assert_eq!(cleaned, 0); // No expired tokens yet
    }
}
