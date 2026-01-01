//! Secure hashing and message authentication
//!
//! Provides enterprise-grade hashing capabilities:
//! - Password hashing with Argon2id
//! - HMAC for message authentication
//! - Constant-time comparison to prevent timing attacks
//!
//! ## OWASP Compliance
//! - Argon2id for password storage (recommended by OWASP)
//! - Strong salt generation
//! - Sufficient computational cost
//! - Timing-attack resistant verification

pub mod hmac;
pub mod password;

pub use hmac::HmacVerifier;
pub use password::PasswordHasher;
