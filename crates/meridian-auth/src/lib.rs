//! Meridian GIS Platform Authentication and Authorization System
//!
//! This crate provides comprehensive authentication and authorization capabilities
//! for the Meridian GIS Platform, including:
//!
//! - **JWT Token Management**: Secure token generation, validation, and refresh
//! - **Password Management**: Argon2id hashing with strength validation
//! - **User Management**: Complete user lifecycle with email verification
//! - **Session Management**: Secure session handling with configurable storage
//! - **RBAC**: Role-based access control with inheritance
//! - **Policy Engine**: Attribute-based access control with conditions
//! - **Audit Logging**: Comprehensive security event tracking
//! - **OAuth2 Support**: Third-party authentication (Google, GitHub, Microsoft)
//!
//! # Examples
//!
//! ## Basic Authentication
//!
//! ```rust,no_run
//! use meridian_auth::{
//!     user::User,
//!     jwt::JwtManager,
//!     password::PasswordPolicy,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new user
//! let mut user = User::new(
//!     "user@example.com".to_string(),
//!     "SecurePassword123!".to_string(),
//!     vec!["viewer".to_string()],
//! )?;
//!
//! // Verify email
//! let token = user.email_verification_token.clone().unwrap();
//! user.verify_email(&token)?;
//!
//! // Authenticate
//! user.authenticate("SecurePassword123!")?;
//!
//! // Generate JWT token
//! let secret = b"your-secret-key-at-least-32-bytes-long!!!";
//! let jwt_manager = JwtManager::new_with_secret(
//!     secret,
//!     "meridian-gis".to_string(),
//!     "meridian-users".to_string(),
//! )?;
//!
//! let token = jwt_manager.generate_access_token(
//!     user.id.clone(),
//!     Some(user.email.clone()),
//!     user.roles.clone(),
//! )?;
//!
//! // Validate token
//! let claims = jwt_manager.validate_access_token(&token)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Role-Based Access Control
//!
//! ```rust
//! use meridian_auth::rbac::{RbacManager, Permission};
//!
//! let rbac = RbacManager::new();
//!
//! // Check permissions
//! let user_roles = vec!["editor".to_string()];
//! let permission = Permission::new("layer", "update");
//!
//! if rbac.has_permission(&user_roles, &permission) {
//!     println!("User can update layers");
//! }
//!
//! // Authorize an action
//! rbac.authorize(&user_roles, "layer", "update").unwrap();
//! ```
//!
//! ## Session Management
//!
//! ```rust,no_run
//! use meridian_auth::session::{SessionManager, InMemorySessionStorage};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = InMemorySessionStorage::new();
//! let mut manager = SessionManager::new(storage);
//!
//! // Create session
//! let session = manager.create_session(
//!     "user123".to_string(),
//!     Some("127.0.0.1".to_string()),
//!     Some("Mozilla/5.0".to_string()),
//! ).await?;
//!
//! // Validate session
//! let validated = manager.validate_session(&session.id).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Audit Logging
//!
//! ```rust,no_run
//! use meridian_auth::audit::{AuditManager, InMemoryAuditLogger};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let logger = InMemoryAuditLogger::new();
//! let mut audit = AuditManager::new(logger);
//!
//! // Log login
//! audit.log_login_success(
//!     "user123",
//!     Some("127.0.0.1".to_string()),
//!     None,
//! ).await?;
//!
//! // Log access denied
//! audit.log_access_denied(
//!     "user123",
//!     "layer",
//!     "layer456",
//!     "delete",
//! ).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod audit;
pub mod error;
pub mod jwt;
pub mod oauth;
pub mod password;
pub mod rbac;
pub mod session;
pub mod user;

// Re-export commonly used types
pub use error::{AuthError, AuthResult};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::audit::{AuditEvent, AuditEventType, AuditLogger, AuditManager, AuditSeverity};
    pub use crate::error::{AuthError, AuthResult};
    pub use crate::jwt::{Claims, JwtManager, TokenPair};
    pub use crate::oauth::{OAuth2Provider, OAuthManager, OAuthProvider, OAuthUserInfo};
    pub use crate::password::{PasswordHasher, PasswordPolicy, PasswordStrength};
    pub use crate::rbac::{Permission, RbacManager, Role};
    pub use crate::rbac::policy::{Policy, PolicyCondition, PolicyContext, PolicyDecision};
    pub use crate::session::{Session, SessionManager, SessionStorage};
    pub use crate::user::{User, UserStatus};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_imports() {
        // This test ensures all prelude exports compile
        use prelude::*;

        let _error: AuthError = AuthError::AccessDenied;
        let _severity: AuditSeverity = AuditSeverity::Info;
        let _provider: OAuthProvider = OAuthProvider::Google;
        let _strength: PasswordStrength = PasswordStrength::Strong;
        let _status: UserStatus = UserStatus::Active;
    }
}
