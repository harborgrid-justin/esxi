//! Authentication middleware
//!
//! Handles JWT token validation, API key authentication,
//! and user context extraction.

use crate::{error::ServerError, state::AppState};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

/// Authentication middleware
pub struct AuthMiddleware;

impl AuthMiddleware {
    /// Create authentication middleware layer
    pub async fn layer(
        State(state): State<AppState>,
        mut req: Request,
        next: Next,
    ) -> Result<Response, ServerError> {
        // Skip authentication if disabled
        if !state.config().auth.enabled {
            return Ok(next.run(req).await);
        }

        // Extract authorization header
        let auth_header = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        if let Some(auth_value) = auth_header {
            // Parse the authorization header
            if let Some(token) = auth_value.strip_prefix("Bearer ") {
                // Validate JWT token
                let user_context = validate_jwt_token(token, &state).await?;
                req.extensions_mut().insert(user_context);
            } else if let Some(api_key) = auth_value.strip_prefix("ApiKey ") {
                // Validate API key
                let user_context = validate_api_key(api_key, &state).await?;
                req.extensions_mut().insert(user_context);
            } else {
                return Err(ServerError::Authentication(
                    "Invalid authorization header format".to_string(),
                ));
            }
        } else {
            return Err(ServerError::Authentication(
                "Missing authorization header".to_string(),
            ));
        }

        Ok(next.run(req).await)
    }

    /// Create optional authentication middleware (doesn't fail if no auth present)
    pub async fn optional_layer(
        State(state): State<AppState>,
        mut req: Request,
        next: Next,
    ) -> Response {
        // Try to extract and validate token if present
        if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    if let Ok(user_context) = validate_jwt_token(token, &state).await {
                        req.extensions_mut().insert(user_context);
                    }
                } else if let Some(api_key) = auth_str.strip_prefix("ApiKey ") {
                    if let Ok(user_context) = validate_api_key(api_key, &state).await {
                        req.extensions_mut().insert(user_context);
                    }
                }
            }
        }

        next.run(req).await
    }
}

/// User context extracted from authentication
#[derive(Clone, Debug)]
pub struct UserContext {
    /// User ID
    pub user_id: uuid::Uuid,

    /// Username
    pub username: String,

    /// User email
    pub email: String,

    /// User roles
    pub roles: Vec<String>,

    /// User permissions
    pub permissions: Vec<String>,

    /// Authentication method
    pub auth_method: AuthMethod,

    /// Token expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Authentication method
#[derive(Clone, Debug, PartialEq)]
pub enum AuthMethod {
    /// JWT token
    Jwt,
    /// API key
    ApiKey,
    /// OAuth2
    OAuth2,
}

impl UserContext {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Check if user is an administrator
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}

/// Validate JWT token
async fn validate_jwt_token(
    _token: &str,
    _state: &AppState,
) -> Result<UserContext, ServerError> {
    // TODO: Implement actual JWT validation using meridian-auth
    // This is a placeholder implementation

    tracing::debug!("Validating JWT token");

    // Mock user context for now
    Ok(UserContext {
        user_id: uuid::Uuid::new_v4(),
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        roles: vec!["user".to_string()],
        permissions: vec!["read:layers".to_string(), "write:features".to_string()],
        auth_method: AuthMethod::Jwt,
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
    })
}

/// Validate API key
async fn validate_api_key(
    _api_key: &str,
    _state: &AppState,
) -> Result<UserContext, ServerError> {
    // TODO: Implement actual API key validation using meridian-auth
    // This is a placeholder implementation

    tracing::debug!("Validating API key");

    // Mock user context for now
    Ok(UserContext {
        user_id: uuid::Uuid::new_v4(),
        username: "api_user".to_string(),
        email: "api@example.com".to_string(),
        roles: vec!["api".to_string()],
        permissions: vec!["read:layers".to_string()],
        auth_method: AuthMethod::ApiKey,
        expires_at: chrono::Utc::now() + chrono::Duration::days(365),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_context() {
        let context = UserContext {
            user_id: uuid::Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec!["read:layers".to_string()],
            auth_method: AuthMethod::Jwt,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        assert!(context.has_role("admin"));
        assert!(!context.has_role("user"));
        assert!(context.is_admin());
        assert!(context.has_permission("read:layers"));
        assert!(!context.is_expired());
    }
}
