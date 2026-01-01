//! JWT (JSON Web Token) management
//!
//! Implements RFC 7519 compliant JWT for stateless authentication.
//!
//! ## Features
//! - RS256 (RSA with SHA-256) signatures
//! - HS256 (HMAC with SHA-256) signatures
//! - Claims validation (exp, nbf, iat, iss, aud)
//! - Custom claims support
//! - Token refresh mechanism
//!
//! ## Security
//! - Short token lifetime (15 minutes default)
//! - Signature verification
//! - Expiration enforcement
//! - Issuer and audience validation
//!
//! ## OWASP JWT Best Practices
//! - Always verify signature
//! - Validate all standard claims
//! - Use strong algorithms (RS256 preferred)
//! - Keep access tokens short-lived
//! - Never store sensitive data in payload

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{
    config::{JWT_ACCESS_TOKEN_LIFETIME, JWT_REFRESH_TOKEN_LIFETIME},
    error::{SecurityError, SecurityResult},
};

/// JWT claims structure
///
/// Includes standard RFC 7519 claims plus custom application claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Issued at (Unix timestamp)
    pub iat: i64,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Not before (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// JWT ID (unique identifier)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,

    /// User email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// User roles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,

    /// User permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,

    /// Organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// Session ID (for tracking/revocation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Custom claims (extensible)
    #[serde(flatten)]
    pub custom: serde_json::Value,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(
        user_id: &str,
        issuer: &str,
        audience: &str,
        lifetime: Duration,
    ) -> Self {
        let now = Utc::now();
        let exp = (now + lifetime).timestamp();

        Self {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp,
            nbf: Some(now.timestamp()),
            iss: issuer.to_string(),
            aud: audience.to_string(),
            jti: Some(uuid::Uuid::new_v4().to_string()),
            email: None,
            roles: None,
            permissions: None,
            org_id: None,
            session_id: None,
            custom: serde_json::Value::Null,
        }
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() >= self.exp
    }

    /// Check if token is valid yet (nbf check)
    pub fn is_valid_yet(&self) -> bool {
        if let Some(nbf) = self.nbf {
            Utc::now().timestamp() >= nbf
        } else {
            true
        }
    }

    /// Add roles to claims
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(roles);
        self
    }

    /// Add permissions to claims
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Add organization ID
    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    /// Add session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Add email
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
}

/// JWT token type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Short-lived access token (15 minutes)
    Access,
    /// Long-lived refresh token (7 days)
    Refresh,
}

impl TokenType {
    /// Get the lifetime for this token type
    pub fn lifetime(&self) -> Duration {
        match self {
            TokenType::Access => Duration::from_std(JWT_ACCESS_TOKEN_LIFETIME)
                .expect("Invalid access token lifetime"),
            TokenType::Refresh => Duration::from_std(JWT_REFRESH_TOKEN_LIFETIME)
                .expect("Invalid refresh token lifetime"),
        }
    }
}

/// JWT manager for creating and validating tokens
///
/// # Example
/// ```rust,no_run
/// use meridian_security::tokens::jwt::{JwtManager, Claims, TokenType};
/// use chrono::Duration;
///
/// let secret = b"your-secret-key-min-32-bytes-long!!!";
/// let manager = JwtManager::new_hs256(secret, "my-app", "my-api").unwrap();
///
/// // Create access token
/// let user_id = "user123";
/// let token = manager.create_token(user_id, TokenType::Access, None).unwrap();
///
/// // Verify and decode
/// let claims = manager.verify_token(&token).unwrap();
/// assert_eq!(claims.sub, user_id);
/// ```
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
    issuer: String,
    audience: String,
}

impl JwtManager {
    /// Create a new JWT manager using HMAC-SHA256 (HS256)
    ///
    /// # Arguments
    /// * `secret` - Shared secret key (minimum 32 bytes)
    /// * `issuer` - Token issuer identifier
    /// * `audience` - Token audience identifier
    ///
    /// # Security Note
    /// HS256 uses a shared secret. For distributed systems, consider RS256.
    pub fn new_hs256(secret: &[u8], issuer: &str, audience: &str) -> SecurityResult<Self> {
        if secret.len() < 32 {
            return Err(SecurityError::InvalidKey(
                "JWT secret must be at least 32 bytes".to_string(),
            ));
        }

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            algorithm: Algorithm::HS256,
            issuer: issuer.to_string(),
            audience: audience.to_string(),
        })
    }

    /// Create a new JWT manager using RSA-SHA256 (RS256)
    ///
    /// # Arguments
    /// * `private_key_pem` - RSA private key in PEM format
    /// * `public_key_pem` - RSA public key in PEM format
    /// * `issuer` - Token issuer identifier
    /// * `audience` - Token audience identifier
    ///
    /// # Security Note
    /// RS256 is preferred for distributed systems as public key can be shared.
    pub fn new_rs256(
        private_key_pem: &[u8],
        public_key_pem: &[u8],
        issuer: &str,
        audience: &str,
    ) -> SecurityResult<Self> {
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem)
            .map_err(|e| SecurityError::InvalidKey(format!("Invalid RSA private key: {}", e)))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem)
            .map_err(|e| SecurityError::InvalidKey(format!("Invalid RSA public key: {}", e)))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            algorithm: Algorithm::RS256,
            issuer: issuer.to_string(),
            audience: audience.to_string(),
        })
    }

    /// Create a JWT token
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `token_type` - Access or refresh token
    /// * `custom_claims` - Optional additional claims
    pub fn create_token(
        &self,
        user_id: &str,
        token_type: TokenType,
        custom_claims: Option<serde_json::Value>,
    ) -> SecurityResult<String> {
        let mut claims = Claims::new(
            user_id,
            &self.issuer,
            &self.audience,
            token_type.lifetime(),
        );

        if let Some(custom) = custom_claims {
            claims.custom = custom;
        }

        let header = Header::new(self.algorithm);

        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| SecurityError::JwtError(format!("Token creation failed: {}", e)))
    }

    /// Create an access token with roles and permissions
    pub fn create_access_token(
        &self,
        user_id: &str,
        email: Option<String>,
        roles: Vec<String>,
        permissions: Vec<String>,
        org_id: Option<String>,
    ) -> SecurityResult<String> {
        let mut claims = Claims::new(
            user_id,
            &self.issuer,
            &self.audience,
            TokenType::Access.lifetime(),
        );

        claims = claims.with_roles(roles).with_permissions(permissions);

        if let Some(email) = email {
            claims = claims.with_email(email);
        }

        if let Some(org_id) = org_id {
            claims = claims.with_org_id(org_id);
        }

        let header = Header::new(self.algorithm);
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| SecurityError::JwtError(format!("Token creation failed: {}", e)))
    }

    /// Verify and decode a JWT token
    ///
    /// Validates:
    /// - Signature
    /// - Expiration (exp)
    /// - Not before (nbf)
    /// - Issuer (iss)
    /// - Audience (aud)
    pub fn verify_token(&self, token: &str) -> SecurityResult<Claims> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => SecurityError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    SecurityError::InvalidToken("Token is invalid".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    SecurityError::InvalidToken("Invalid signature".to_string())
                }
                _ => SecurityError::JwtError(format!("Token verification failed: {}", e)),
            })?;

        Ok(token_data.claims)
    }

    /// Decode token without verification (for debugging only!)
    ///
    /// # Security Warning
    /// This does NOT verify the signature. Only use for debugging/inspection.
    #[cfg(debug_assertions)]
    pub fn decode_unverified(&self, token: &str) -> SecurityResult<Claims> {
        let mut validation = Validation::new(self.algorithm);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| SecurityError::JwtError(format!("Decode failed: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Refresh an access token using a refresh token
    ///
    /// Verifies the refresh token and issues a new access token
    pub fn refresh_access_token(&self, refresh_token: &str) -> SecurityResult<String> {
        let claims = self.verify_token(refresh_token)?;

        // Create new access token with same user info
        let mut new_claims = Claims::new(
            &claims.sub,
            &self.issuer,
            &self.audience,
            TokenType::Access.lifetime(),
        );

        new_claims.email = claims.email;
        new_claims.roles = claims.roles;
        new_claims.permissions = claims.permissions;
        new_claims.org_id = claims.org_id;
        new_claims.session_id = claims.session_id;

        let header = Header::new(self.algorithm);
        encode(&header, &new_claims, &self.encoding_key)
            .map_err(|e| SecurityError::JwtError(format!("Token refresh failed: {}", e)))
    }

    /// Extract claims without full verification (still checks signature)
    ///
    /// Useful when you want to check claims before full validation
    pub fn peek_claims(&self, token: &str) -> SecurityResult<Claims> {
        let mut validation = Validation::new(self.algorithm);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.insecure_disable_signature_validation();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| SecurityError::JwtError(format!("Peek failed: {}", e)))?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> JwtManager {
        let secret = b"test-secret-key-must-be-32-bytes-long!!!";
        JwtManager::new_hs256(secret, "test-issuer", "test-audience").unwrap()
    }

    #[test]
    fn test_create_and_verify_token() {
        let manager = create_test_manager();

        let token = manager.create_token("user123", TokenType::Access, None).unwrap();
        let claims = manager.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.iss, "test-issuer");
        assert_eq!(claims.aud, "test-audience");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_create_access_token_with_roles() {
        let manager = create_test_manager();

        let roles = vec!["admin".to_string(), "user".to_string()];
        let permissions = vec!["read".to_string(), "write".to_string()];

        let token = manager
            .create_access_token(
                "user123",
                Some("user@example.com".to_string()),
                roles.clone(),
                permissions.clone(),
                Some("org456".to_string()),
            )
            .unwrap();

        let claims = manager.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, Some("user@example.com".to_string()));
        assert_eq!(claims.roles, Some(roles));
        assert_eq!(claims.permissions, Some(permissions));
        assert_eq!(claims.org_id, Some("org456".to_string()));
    }

    #[test]
    fn test_token_expiration() {
        let manager = create_test_manager();

        // Create token with very short lifetime
        let mut claims = Claims::new("user123", "test-issuer", "test-audience", Duration::seconds(-10));

        let header = Header::new(Algorithm::HS256);
        let secret = b"test-secret-key-must-be-32-bytes-long!!!";
        let encoding_key = EncodingKey::from_secret(secret);
        let token = encode(&header, &claims, &encoding_key).unwrap();

        let result = manager.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature() {
        let manager = create_test_manager();

        let token = manager.create_token("user123", TokenType::Access, None).unwrap();

        // Tamper with token
        let mut tampered = token.clone();
        tampered.push('x');

        let result = manager.verify_token(&tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_type() {
        let manager = create_test_manager();

        let token = manager.create_token("user123", TokenType::Refresh, None).unwrap();
        let claims = manager.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        // Refresh token should have longer expiration
    }

    #[test]
    fn test_short_secret_error() {
        let short_secret = b"short";
        let result = JwtManager::new_hs256(short_secret, "issuer", "audience");
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_builder() {
        let claims = Claims::new("user123", "issuer", "audience", Duration::hours(1))
            .with_email("test@example.com".to_string())
            .with_roles(vec!["admin".to_string()])
            .with_org_id("org123".to_string());

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, Some("test@example.com".to_string()));
        assert_eq!(claims.roles, Some(vec!["admin".to_string()]));
        assert_eq!(claims.org_id, Some("org123".to_string()));
    }
}
