//! Error types for the Meridian authentication system

use thiserror::Error;

/// Result type alias for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Comprehensive authentication and authorization errors
#[derive(Debug, Error)]
pub enum AuthError {
    /// Password-related errors
    #[error("Invalid password: {0}")]
    InvalidPassword(String),

    #[error("Password too weak: {0}")]
    WeakPassword(String),

    #[error("Password hashing failed: {0}")]
    PasswordHashError(String),

    /// JWT-related errors
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Token generation failed: {0}")]
    TokenGenerationError(String),

    #[error("Token revoked")]
    TokenRevoked,

    #[error("Invalid token signature")]
    InvalidSignature,

    /// User-related errors
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("User already exists: {0}")]
    UserAlreadyExists(String),

    #[error("User account locked")]
    UserLocked,

    #[error("User account disabled")]
    UserDisabled,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Invalid email address: {0}")]
    InvalidEmail(String),

    /// Session-related errors
    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session invalid")]
    SessionInvalid,

    #[error("Too many sessions")]
    TooManySessions,

    /// Authorization errors
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("Access denied")]
    AccessDenied,

    #[error("Invalid role: {0}")]
    InvalidRole(String),

    #[error("Policy evaluation failed: {0}")]
    PolicyEvaluationError(String),

    /// OAuth-related errors
    #[error("OAuth provider error: {0}")]
    OAuthProviderError(String),

    #[error("OAuth token exchange failed: {0}")]
    OAuthTokenExchangeFailed(String),

    #[error("Unsupported OAuth provider: {0}")]
    UnsupportedOAuthProvider(String),

    /// Audit-related errors
    #[error("Audit logging failed: {0}")]
    AuditLogError(String),

    /// Generic errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Implement conversions from various error types
impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            ErrorKind::InvalidSignature => AuthError::InvalidSignature,
            _ => AuthError::InvalidToken(err.to_string()),
        }
    }
}

impl From<argon2::Error> for AuthError {
    fn from(err: argon2::Error) -> Self {
        AuthError::PasswordHashError(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AuthError::PasswordHashError(err.to_string())
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(err: serde_json::Error) -> Self {
        AuthError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AuthError::InvalidPassword("too short".to_string());
        assert_eq!(err.to_string(), "Invalid password: too short");

        let err = AuthError::TokenExpired;
        assert_eq!(err.to_string(), "Token expired");

        let err = AuthError::AccessDenied;
        assert_eq!(err.to_string(), "Access denied");
    }

    #[test]
    fn test_error_conversions() {
        let jwt_err = jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::ExpiredSignature
        );
        let auth_err: AuthError = jwt_err.into();
        assert!(matches!(auth_err, AuthError::TokenExpired));
    }
}
