# Meridian Security

Enterprise-grade security module for the $983M Meridian Enterprise SaaS Platform v0.5.

## Overview

This crate provides comprehensive security capabilities following OWASP best practices, NIST standards, and compliance requirements (SOC 2, ISO 27001, GDPR, HIPAA).

## Features

### 🔐 Encryption
- **AES-256-GCM**: Hardware-accelerated authenticated encryption
- **ChaCha20-Poly1305**: Software-optimized AEAD for mobile/embedded
- **Envelope Encryption**: Key hierarchy with KEK/DEK separation
- NIST-approved algorithms with proper nonce handling

### 🔑 Key Management System (KMS)
- Secure keyring with version tracking
- Automatic key rotation (90-day intervals)
- Key derivation functions (HKDF, PBKDF2, Argon2)
- Key lifecycle management (generation, storage, rotation, revocation)

### 🔒 Hashing
- **Password Hashing**: Argon2id (OWASP recommended)
  - Memory cost: 47 MiB
  - Iterations: 1
  - Resistant to GPU/ASIC attacks
- **HMAC**: Message authentication and integrity
  - API request signing
  - Webhook verification
  - Signed URLs

### 🎫 Token Management
- **JWT**: Stateless authentication with RS256/HS256
  - Short-lived access tokens (15 min)
  - Claims-based authorization
  - Signature verification
- **Refresh Tokens**: Secure session management
  - Token rotation on each use
  - Family-based revocation
  - Compromise detection

### 🛡️ Zero-Trust Security
- Policy-based access control (PBAC)
- Context-aware security decisions
- Risk scoring and trust levels
- Device posture evaluation
- Network trust assessment
- Continuous verification

### 📋 Audit Logging
- Comprehensive security event logging
- Compliance-ready audit trails
- Tamper-evident logs
- Multiple output destinations
- SOC 2 / ISO 27001 compliant

### 🔐 Secrets Management
- Encrypted secrets vault
- Version control for secrets
- TTL and expiration
- Automatic rotation
- Access tracking

## Security Standards

### OWASP Compliance
- ✅ OWASP Top 10 2021
- ✅ OWASP ASVS Level 3
- ✅ OWASP Cryptographic Storage Cheat Sheet
- ✅ OWASP Authentication Cheat Sheet
- ✅ OWASP Session Management Cheat Sheet

### NIST Standards
- ✅ NIST SP 800-57 (Key Management)
- ✅ NIST SP 800-63B (Digital Identity)
- ✅ NIST SP 800-132 (Password-Based Key Derivation)
- ✅ NIST SP 800-207 (Zero Trust Architecture)
- ✅ FIPS 140-2 approved algorithms

## Usage Examples

### Encryption
```rust
use meridian_security::encryption::aes::AesGcmEncryptor;
use meridian_security::encryption::{Encryptor, KeyGenerator};

// Generate key and encrypt
let key = AesGcmEncryptor::generate_key()?;
let encryptor = AesGcmEncryptor::new(&key)?;

let plaintext = b"Sensitive enterprise data";
let ciphertext = encryptor.encrypt(plaintext)?;
let decrypted = encryptor.decrypt(&ciphertext)?;
```

### Password Hashing
```rust
use meridian_security::hashing::password::PasswordHasher;

let hasher = PasswordHasher::new();

// Hash password with Argon2id
let password = "user-password-123";
let hash = hasher.hash_password(password.as_bytes())?;

// Verify password
let is_valid = hasher.verify_password(password.as_bytes(), &hash)?;
```

### JWT Authentication
```rust
use meridian_security::tokens::jwt::{JwtManager, TokenType};

let secret = b"your-secret-key-min-32-bytes-long";
let manager = JwtManager::new_hs256(secret, "my-app", "my-api")?;

// Create access token
let token = manager.create_token("user123", TokenType::Access, None)?;

// Verify token
let claims = manager.verify_token(&token)?;
```

### Zero-Trust Policy
```rust
use meridian_security::zero_trust::policy::{PolicyEngine, PolicyBuilder};
use meridian_security::zero_trust::context::RequestContext;

let mut engine = PolicyEngine::new();

// Add RBAC policy
let policy = PolicyBuilder::rbac(
    "/api/admin",
    "write",
    vec!["admin".to_string()]
);
engine.add_policy(policy);

// Evaluate access
let context = RequestContext::new("user123", "org456", "/api/admin", "write")
    .with_roles(vec!["admin".to_string()]);

let decision = engine.evaluate(&context)?;
```

### Secrets Vault
```rust
use meridian_security::secrets::vault::SecretsVault;
use meridian_security::encryption::envelope::EnvelopeEncryption;

// Create vault with encryption
let kek = EnvelopeEncryption::generate_kek()?;
let mut vault = SecretsVault::new(kek, 1)?;

// Store secret
vault.store("db-password", b"super-secret-password".to_vec())?;

// Retrieve secret
let secret = vault.get("db-password")?;
println!("Password: {}", secret.as_string()?);

// Rotate secret
vault.rotate("db-password", b"new-password".to_vec())?;
```

## Architecture

```
meridian-security/
├── encryption/          # Cryptographic primitives
│   ├── aes.rs          # AES-256-GCM
│   ├── chacha.rs       # ChaCha20-Poly1305
│   └── envelope.rs     # Envelope encryption
├── kms/                # Key management
│   ├── keyring.rs      # Key storage & rotation
│   └── derivation.rs   # KDF (HKDF, PBKDF2, Argon2)
├── hashing/            # Secure hashing
│   ├── password.rs     # Argon2id password hashing
│   └── hmac.rs         # HMAC authentication
├── tokens/             # Authentication tokens
│   ├── jwt.rs          # JWT management
│   └── refresh.rs      # Refresh tokens
├── zero_trust/         # Zero-trust architecture
│   ├── policy.rs       # Policy engine
│   └── context.rs      # Security context
├── audit/              # Security audit logging
│   └── mod.rs          # Audit events & loggers
└── secrets/            # Secrets management
    └── vault.rs        # Secrets vault
```

## Dependencies

- **ring**: Low-level cryptographic primitives
- **aes-gcm**: AES-GCM AEAD cipher
- **chacha20poly1305**: ChaCha20-Poly1305 AEAD
- **argon2**: Argon2 password hashing
- **jsonwebtoken**: JWT encoding/decoding
- **sha2**: SHA-2 family of hash functions
- **hmac**: HMAC message authentication

## Compliance

This security module helps meet compliance requirements for:

- **SOC 2 Type II**: Security controls and audit logging
- **ISO 27001**: Information security management
- **GDPR**: Data protection and encryption
- **HIPAA**: Healthcare data security
- **PCI-DSS**: Payment card data protection

## Security Considerations

### What This Module Does
- ✅ Provides cryptographic building blocks
- ✅ Implements secure defaults
- ✅ Follows industry best practices
- ✅ Enables compliance requirements
- ✅ Provides audit trails

### What You Must Do
- 🔒 Store KEKs in HSM or KMS (AWS KMS, Azure Key Vault)
- 🔒 Rotate keys regularly (90-day intervals)
- 🔒 Monitor audit logs for anomalies
- 🔒 Use TLS 1.3 for transport security
- 🔒 Implement rate limiting
- 🔒 Enable MFA for sensitive operations
- 🔒 Regular security audits and penetration testing

## Performance

### Benchmarks (approximate)
- AES-256-GCM encryption: ~1 GB/s (with AES-NI)
- ChaCha20-Poly1305: ~700 MB/s
- Argon2id hash (47 MiB): ~100 ms per hash
- JWT sign/verify: ~10,000 ops/sec

## Contributing

When contributing security-related code:
1. Follow OWASP secure coding guidelines
2. All cryptographic code must be reviewed
3. Add comprehensive tests
4. Update audit logging
5. Document security implications

## License

Proprietary - HarborGrid Enterprise

## Support

For security issues, contact: security@harborgrid.com

**DO NOT** open public issues for security vulnerabilities.
