//! HMAC-based message authentication
//!
//! Provides Hash-based Message Authentication Code (HMAC) for:
//! - Message integrity verification
//! - Message authenticity verification
//! - Preventing tampering attacks
//! - API request signing
//!
//! ## Security Properties
//! - HMAC-SHA256 (256-bit output)
//! - Cryptographically secure key derivation
//! - Constant-time comparison
//! - Resistance to length extension attacks
//!
//! ## Use Cases
//! - API authentication (AWS Signature v4 style)
//! - Webhook verification (GitHub, Stripe style)
//! - Cookie integrity
//! - Message queues
//! - Signed URLs

use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::{
    config::HMAC_KEY_SIZE,
    error::{SecurityError, SecurityResult},
};

/// HMAC-SHA256 verifier for message authentication
///
/// # Example
/// ```rust,no_run
/// use meridian_security::hashing::hmac::HmacVerifier;
///
/// let key = HmacVerifier::generate_key().unwrap();
/// let verifier = HmacVerifier::new(&key).unwrap();
///
/// let message = b"Important API request data";
/// let signature = verifier.sign(message).unwrap();
///
/// // Later, verify the message hasn't been tampered with
/// assert!(verifier.verify(message, &signature).unwrap());
/// ```
pub struct HmacVerifier {
    key: Zeroizing<Vec<u8>>,
}

impl HmacVerifier {
    /// Create a new HMAC verifier with the given key
    ///
    /// # Arguments
    /// * `key` - Secret key (minimum 32 bytes recommended)
    pub fn new(key: &[u8]) -> SecurityResult<Self> {
        if key.len() < HMAC_KEY_SIZE {
            return Err(SecurityError::InvalidKey(format!(
                "HMAC key should be at least {} bytes, got {}",
                HMAC_KEY_SIZE,
                key.len()
            )));
        }

        Ok(Self {
            key: Zeroizing::new(key.to_vec()),
        })
    }

    /// Generate a new cryptographically secure HMAC key
    pub fn generate_key() -> SecurityResult<Vec<u8>> {
        use rand::RngCore;
        let mut key = Zeroizing::new(vec![0u8; HMAC_KEY_SIZE]);
        rand::thread_rng().fill_bytes(&mut key);
        Ok(key.to_vec())
    }

    /// Sign a message and return the HMAC signature
    ///
    /// # Arguments
    /// * `message` - Data to sign
    ///
    /// # Returns
    /// 32-byte HMAC-SHA256 signature
    pub fn sign(&self, message: &[u8]) -> SecurityResult<Vec<u8>> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.key)
            .map_err(|e| SecurityError::CryptoError(format!("HMAC init failed: {}", e)))?;

        mac.update(message);
        let result = mac.finalize();
        Ok(result.into_bytes().to_vec())
    }

    /// Verify a message signature using constant-time comparison
    ///
    /// # Arguments
    /// * `message` - Original message
    /// * `signature` - HMAC signature to verify
    ///
    /// # Returns
    /// `Ok(true)` if signature is valid, `Ok(false)` if invalid
    ///
    /// # Security
    /// Uses constant-time comparison to prevent timing attacks
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> SecurityResult<bool> {
        let expected = self.sign(message)?;

        // Constant-time comparison
        if expected.len() != signature.len() {
            return Ok(false);
        }

        let mut mac = Hmac::<Sha256>::new_from_slice(&self.key)
            .map_err(|e| SecurityError::CryptoError(format!("HMAC init failed: {}", e)))?;

        mac.update(message);

        match mac.verify_slice(signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Sign a message and return hex-encoded signature
    ///
    /// Useful for HTTP headers, URLs, etc.
    pub fn sign_hex(&self, message: &[u8]) -> SecurityResult<String> {
        let signature = self.sign(message)?;
        Ok(hex::encode(signature))
    }

    /// Verify a hex-encoded signature
    pub fn verify_hex(&self, message: &[u8], signature_hex: &str) -> SecurityResult<bool> {
        let signature = hex::decode(signature_hex)
            .map_err(|e| SecurityError::CryptoError(format!("Invalid hex signature: {}", e)))?;

        self.verify(message, &signature)
    }

    /// Sign a message and return base64-encoded signature
    ///
    /// Common for API authentication (AWS, OAuth)
    pub fn sign_base64(&self, message: &[u8]) -> SecurityResult<String> {
        let signature = self.sign(message)?;
        Ok(base64::encode(signature))
    }

    /// Verify a base64-encoded signature
    pub fn verify_base64(&self, message: &[u8], signature_b64: &str) -> SecurityResult<bool> {
        let signature = base64::decode(signature_b64)
            .map_err(|e| SecurityError::CryptoError(format!("Invalid base64 signature: {}", e)))?;

        self.verify(message, &signature)
    }
}

/// API request signer for webhook/API authentication
///
/// Implements patterns similar to AWS Signature v4, GitHub webhooks, etc.
pub struct ApiRequestSigner {
    verifier: HmacVerifier,
}

impl ApiRequestSigner {
    /// Create a new API request signer
    pub fn new(secret: &[u8]) -> SecurityResult<Self> {
        Ok(Self {
            verifier: HmacVerifier::new(secret)?,
        })
    }

    /// Sign an API request
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Request path
    /// * `body` - Request body
    /// * `timestamp` - Unix timestamp (prevents replay attacks)
    ///
    /// # Returns
    /// Hex-encoded signature
    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        timestamp: i64,
    ) -> SecurityResult<String> {
        // Build canonical request
        let canonical = format!("{}\n{}\n{}\n", method.to_uppercase(), path, timestamp);

        let mut message = canonical.into_bytes();
        message.extend_from_slice(body);

        self.verifier.sign_hex(&message)
    }

    /// Verify an API request signature
    ///
    /// # Arguments
    /// * `method` - HTTP method
    /// * `path` - Request path
    /// * `body` - Request body
    /// * `timestamp` - Timestamp from request
    /// * `signature` - Signature to verify
    /// * `max_age_seconds` - Maximum allowed age for timestamp (prevents replay)
    pub fn verify_request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        timestamp: i64,
        signature: &str,
        max_age_seconds: i64,
    ) -> SecurityResult<bool> {
        // Check timestamp is not too old (prevent replay attacks)
        let now = chrono::Utc::now().timestamp();
        if (now - timestamp).abs() > max_age_seconds {
            return Ok(false);
        }

        let canonical = format!("{}\n{}\n{}\n", method.to_uppercase(), path, timestamp);

        let mut message = canonical.into_bytes();
        message.extend_from_slice(body);

        self.verifier.verify_hex(&message, signature)
    }

    /// Sign a webhook payload (GitHub/Stripe style)
    ///
    /// # Arguments
    /// * `payload` - JSON or raw payload
    /// * `timestamp` - Optional timestamp
    ///
    /// # Returns
    /// Signature in format: "sha256=<hex_signature>"
    pub fn sign_webhook(&self, payload: &[u8], timestamp: Option<i64>) -> SecurityResult<String> {
        let message = if let Some(ts) = timestamp {
            let mut msg = ts.to_string().into_bytes();
            msg.push(b'.');
            msg.extend_from_slice(payload);
            msg
        } else {
            payload.to_vec()
        };

        let signature = self.verifier.sign_hex(&message)?;
        Ok(format!("sha256={}", signature))
    }

    /// Verify a webhook signature
    pub fn verify_webhook(
        &self,
        payload: &[u8],
        signature: &str,
        timestamp: Option<i64>,
    ) -> SecurityResult<bool> {
        // Strip "sha256=" prefix if present
        let sig_hex = signature.strip_prefix("sha256=").unwrap_or(signature);

        let message = if let Some(ts) = timestamp {
            let mut msg = ts.to_string().into_bytes();
            msg.push(b'.');
            msg.extend_from_slice(payload);
            msg
        } else {
            payload.to_vec()
        };

        self.verifier.verify_hex(&message, sig_hex)
    }
}

/// Signed URL generator for temporary access to resources
pub struct SignedUrlGenerator {
    verifier: HmacVerifier,
}

impl SignedUrlGenerator {
    /// Create a new signed URL generator
    pub fn new(secret: &[u8]) -> SecurityResult<Self> {
        Ok(Self {
            verifier: HmacVerifier::new(secret)?,
        })
    }

    /// Generate a signed URL
    ///
    /// # Arguments
    /// * `base_url` - Base URL (e.g., "/download/file.pdf")
    /// * `expires_at` - Unix timestamp when URL expires
    ///
    /// # Returns
    /// URL with signature: "/download/file.pdf?expires=123&signature=abc"
    pub fn generate_url(&self, base_url: &str, expires_at: i64) -> SecurityResult<String> {
        let message = format!("{}:{}", base_url, expires_at);
        let signature = self.verifier.sign_hex(message.as_bytes())?;

        Ok(format!(
            "{}?expires={}&signature={}",
            base_url, expires_at, signature
        ))
    }

    /// Verify a signed URL
    pub fn verify_url(&self, base_url: &str, expires_at: i64, signature: &str) -> SecurityResult<bool> {
        // Check if URL has expired
        let now = chrono::Utc::now().timestamp();
        if now > expires_at {
            return Ok(false);
        }

        let message = format!("{}:{}", base_url, expires_at);
        self.verifier.verify_hex(message.as_bytes(), signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sign_verify() {
        let key = HmacVerifier::generate_key().unwrap();
        let verifier = HmacVerifier::new(&key).unwrap();

        let message = b"Test message";
        let signature = verifier.sign(message).unwrap();

        assert_eq!(signature.len(), 32); // SHA-256 output
        assert!(verifier.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_hmac_wrong_message() {
        let key = HmacVerifier::generate_key().unwrap();
        let verifier = HmacVerifier::new(&key).unwrap();

        let message = b"Original message";
        let tampered = b"Tampered message";

        let signature = verifier.sign(message).unwrap();
        assert!(!verifier.verify(tampered, &signature).unwrap());
    }

    #[test]
    fn test_hmac_hex_encoding() {
        let key = HmacVerifier::generate_key().unwrap();
        let verifier = HmacVerifier::new(&key).unwrap();

        let message = b"API request data";
        let signature = verifier.sign_hex(message).unwrap();

        assert_eq!(signature.len(), 64); // 32 bytes in hex
        assert!(verifier.verify_hex(message, &signature).unwrap());
    }

    #[test]
    fn test_hmac_base64_encoding() {
        let key = HmacVerifier::generate_key().unwrap();
        let verifier = HmacVerifier::new(&key).unwrap();

        let message = b"Authorization header";
        let signature = verifier.sign_base64(message).unwrap();

        assert!(verifier.verify_base64(message, &signature).unwrap());
    }

    #[test]
    fn test_api_request_signing() {
        let secret = HmacVerifier::generate_key().unwrap();
        let signer = ApiRequestSigner::new(&secret).unwrap();

        let method = "POST";
        let path = "/api/v1/users";
        let body = br#"{"name":"John Doe"}"#;
        let timestamp = chrono::Utc::now().timestamp();

        let signature = signer.sign_request(method, path, body, timestamp).unwrap();

        let valid = signer
            .verify_request(method, path, body, timestamp, &signature, 300)
            .unwrap();
        assert!(valid);
    }

    #[test]
    fn test_api_request_replay_protection() {
        let secret = HmacVerifier::generate_key().unwrap();
        let signer = ApiRequestSigner::new(&secret).unwrap();

        let method = "POST";
        let path = "/api/v1/transfer";
        let body = b"amount=1000";
        let old_timestamp = chrono::Utc::now().timestamp() - 600; // 10 minutes ago

        let signature = signer.sign_request(method, path, body, old_timestamp).unwrap();

        // Should fail due to timestamp too old
        let valid = signer
            .verify_request(method, path, body, old_timestamp, &signature, 300)
            .unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_webhook_signing() {
        let secret = HmacVerifier::generate_key().unwrap();
        let signer = ApiRequestSigner::new(&secret).unwrap();

        let payload = br#"{"event":"user.created","user_id":123}"#;
        let timestamp = chrono::Utc::now().timestamp();

        let signature = signer.sign_webhook(payload, Some(timestamp)).unwrap();
        assert!(signature.starts_with("sha256="));

        let valid = signer
            .verify_webhook(payload, &signature, Some(timestamp))
            .unwrap();
        assert!(valid);
    }

    #[test]
    fn test_signed_url() {
        let secret = HmacVerifier::generate_key().unwrap();
        let generator = SignedUrlGenerator::new(&secret).unwrap();

        let base_url = "/download/secret-file.pdf";
        let expires_at = chrono::Utc::now().timestamp() + 3600; // 1 hour

        let signed_url = generator.generate_url(base_url, expires_at).unwrap();
        assert!(signed_url.contains("expires="));
        assert!(signed_url.contains("signature="));

        // Parse the signature from URL
        let parts: Vec<&str> = signed_url.split("signature=").collect();
        let signature = parts[1];

        let valid = generator
            .verify_url(base_url, expires_at, signature)
            .unwrap();
        assert!(valid);
    }

    #[test]
    fn test_signed_url_expired() {
        let secret = HmacVerifier::generate_key().unwrap();
        let generator = SignedUrlGenerator::new(&secret).unwrap();

        let base_url = "/download/file.pdf";
        let expires_at = chrono::Utc::now().timestamp() - 10; // Expired 10 seconds ago

        let signed_url = generator.generate_url(base_url, expires_at).unwrap();
        let parts: Vec<&str> = signed_url.split("signature=").collect();
        let signature = parts[1];

        let valid = generator
            .verify_url(base_url, expires_at, signature)
            .unwrap();
        assert!(!valid); // Should fail due to expiration
    }

    #[test]
    fn test_key_too_short() {
        let short_key = vec![0u8; 16];
        let result = HmacVerifier::new(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_signature() {
        let key = vec![0x42u8; 32];
        let verifier = HmacVerifier::new(&key).unwrap();

        let message = b"Consistent message";
        let sig1 = verifier.sign(message).unwrap();
        let sig2 = verifier.sign(message).unwrap();

        // Same key + same message = same signature
        assert_eq!(sig1, sig2);
    }
}
