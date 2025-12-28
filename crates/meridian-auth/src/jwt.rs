//! JWT token handling for authentication

use crate::error::{AuthError, AuthResult};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Not before (Unix timestamp)
    pub nbf: i64,
    /// JWT ID (unique identifier)
    pub jti: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// User email
    pub email: Option<String>,
    /// User roles
    pub roles: Vec<String>,
    /// Custom claims
    #[serde(flatten)]
    pub custom: serde_json::Value,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        issuer: String,
        audience: String,
        expires_in: Duration,
    ) -> Self {
        let now = Utc::now();
        let exp = now + expires_in;

        Self {
            sub: user_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            iss: issuer,
            aud: audience,
            email,
            roles,
            custom: serde_json::Value::Null,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() >= self.exp
    }

    /// Get expiration time
    pub fn expiration(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_else(|| Utc::now())
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.roles.iter().any(|r| roles.contains(&r.as_str()))
    }
}

/// Refresh token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// JWT ID (unique identifier)
    pub jti: String,
    /// Token family ID (for rotation)
    pub family: String,
}

impl RefreshClaims {
    /// Create new refresh token claims
    pub fn new(user_id: String, expires_in: Duration) -> Self {
        let now = Utc::now();
        let exp = now + expires_in;

        Self {
            sub: user_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            family: Uuid::new_v4().to_string(),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() >= self.exp
    }
}

/// JWT token manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
    issuer: String,
    audience: String,
    access_token_duration: Duration,
    refresh_token_duration: Duration,
    revoked_tokens: HashSet<String>,
}

impl JwtManager {
    /// Create a new JWT manager with HMAC SHA256
    pub fn new_with_secret(
        secret: &[u8],
        issuer: String,
        audience: String,
    ) -> AuthResult<Self> {
        if secret.len() < 32 {
            return Err(AuthError::ConfigurationError(
                "JWT secret must be at least 32 bytes".to_string(),
            ));
        }

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            algorithm: Algorithm::HS256,
            issuer,
            audience,
            access_token_duration: Duration::hours(1),
            refresh_token_duration: Duration::days(30),
            revoked_tokens: HashSet::new(),
        })
    }

    /// Create a new JWT manager with RSA keys
    pub fn new_with_rsa(
        private_key_pem: &[u8],
        public_key_pem: &[u8],
        issuer: String,
        audience: String,
    ) -> AuthResult<Self> {
        Ok(Self {
            encoding_key: EncodingKey::from_rsa_pem(private_key_pem)
                .map_err(|e| AuthError::ConfigurationError(e.to_string()))?,
            decoding_key: DecodingKey::from_rsa_pem(public_key_pem)
                .map_err(|e| AuthError::ConfigurationError(e.to_string()))?,
            algorithm: Algorithm::RS256,
            issuer,
            audience,
            access_token_duration: Duration::hours(1),
            refresh_token_duration: Duration::days(30),
            revoked_tokens: HashSet::new(),
        })
    }

    /// Set access token duration
    pub fn set_access_token_duration(&mut self, duration: Duration) {
        self.access_token_duration = duration;
    }

    /// Set refresh token duration
    pub fn set_refresh_token_duration(&mut self, duration: Duration) {
        self.refresh_token_duration = duration;
    }

    /// Generate an access token
    pub fn generate_access_token(
        &self,
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
    ) -> AuthResult<String> {
        let claims = Claims::new(
            user_id,
            email,
            roles,
            self.issuer.clone(),
            self.audience.clone(),
            self.access_token_duration,
        );

        let header = Header::new(self.algorithm);
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(&self, user_id: String) -> AuthResult<String> {
        let claims = RefreshClaims::new(user_id, self.refresh_token_duration);

        let header = Header::new(self.algorithm);
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
    }

    /// Generate both access and refresh tokens
    pub fn generate_token_pair(
        &self,
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
    ) -> AuthResult<TokenPair> {
        let access_token = self.generate_access_token(user_id.clone(), email, roles)?;
        let refresh_token = self.generate_refresh_token(user_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
        })
    }

    /// Validate and decode an access token
    pub fn validate_access_token(&self, token: &str) -> AuthResult<Claims> {
        // Check if token is revoked
        if self.is_token_revoked(token) {
            return Err(AuthError::TokenRevoked);
        }

        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        Ok(token_data.claims)
    }

    /// Validate and decode a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> AuthResult<RefreshClaims> {
        // Check if token is revoked
        if self.is_token_revoked(token) {
            return Err(AuthError::TokenRevoked);
        }

        let mut validation = Validation::new(self.algorithm);
        validation.set_required_spec_claims(&["sub", "exp", "iat"]);

        let token_data = decode::<RefreshClaims>(token, &self.decoding_key, &validation)?;

        if token_data.claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        Ok(token_data.claims)
    }

    /// Refresh access token using a refresh token
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        email: Option<String>,
        roles: Vec<String>,
    ) -> AuthResult<String> {
        let refresh_claims = self.validate_refresh_token(refresh_token)?;
        self.generate_access_token(refresh_claims.sub, email, roles)
    }

    /// Revoke a token
    pub fn revoke_token(&mut self, token: &str) -> AuthResult<()> {
        // Extract JWT ID from token without full validation
        let jti = self.extract_jti(token)?;
        self.revoked_tokens.insert(jti);
        Ok(())
    }

    /// Check if token is revoked
    pub fn is_token_revoked(&self, token: &str) -> bool {
        if let Ok(jti) = self.extract_jti(token) {
            self.revoked_tokens.contains(&jti)
        } else {
            false
        }
    }

    /// Extract JWT ID from token
    fn extract_jti(&self, token: &str) -> AuthResult<String> {
        // Decode without validation to extract JTI
        let mut validation = Validation::new(self.algorithm);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.validate_nbf = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .or_else(|_| {
                decode::<RefreshClaims>(token, &self.decoding_key, &validation)
                    .map(|td| {
                        // Convert RefreshClaims to Claims-like structure for JTI extraction
                        jsonwebtoken::TokenData {
                            header: td.header,
                            claims: Claims {
                                sub: td.claims.sub.clone(),
                                exp: td.claims.exp,
                                iat: td.claims.iat,
                                nbf: 0,
                                jti: td.claims.jti.clone(),
                                iss: String::new(),
                                aud: String::new(),
                                email: None,
                                roles: Vec::new(),
                                custom: serde_json::Value::Null,
                            },
                        }
                    })
            })
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(token_data.claims.jti)
    }

    /// Clear all revoked tokens (for maintenance)
    pub fn clear_revoked_tokens(&mut self) {
        self.revoked_tokens.clear();
    }

    /// Get number of revoked tokens
    pub fn revoked_tokens_count(&self) -> usize {
        self.revoked_tokens.len()
    }
}

/// Token pair containing access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> JwtManager {
        let secret = b"test_secret_key_must_be_at_least_32_bytes_long!!!";
        JwtManager::new_with_secret(
            secret,
            "test-issuer".to_string(),
            "test-audience".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new(
            "user123".to_string(),
            Some("user@example.com".to_string()),
            vec!["admin".to_string()],
            "issuer".to_string(),
            "audience".to_string(),
            Duration::hours(1),
        );

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, Some("user@example.com".to_string()));
        assert!(claims.has_role("admin"));
        assert!(!claims.has_role("viewer"));
    }

    #[test]
    fn test_access_token_generation() {
        let manager = create_test_manager();
        let token = manager
            .generate_access_token(
                "user123".to_string(),
                Some("user@example.com".to_string()),
                vec!["admin".to_string()],
            )
            .unwrap();

        assert!(!token.is_empty());
        assert!(token.split('.').count() == 3); // JWT has 3 parts
    }

    #[test]
    fn test_access_token_validation() {
        let manager = create_test_manager();
        let token = manager
            .generate_access_token(
                "user123".to_string(),
                Some("user@example.com".to_string()),
                vec!["admin".to_string()],
            )
            .unwrap();

        let claims = manager.validate_access_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, Some("user@example.com".to_string()));
        assert!(claims.has_role("admin"));
    }

    #[test]
    fn test_refresh_token_generation() {
        let manager = create_test_manager();
        let token = manager.generate_refresh_token("user123".to_string()).unwrap();

        assert!(!token.is_empty());
        assert!(token.split('.').count() == 3);
    }

    #[test]
    fn test_token_pair_generation() {
        let manager = create_test_manager();
        let pair = manager
            .generate_token_pair(
                "user123".to_string(),
                Some("user@example.com".to_string()),
                vec!["admin".to_string()],
            )
            .unwrap();

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
    }

    #[test]
    fn test_token_revocation() {
        let mut manager = create_test_manager();
        let token = manager
            .generate_access_token(
                "user123".to_string(),
                None,
                vec!["admin".to_string()],
            )
            .unwrap();

        // Token should be valid initially
        assert!(manager.validate_access_token(&token).is_ok());

        // Revoke token
        manager.revoke_token(&token).unwrap();

        // Token should now be revoked
        assert!(matches!(
            manager.validate_access_token(&token),
            Err(AuthError::TokenRevoked)
        ));
    }

    #[test]
    fn test_invalid_token() {
        let manager = create_test_manager();
        let result = manager.validate_access_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_refresh() {
        let manager = create_test_manager();
        let refresh_token = manager.generate_refresh_token("user123".to_string()).unwrap();

        let new_access_token = manager
            .refresh_access_token(
                &refresh_token,
                Some("user@example.com".to_string()),
                vec!["admin".to_string()],
            )
            .unwrap();

        assert!(!new_access_token.is_empty());

        let claims = manager.validate_access_token(&new_access_token).unwrap();
        assert_eq!(claims.sub, "user123");
    }
}
