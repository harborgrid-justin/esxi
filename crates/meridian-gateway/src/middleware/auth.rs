//! Authentication Middleware
//!
//! Enterprise authentication with JWT, API Key, and OAuth support.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    /// No authentication token provided
    #[error("Missing authentication token")]
    MissingToken,

    /// Authentication token is invalid
    #[error("Invalid authentication token")]
    InvalidToken,

    /// Authentication token has expired
    #[error("Expired token")]
    ExpiredToken,

    /// User lacks required permissions
    #[error("Insufficient permissions")]
    InsufficientPermissions,

    /// Unknown authentication method requested
    #[error("Unknown authentication method")]
    UnknownMethod,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authentication token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authentication token"),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
            AuthError::UnknownMethod => (StatusCode::UNAUTHORIZED, "Unknown authentication method"),
        };

        (status, message).into_response()
    }
}

/// Authentication method
#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    /// JWT token authentication
    Jwt,
    /// API key authentication
    ApiKey,
    /// OAuth authentication
    OAuth,
    /// No authentication
    None,
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time
    pub exp: usize,
    /// Issued at time
    pub iat: usize,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// User roles
    #[serde(default)]
    pub roles: Vec<String>,
    /// User permissions
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Authenticated user context
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// User identifier
    pub user_id: String,
    /// Authentication method used
    pub method: AuthMethod,
    /// User roles
    pub roles: Vec<String>,
    /// User permissions
    pub permissions: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// JWT Configuration
#[derive(Clone)]
pub struct JwtConfig {
    /// JWT secret key
    pub secret: String,
    /// Token issuer
    pub issuer: String,
    /// Token audience
    pub audience: String,
    /// JWT algorithm
    pub algorithm: Algorithm,
}

/// API Key Configuration
#[derive(Clone)]
pub struct ApiKeyConfig {
    /// HTTP header name for API key
    pub header_name: String,
    /// Valid API keys and their information
    pub keys: Arc<HashMap<String, ApiKeyInfo>>,
}

/// API Key information
#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// API key owner
    pub owner: String,
    /// Roles assigned to this API key
    pub roles: Vec<String>,
    /// Permissions for this API key
    pub permissions: Vec<String>,
}

/// Authentication Middleware
#[derive(Clone)]
pub struct AuthMiddleware {
    jwt_config: Option<JwtConfig>,
    api_key_config: Option<ApiKeyConfig>,
    required: bool,
}

impl AuthMiddleware {
    /// Create a new authentication middleware
    pub fn new() -> Self {
        Self {
            jwt_config: None,
            api_key_config: None,
            required: true,
        }
    }

    /// Configure JWT authentication
    pub fn with_jwt(mut self, config: JwtConfig) -> Self {
        self.jwt_config = Some(config);
        self
    }

    /// Configure API Key authentication
    pub fn with_api_key(mut self, config: ApiKeyConfig) -> Self {
        self.api_key_config = Some(config);
        self
    }

    /// Make authentication optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Authenticate request
    pub async fn authenticate(
        &self,
        headers: &HeaderMap,
    ) -> Result<AuthContext, AuthError> {
        // Try JWT authentication
        if let Some(ref jwt_config) = self.jwt_config {
            if let Some(token) = extract_bearer_token(headers) {
                return self.authenticate_jwt(token, jwt_config).await;
            }
        }

        // Try API Key authentication
        if let Some(ref api_key_config) = self.api_key_config {
            if let Some(key) = headers.get(&api_key_config.header_name) {
                if let Ok(key_str) = key.to_str() {
                    return self.authenticate_api_key(key_str, api_key_config).await;
                }
            }
        }

        if self.required {
            Err(AuthError::MissingToken)
        } else {
            // Anonymous context
            Ok(AuthContext {
                user_id: "anonymous".to_string(),
                method: AuthMethod::None,
                roles: vec![],
                permissions: vec![],
                metadata: HashMap::new(),
            })
        }
    }

    /// Authenticate using JWT
    async fn authenticate_jwt(
        &self,
        token: &str,
        config: &JwtConfig,
    ) -> Result<AuthContext, AuthError> {
        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&[&config.audience]);

        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthContext {
            user_id: token_data.claims.sub,
            method: AuthMethod::Jwt,
            roles: token_data.claims.roles,
            permissions: token_data.claims.permissions,
            metadata: HashMap::new(),
        })
    }

    /// Authenticate using API Key
    async fn authenticate_api_key(
        &self,
        key: &str,
        config: &ApiKeyConfig,
    ) -> Result<AuthContext, AuthError> {
        let key_info = config
            .keys
            .get(key)
            .ok_or(AuthError::InvalidToken)?;

        Ok(AuthContext {
            user_id: key_info.owner.clone(),
            method: AuthMethod::ApiKey,
            roles: key_info.roles.clone(),
            permissions: key_info.permissions.clone(),
            metadata: HashMap::new(),
        })
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
}

/// Middleware handler
pub async fn auth_middleware(
    State(auth): State<Arc<AuthMiddleware>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let context = auth.authenticate(request.headers()).await?;

    // Add auth context to request extensions
    request.extensions_mut().insert(context);

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test-token"),
        );

        let token = extract_bearer_token(&headers);
        assert_eq!(token, Some("test-token"));
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let headers = HeaderMap::new();
        let token = extract_bearer_token(&headers);
        assert_eq!(token, None);
    }

    #[tokio::test]
    async fn test_api_key_auth() {
        let mut keys = HashMap::new();
        keys.insert(
            "test-key".to_string(),
            ApiKeyInfo {
                owner: "test-user".to_string(),
                roles: vec!["admin".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
            },
        );

        let config = ApiKeyConfig {
            header_name: "x-api-key".to_string(),
            keys: Arc::new(keys),
        };

        let middleware = AuthMiddleware::new().with_api_key(config);

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("test-key"));

        let context = middleware.authenticate(&headers).await.unwrap();
        assert_eq!(context.user_id, "test-user");
        assert_eq!(context.method, AuthMethod::ApiKey);
        assert!(context.roles.contains(&"admin".to_string()));
    }
}
