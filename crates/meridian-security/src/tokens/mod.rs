//! Token management for authentication and authorization
//!
//! Provides enterprise-grade token management:
//! - JWT (JSON Web Tokens) for stateless authentication
//! - Refresh tokens for long-lived sessions
//! - Token rotation and revocation
//! - Claims-based authorization
//!
//! ## OWASP Compliance
//! - Short-lived access tokens (15 minutes)
//! - Secure refresh token storage
//! - Token binding to prevent theft
//! - Proper signature verification

pub mod jwt;
pub mod refresh;

pub use jwt::{Claims, JwtManager};
pub use refresh::{RefreshToken, RefreshTokenManager};
