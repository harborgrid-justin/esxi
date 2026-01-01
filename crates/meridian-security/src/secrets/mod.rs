//! Secrets management
//!
//! Secure storage and retrieval of sensitive data:
//! - API keys
//! - Database credentials
//! - Service tokens
//! - Certificates
//! - Encryption keys
//!
//! ## Security Features
//! - Encrypted at rest
//! - Access control
//! - Audit logging
//! - Versioning
//! - Automatic rotation
//!
//! ## Integration
//! This module can integrate with external secret management systems:
//! - HashiCorp Vault
//! - AWS Secrets Manager
//! - Azure Key Vault
//! - Google Secret Manager

pub mod vault;

pub use vault::{Secret, SecretMetadata, SecretsVault};
