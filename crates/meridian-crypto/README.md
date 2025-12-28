# Meridian Crypto

Enterprise-grade cryptographic services for the Meridian GIS Platform v0.1.5.

## Overview

The `meridian-crypto` crate provides a comprehensive suite of cryptographic services designed specifically for the Meridian GIS Platform. It implements industry-standard encryption, key management, digital signatures, and advanced privacy-preserving technologies.

## Features

### Core Cryptography
- **Envelope Encryption**: Multi-layer encryption using Data Encryption Keys (DEKs) and Key Encryption Keys (KEKs)
- **Field-Level Encryption**: Granular encryption of sensitive data fields with support for deterministic encryption
- **Key Derivation**: HKDF, PBKDF2, and Argon2 implementations for secure key derivation
- **Digital Signatures**: Ed25519, ECDSA (P-256/P-384), and RSA-PSS signature schemes

### Key Management
- **KMS Integration**: AWS KMS and HashiCorp Vault support
- **HSM Support**: Hardware Security Module integration for enhanced key security
- **Key Rotation**: Automatic key rotation with transparent re-encryption
- **Key Hierarchy**: Derive multiple keys from a master key for different purposes

### Transport Security
- **TLS Configuration**: Comprehensive TLS/SSL setup and management
- **Certificate Management**: X.509 certificate generation, validation, and chain verification
- **Mutual TLS**: Support for client certificate authentication

### Advanced Features
- **Zero-Knowledge Proofs**: Privacy-preserving proofs for spatial data using Bulletproofs
- **Homomorphic Encryption**: Experimental support for computation on encrypted data (feature-gated)
- **Cryptographic Audit Logging**: Comprehensive logging of all cryptographic operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
meridian-crypto = { version = "0.1.5", features = ["aws-kms", "vault", "zkp"] }
```

## Quick Start

### Envelope Encryption

```rust
use meridian_crypto::envelope::EnvelopeEncryption;
use aes_gcm::aead::OsRng;
use rand::RngCore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let envelope_enc = EnvelopeEncryption::new();

    // Generate a KEK
    let mut kek = vec![0u8; 32];
    OsRng.fill_bytes(&mut kek);

    // Encrypt sensitive GIS data
    let plaintext = b"Confidential location: 37.7749, -122.4194";
    let envelope = envelope_enc.encrypt(plaintext, &kek, "master-key-id", None)?;

    // Decrypt
    let decrypted = envelope_enc.decrypt(&envelope, &kek)?;
    assert_eq!(plaintext, decrypted.as_slice());

    Ok(())
}
```

### Field-Level Encryption

```rust
use meridian_crypto::field::{FieldEncryption, FieldEncryptionPolicy};

let mut field_enc = FieldEncryption::new();

// Register encryption policy
let policy = FieldEncryptionPolicy {
    encrypted_fields: vec!["email".to_string(), "coordinates".to_string()],
    key_id: "field-key-1".to_string(),
    deterministic: false,
    aad_template: Some("user:{field_path}".to_string()),
};

field_enc.register_policy("user", policy);
```

### Digital Signatures

```rust
use meridian_crypto::signature::{SignatureService, SignatureAlgorithm};

let mut service = SignatureService::new();

// Generate Ed25519 key pair
let key_id = service.generate_key_pair(SignatureAlgorithm::Ed25519)?;

// Sign data
let message = b"Critical GIS transaction data";
let signature = service.sign(&key_id, message)?;

// Verify signature
let verified = service.verify(&signature, message)?;
assert!(verified);
```

### AWS KMS Integration

```rust
#[cfg(feature = "aws-kms")]
use meridian_crypto::kms::aws::AwsKms;
use meridian_crypto::kms::{KeyManagementService, KeySpec};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let kms = AwsKms::new(Some("us-east-1".to_string())).await?;

    // Generate a data encryption key
    let key_pair = kms.generate_data_key(
        "alias/my-master-key",
        KeySpec::Aes256,
        None
    ).await?;

    // Use the plaintext key for encryption
    // Store the encrypted key securely

    Ok(())
}
```

### Zero-Knowledge Proofs

```rust
#[cfg(feature = "zkp")]
use meridian_crypto::zkp::{ZkpService, RangeProofParams};

let zkp = ZkpService::new();

// Prove a value is within a range without revealing it
let params = RangeProofParams {
    min_value: 0,
    max_value: 1000,
    bit_length: 16,
};

let value = 500u64;
let blinding = ZkpService::generate_blinding();

let proof = zkp.create_range_proof(value, &params, &blinding)?;
let verified = zkp.verify_range_proof(&proof)?;
assert!(verified);
```

## Feature Flags

- **`default`**: No features enabled by default
- **`aws-kms`**: Enable AWS KMS integration
- **`vault`**: Enable HashiCorp Vault integration
- **`zkp`**: Enable zero-knowledge proofs (requires bulletproofs)
- **`homomorphic`**: Enable homomorphic encryption (experimental)
- **`hsm-support`**: Enable HSM support

## Architecture

### Module Organization

```
meridian-crypto/
├── audit.rs           # Cryptographic audit logging
├── certificate.rs     # X.509 certificate management
├── derivation.rs      # Key derivation (HKDF, PBKDF2, Argon2)
├── envelope.rs        # Envelope encryption
├── error.rs           # Error types and handling
├── field.rs           # Field-level encryption
├── hsm.rs             # Hardware Security Module support
├── kms/
│   ├── mod.rs        # KMS abstraction
│   ├── aws.rs        # AWS KMS implementation
│   └── vault.rs      # HashiCorp Vault implementation
├── rotation.rs        # Key rotation and re-encryption
├── signature.rs       # Digital signatures
├── transport.rs       # TLS and transport security
└── zkp.rs            # Zero-knowledge proofs
```

## Security Considerations

### Best Practices

1. **Key Management**: Always use a KMS or HSM for production key storage
2. **Key Rotation**: Implement regular key rotation policies (90-day default)
3. **Audit Logging**: Enable comprehensive audit logging for compliance
4. **Secure Memory**: All sensitive key material uses zeroization on drop
5. **TLS**: Use TLS 1.3 for all network communications

### Cryptographic Algorithms

- **Symmetric Encryption**: AES-256-GCM, ChaCha20-Poly1305
- **Key Derivation**: HKDF-SHA256/512, PBKDF2-HMAC-SHA256/512, Argon2id
- **Digital Signatures**: Ed25519, ECDSA-P256/P384, RSA-PSS-2048/4096
- **Hash Functions**: SHA-256, SHA-384, SHA-512

## Performance

The library is designed for high-performance cryptographic operations:

- Envelope encryption: ~1-2ms per operation
- Field encryption: ~0.5-1ms per field
- Digital signatures (Ed25519): ~50-100μs per signature
- Signature verification: ~100-200μs per verification

## Compliance

This library is designed to support:

- **FIPS 140-2**: When used with approved algorithms and KMS/HSM
- **SOC 2**: Comprehensive audit logging support
- **GDPR**: Field-level encryption for PII protection
- **HIPAA**: Strong encryption and key management

## Examples

See the `examples/` directory for complete working examples:

- `envelope_encryption.rs` - Basic envelope encryption
- `kms_integration.rs` - AWS KMS integration
- `field_encryption.rs` - Encrypting database fields
- `signature_verification.rs` - Digital signature workflows
- `key_rotation.rs` - Key rotation strategies
- `audit_logging.rs` - Setting up audit logs

## Testing

Run the test suite:

```bash
# Run all tests
cargo test --all-features

# Run tests for a specific feature
cargo test --features aws-kms
cargo test --features zkp

# Run benchmarks
cargo bench
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass
2. Code is properly documented
3. Security implications are considered
4. Audit logging is implemented for new operations

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Support

For issues or questions:

- GitHub Issues: https://github.com/meridian/meridian-gis/issues
- Documentation: https://docs.meridian-gis.com/crypto
- Security Issues: security@meridian-gis.com (PGP key available)

## Version History

### v0.1.5 (Current)
- Initial release
- Envelope encryption with DEK/KEK hierarchy
- AWS KMS and HashiCorp Vault integration
- HSM support framework
- Key rotation with automatic re-encryption
- Field-level encryption for GIS data
- TLS/transport security
- Digital signatures (Ed25519, ECDSA, RSA)
- Certificate management
- Key derivation (HKDF, PBKDF2, Argon2)
- Comprehensive audit logging
- Zero-knowledge proofs for spatial data
- Experimental homomorphic encryption support
